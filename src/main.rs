
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
mod commands;
mod tags;
mod music;
mod affinity;

#[macro_use] 
extern crate prettytable;

use std::sync::Arc;
use std::{env, error::Error};
use std::io::{Write};

use tokio::time;
use rand::prelude::*;

use tracing::{info, warn};
use tracing_subscriber;

use serenity::{
    async_trait,
    model::{
        event::ResumedEvent, 
        gateway::Ready,
        channel::Message,
    },
    framework::{
        StandardFramework,
        standard::{
            CommandResult,
            macros::hook
        },
    },
    http::Http,
    prelude::*,
    voice,
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
    tags::{
        Tag, Filter
    },
    affinity::{Affinity, AffinityChange},
    commands::{Commands, misc::*, music::*},
    music::{Orchestra, OrchestraMapKey},
};

// Per evitare di scrivere sto schifo
type Failable<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Failable<()> {
    dotenv::dotenv().ok();

    // Carica contesto di tracing
    tracing_subscriber::fmt::init();

    let redis_url = std::env::var("REDIS_URL")?;
    let token = std::env::var("DISCORD_TOKEN")?;   
    
    print!("\n{}", LOGO);
    print!("\n================================= INITIALIZATION =================================\n");

    print!("[INFO] Connecting to Redis server at {} ... ", redis_url);
    std::io::stdout().flush().unwrap();
    
    let redis = Arc::new(Mutex::new(Redis::connect(&redis_url)
        .expect("Error connecting to Redis server")));
    println!("[OK]");

    // Crea framework di base
    let framework = StandardFramework::new()
        .configure(|ctx| ctx
            .prefix(".")
        )
        .group(&MUSIC_GROUP)
        .group(&MISC_GROUP);

    print!("[INFO] Creating serenity client ... ");
    std::io::stdout().flush().unwrap();

    // Crea client per serenity
    let mut client = Client::new(&token)
        .event_handler(DiscordiaEventHandler)
        .framework(framework)
        .await?;
    println!("[OK]");

    // Inserisce risorse globali
    {
        let mut data = client.data.write().await;
        data.insert::<RedisMapKey>(redis.clone());
        data.insert::<VoiceMapKey>(client.voice_manager.clone());

        let orchestra = Orchestra::new();
        data.insert::<OrchestraMapKey>(orchestra);
    }

    println!("\n                                        Now things should just work... I hope :)");
    println!("==================================================================================");

    // Avvia serenity e lascia il controllo
    client.start().await?;
    Ok(())
}

// Danno di exp per usare un altro bot
const OTHER_BOT_DAMANGE_EXP: i8 = -50;

// Controller principale del bot
struct DiscordiaEventHandler;

#[async_trait] 
impl EventHandler for DiscordiaEventHandler
{
    /**
     * Gestisce la coda della musica
     */
    async fn ready(&self, ctx: Context, _: Ready) {
      
        let mut interval = time::interval(time::Duration::from_secs(1));
        loop {
            let mut data = ctx.data.write().await;
            let orchesta = data.get_mut::<OrchestraMapKey>().unwrap();
            orchesta.update().await;

            // Dorme per 1 secondi
            interval.tick().await;
        }
    }

    /**
     * Gestisce eventi non legati ai comandi
     */
    async fn message(&self, ctx: Context, msg: Message) {

        // NOTE Se abbiamo rispost ad un altro bot
        if msg.content.starts_with("!") || msg.content.starts_with("-")
        {
            let mut data = ctx.data.write().await;
            let mutex = data.get_mut::<RedisMapKey>().unwrap().clone();
            let mut redis = mutex.lock().await;
    
            let filter = Filter::new()
                .tag(Tag::Anima(msg.author.id.0))
                .tag(Tag::UsedOtherBot);

            // Cerca risposta
            match redis.generate_response(filter).unwrap() {
                Some(response) => commands::embed_affinity_score_change(&ctx, &msg, &response, OTHER_BOT_DAMANGE_EXP).await,
                None => {
                    msg.channel_id.say(&ctx.http, "[WARN] No response found").await.unwrap();
                }
            }

            // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
            let mut anima = redis.get_anima(msg.author.id.0).unwrap();
            if let AffinityChange::Some(old, new) = anima.affinity_sub(OTHER_BOT_DAMANGE_EXP.abs() as u8) 
            {
                let filter = Filter::new()
                    .tag(Tag::Anima(msg.author.id.0))
                    .tag(Tag::UserLevelDown);

                // Cerca risposta
                match redis.generate_response(filter).unwrap() {
                    Some(response) => commands::embed_affinity_level_change(&ctx, &msg, &response, old, new).await,
                    None => {
                        msg.channel_id.say(&ctx.http, "[WARN] No response found").await.unwrap();
                    }
                }
            }

            redis.set_anima(msg.author.id.0, &anima).unwrap();
        }
    }
}

//
// Invocato prima dell'esecuzione di un programma
//

#[hook]
async fn before(ctx: &Context, msg: &Message, cmd: &str) -> bool 
{
    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<RedisMapKey>().unwrap().clone();
    let mut redis = mutex.lock().await;

    let filter = Filter::new()
        .tag(Tag::Command(Commands::from_str(cmd).unwrap()))
        .tag(Tag::Anima(msg.author.id.0));

    // Printa risposta sul canale
    match redis.generate_response(filter).unwrap() {
        None => {
            msg.channel_id.say(&ctx.http, "[WARN] No response found").await.unwrap();
        }
        Some(response) => {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.description(response);
                    e
                })
            })
            .await.unwrap();
        },
    }

    true
}

//
// Invocato dopo l'esecuzione di un comando
//

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) 
{
    match command_result {
        Err(why) => eprintln!("Command '{}' returned error {:?}", command_name, why),
        Ok(()) => {          

            // Ottiene aumento exp dei comandi
            let command = Commands::from_str(command_name).unwrap();
            let change: i8 = match command {
                Commands::Play    => 6,
                Commands::Balance => 2,
                Commands::Profile => 2,
                _ => 0,
            };  

            if change != 0 
            {
                let mut data = ctx.data.write().await;
                let mutex = data.get_mut::<RedisMapKey>().unwrap().clone();
                let mut redis = mutex.lock().await;

                let filter = Filter::new()
                    .tag(Tag::Anima(msg.author.id.0))
                    .tag(Tag::UserExpUp);

                // Cerca risposta
                match redis.generate_response(filter).unwrap() {
                    Some(response) => commands::embed_affinity_score_change(&ctx, &msg, &response, change).await,
                    None => {
                        //msg.channel_id.say(&ctx.http, "[WARN] No response found").await.unwrap();
                    }
                }

                // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
                let mut anima = redis.get_anima(msg.author.id.0).unwrap();
                if let AffinityChange::Some(old, new) = anima.affinity_add(change as u8) 
                {
                    let filter = Filter::new()
                        .tag(Tag::Anima(msg.author.id.0))
                        .tag(Tag::UserLevelUp);

                    match redis.generate_response(filter).unwrap() {
                        Some(response) => commands::embed_affinity_level_change(&ctx, &msg, &response, old, new).await,
                        None => {
                            //msg.channel_id.say(&ctx.http, "[WARN] No response found").await.unwrap();
                        }
                    }
                }

                redis.set_anima(msg.author.id.0, &anima).unwrap();
            }
        }
    }
}