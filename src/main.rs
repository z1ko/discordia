
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

use std::sync::Arc;
use std::{env, error::Error};
use std::io::{Write};

use serenity::{
    async_trait,
    model::{
        event::ResumedEvent, 
        gateway::Ready,
        channel::Message,
    },
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
    prelude::*,
};

use crate::{
    redis::{
        RedisMapKey, 
        Redis
    },
    anima::{
        exp::{
            LevelChange,
            Levelling
        },
        Anima
    },
    response::generate_response,
    tags::{
        Tag, Filter, Commands
    },
    commands::{misc::*,}
};

// Per evitare di scrivere sto schifo
type Failable<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Failable<()> {
    dotenv::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL")?;
    let token = std::env::var("DISCORD_TOKEN")?;   
    
    println!("\n{}", LOGO);
    println!("\n================================= INITIALIZATION =================================\n");

    print!("[INFO] Connecting to Redis server at {} ... ", redis_url);
    std::io::stdout().flush().unwrap();
    
    let redis = Arc::new(Mutex::new(Redis::connect(&redis_url)
        .expect("Error connecting to Redis server")));
    println!("[OK]");

    // TODO: Carica info sullo stato corrente
    let http = Http::new_with_token(&token);
    let root = match http.get_current_application_info().await {
        Ok(info) => info.owner.id,
        Err(_)   => panic!("Owner not found"),
    };
    println!("[INFO] Current root user: {}", root);

    // Crea framework di base
    let framework = StandardFramework::new()
        .configure(|ctx| ctx
            .prefix(".")
        )
        .group(&MISC_GROUP);

    print!("[INFO] Creating serenity client ... ");
    std::io::stdout().flush().unwrap();

    // Crea client per serenity
    let mut client = Client::new(&token)
        .framework(framework).await?;
    println!("[OK]");

    // Inserisce risorse globali
    {
        let mut data = client.data.write().await;
        data.insert::<RedisMapKey>(redis.clone());
    }

    println!("\n                                        Now things should just work... I hope :)");
    println!("==================================================================================");

    // Avvia serenity e lascia il controllo
    client.start().await?;
    Ok(())
}

// Controller principale del bot
struct DiscordiaEventHandler;

#[async_trait] 
impl EventHandler for DiscordiaEventHandler
{
    /**
     * Gestisce la coda della musica
     */
    async fn ready(&self, _: Context, ready: Ready) {
        // TODO        
    }

    /**
     * Gestisce eventi non legati ai comandi
     */
    async fn message(&self, ctx: Context, msg: Message) {

        // NOTE Se abbiamo rispost ad un altro bot
        if msg.content.starts_with("!") || msg.content.starts_with("-")
        {
            let mut data = ctx.data.write().await;
            let mutex = data.get_mut::<RedisMapKey>()
                .unwrap().clone();
            
            let mut redis = mutex.lock().await;
    
            // Ottiene risposta per l'utente e se questa esiste
            // la invia alla chat discrod
            let filter = Filter::new().tag(Tag::UsedOtherBot);
            let response = match response::generate_response(&mut redis, filter).unwrap() {
                Some(response) => response,
                None => {
                    eprintln!("[WARN] No responses found for tag");
                    String::default()
                }
            };

            // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
            let mut anima = redis.get_anima(msg.author.id.0).unwrap();
            
            // TODO
            //utils::decrease_exp(&mut redis, state.http, &mut anima, &msg, &response, exp_damage).await;

            redis.set_anima(msg.author.id.0, &anima).unwrap();
        }        
    }
}


/*
// Smista comandi e in base al risultato aumenta exp utente
async fn handle_command(msg: MessageCreate, shard_id: u64, state: CmdState) -> Failable<()> {
    
    // Divide comando in argomenti
    let args = msg.content.split(' ').collect::<Vec<&str>>();
    
    // Smista e conserva risultato comando per aumentare exp se richiesto
    let result = match args[0]
    {
        //       MISC
        ".ping"  => commands::misc::ping(&msg, state.clone()).await?,
        ".stats" => commands::misc::stats(&msg, state.clone()).await?,
        ".exp"   => commands::misc::exp(&msg, state.clone()).await?,

        //       MUSIC
        ".join"  => commands::music::join(&msg, shard_id, state.clone()).await?,
        ".leave" => commands::music::leave(&msg, shard_id, state.clone()).await?,
        ".play"  => commands::music::play(&msg, state.clone()).await?,

        _ => { CmdResult::Skip }
    };
    
    // Se richiesto aumenta exp
    if let CmdResult::Success(exp) = result {
        let mut redis = state.redis.lock().await;

        // Aggiorna exp utente
        let mut anima = redis.get_anima(msg.author.id.0)?;
        utils::increase_exp(&mut redis, state.http, &mut anima, &msg, "", exp).await?;
        redis.set_anima(msg.author.id.0, &anima)?;
    }
   
    println!("DIO");
    Ok(())
}
*/