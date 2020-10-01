
// Perché i terminali non sono mai abbastanza belli
static LOGO: &str = r#"
                  ·▄▄▄▄  ▪  .▄▄ ·  ▄▄·       ▄▄▄  ·▄▄▄▄  ▪   ▄▄▄· 
                  ██▪ ██ ██ ▐█ ▀. ▐█ ▌▪▪     ▀▄ █·██▪ ██ ██ ▐█ ▀█ 
                  ▐█· ▐█▌▐█·▄▀▀▀█▄██ ▄▄ ▄█▀▄ ▐▀▀▄ ▐█· ▐█▌▐█·▄█▀▀█ 
                  ██. ██ ▐█▌▐█▄▪▐█▐███▌▐█▌.▐▌▐█•█▌██. ██ ▐█▌▐█ ▪▐▌
                  ▀▀▀▀▀• ▀▀▀ ▀▀▀▀ ·▀▀▀  ▀█▄▀▪.▀  ▀▀▀▀▀▀• ▀▀▀ ▀  ▀ 
                                                    by Z1ko
"#;

mod redis;
mod anima;
mod response;
mod commands;
mod tags;
mod embed;
mod utils;

#[macro_use] 
extern crate prettytable;

use twilight_model::id::ChannelId;
use twilight_model::gateway::payload::MessageCreate;
use std::str::FromStr;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::{env, error::Error};
use std::io::{self, Write};

use tokio::stream::StreamExt;
use tokio::sync::Mutex;

use reqwest::Client as ReqwestClient;

use twilight_gateway::{
    cluster::{
        Cluster, ShardScheme
    }, 
    Event
};

use twilight_model::gateway::Intents;
use twilight_http::Client as HttpClient;
use twilight_lavalink::{Lavalink};
use twilight_cache_inmemory::{
    EventType, InMemoryCache
};

use crate::{
    redis::Redis,
    anima::{
        exp::{
            LevelChange,
            Levelling
        },
        Anima
    },
    commands::{
        state::CmdState,
        CmdResult,
    },
    response::generate_response,
    tags::{
        Tag, Filter, Commands
    }
};

// Per evitare di scrivere sto schifo
type Failable<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Failable<()> {
    dotenv::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL")?;
    let token = "NzU2MDcxNzQyOTU4NjAwMjkz.X2Mgrg.9xEU6HMHCACEQ81znCgVNa7Fcow".to_string();//std::env::var("DISCORD_TOKEN")?;   
    
    let lavalink_psw = std::env::var("LAVALINK_PSW")?;
    let lavalink_url = std::env::var("LAVALINK_URL")?;
    let lavalink_socket = SocketAddr::from_str(&lavalink_url)?;

    println!("\n{}", LOGO);
    println!("\n================================= INITIALIZATION =================================\n");

    print!("[INFO] Connecting to Redis server at {} ... ", redis_url);
    std::io::stdout().flush().unwrap();
    
    let redis = Arc::new(Mutex::new(Redis::connect(&redis_url)
        .expect("Error connecting to Redis server")));
    println!("[OK]");

    // Crea quante shard vuole discord
    let scheme = ShardScheme::Auto;
    let shard_count = 1;
    
    print!("[INFO] Creating shard cluster using token ... ");
    std::io::stdout().flush().unwrap();

    let cluster = Cluster::builder(&token)
        .shard_scheme(scheme)
        .intents(Intents::GUILD_MESSAGES)
        .build().await?;
    println!("[OK]");
    
    // Crea client http per richieste all'API e ottiene l'id del bot
    let http = HttpClient::new(&token);
    let reqwest = ReqwestClient::new();
    let user = http.current_user().await?;

    /*
    print!("[INFO] Connecting to Lavalink at {} ... ", lavalink_socket); 
    std::io::stdout().flush().unwrap();
    
    // Creazione collegamento a lavalink
    let lavalink = Lavalink::new(user.id, shard_count);
    lavalink.add(lavalink_socket, lavalink_psw)
        .await.expect("Error connecting Lavalink");
    println!("[OK]");
    */

    print!("[INFO] Starting shard cluster ... ");
    std::io::stdout().flush().unwrap();
    
    // Fai partile le shard in background
    let cluster_core = cluster.clone();
    tokio::spawn(async move {
        cluster_core.up().await;
    });
    println!("[OK]");

    println!("\n                                        Now things should just work... I hope :)");
    println!("==================================================================================");

    // La cache per ora contiene solo i messaggi
    let cache = InMemoryCache::builder()
        .event_types(
            EventType::MESSAGE_CREATE      | 
            EventType::MESSAGE_DELETE      | 
            EventType::MESSAGE_DELETE_BULK | 
            EventType::MESSAGE_UPDATE,
        )
        .build();

    // Ottiene messaggi dal sink del cluster
    let cluster_core = cluster.clone(); 
    let mut events = cluster_core.events();

    // Template dello stato usabile dei comandi
    let state = CmdState { redis, cluster, http, reqwest };
    while let Some((shard_id, event)) = events.next().await 
    {
        cache.update(&event);
        //lavalink.process(&event).await?;

        // Smista eventi
        handle_event(shard_id, event, state.clone())?;
    }

    Ok(())
}

// Smista gli eventi e spawna i thread nel caso di comando
fn handle_event(shard_id: u64, event: Event, state: CmdState) -> Failable<()> {
    match event
    {
        // Nuovo messaggio
        Event::MessageCreate(msg) => 
        {
            // Se usa un altro bot
            if msg.content.starts_with("!") || msg.content.starts_with("-") {
                tokio::spawn(handle_other_bot_use((*msg).clone(), state.clone()));
                return Ok(());
            }

            // Se è un comando lo smista
            if msg.content.starts_with(".") {
                tokio::spawn(handle_command((*msg).clone(), state.clone()));
            }
        },

        // Shard connessa o riconnessa al server
        Event::ShardConnected(_) => 
            println!("[INFO] Shard {} is connected", shard_id),

        _ => { /* TODO */ }
    }

    Ok(())
}

// Insulta l'utente e rimuove esperienza
async fn handle_other_bot_use(msg: MessageCreate, state: CmdState) -> Failable<()> {
    let mut redis = state.redis.lock().await;
    
    let exp_damage: i32 = 50;

    // Ottiene risposta per l'utente e se questa esiste
    // la invia alla chat discrod
    let filter = Filter::new().tag(Tag::UsedOtherBot);
    let response = match response::generate_response(&mut redis, filter)? {
        Some(response) => response,
        None => String::default(),
    };

    // Ottiene anima e decrementa l'exp
    let mut anima = redis.get_anima(msg.author.id.0).unwrap();
    utils::decrease_exp(&mut redis, state.http, &mut anima, &msg, &response, exp_damage).await?;
    redis.set_anima(msg.author.id.0, &anima).unwrap();

    Ok(())
} 

// Quanto aumentare l'exp dell'utente dopo un comando con successo
const CMD_EXP_GAIN: i32 = 100;

// Smista comandi e in base al risultato aumenta exp utente
async fn handle_command(msg: MessageCreate, state: CmdState) -> Failable<()> {
    
    // Divide comando in argomenti
    let args = msg.content.split(' ').collect::<Vec<&str>>();
    
    // Smista e conserva risultato comando per aumentare exp se richiesto
    let result = match args[0]
    {
        //       MISC
        ".ping"  => commands::misc::ping(&msg, state.clone()).await?,
        ".stats" => commands::misc::stats(&msg, state.clone()).await?,
        ".exp"   => commands::misc::exp(&msg, state.clone()).await?,

        _ => { CmdResult::Skip }
    };
    
    // Se richiesto aumenta exp
    if result == CmdResult::Success {
        let mut redis = state.redis.lock().await;

        // Aggiorna exp utente
        let mut anima = redis.get_anima(msg.author.id.0)?;
        utils::increase_exp(&mut redis, state.http, &mut anima, &msg, "", CMD_EXP_GAIN).await?;
        redis.set_anima(msg.author.id.0, &anima)?;
    }
    
    Ok(())
}