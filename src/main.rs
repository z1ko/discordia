
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

#[macro_use] 
extern crate prettytable;

use std::sync::Arc;
use std::{env, error::Error};
use std::io::{Write};

use tokio::time;
use rand::prelude::*;

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
        Tag, Filter, Commands
    },
    commands::{misc::*, music::*},
    music::{Orchestra, OrchestraMapKey},
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
    //let http = Http::new_with_token(&token);
    //let root = match http.get_current_application_info().await {
    //    Ok(info) => info.owner.id,
    //    Err(_)   => panic!("Owner not found"),
    //};
    //println!("[INFO] Current root user: {}", root);

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
const OTHER_BOT_DAMANGE_EXP: i32 = 100;

// Controller principale del bot
struct DiscordiaEventHandler;

#[async_trait] 
impl EventHandler for DiscordiaEventHandler
{
    /**
     * Gestisce la coda della musica
     */
    async fn ready(&self, ctx: Context, ready: Ready) {
      
        let mut interval = time::interval(time::Duration::from_secs(1));
        loop {
            let mut data = ctx.data.write().await;
            let mut orchesta = data.get_mut::<OrchestraMapKey>().unwrap();
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
            let mutex = data.get_mut::<RedisMapKey>()
                .unwrap().clone();
            
            let mut redis = mutex.lock().await;
    
            // Cerca risposta
            match redis.generate_response(Filter::new().tag(Tag::UsedOtherBot)).unwrap() {
                Some(response) => commands::embed_decrease_exp(&ctx, &msg, &response, OTHER_BOT_DAMANGE_EXP).await,
                None => eprintln!("[WARN] No response found"),
            }

            // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
            let mut anima = redis.get_anima(msg.author.id.0).unwrap();
            if let LevelChange::Delta(old, new) = anima.decrease_exp(OTHER_BOT_DAMANGE_EXP) 
            {
                match redis.generate_response(Filter::new().tag(Tag::UserLevelDown)).unwrap() {
                    Some(response) => commands::embed_level_down(&ctx, &msg, &response, old, new).await,
                    None => eprintln!("[WARN] No response found"),
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
async fn before(ctx: &Context, msg: &Message, cmd: &str) -> bool {
    println!("[INFO] Command {} from {}", cmd, msg.author.name);

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<RedisMapKey>().unwrap();

    let mut redis = mutex.lock().await;
    let anima = redis.get_anima(msg.author.id.0).unwrap();

    // Calcola probabilità di non eseguire il comando
    let n = anima.level as f32;
    let prob = -(2.0_f32).powf(-(0.7_f32) * n) + 1.0_f32;
    
    // Non esegue il comando
    if rand::thread_rng().gen::<f32>() > prob 
    {
        let filter = Filter::new()
            .tag(Tag::Command(cmd.to_string()))
            .tag(Tag::NoExec)
            .tag(Tag::Anima(msg.author.id.0));
        
        match redis.generate_response(filter).unwrap() {
            None => eprintln!("[WARN] No response found"),
            Some(response) => 
            {
                // TODO
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(response);
                        e
                    })
                })
                .await.unwrap();
            },
        }

        return false;
    }

    // Esegue il comando
    let filter = Filter::new()
        .tag(Tag::Command(cmd.to_string()))
        .tag(Tag::Anima(msg.author.id.0));

    match redis.generate_response(filter).unwrap() {
        None => eprintln!("[WARN] No response found"),
        Some(response) => 
        {
            // TODO
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
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
        Ok(()) => 
        {
            let mut data = ctx.data.write().await;
            let mutex = data.get_mut::<RedisMapKey>()
                .unwrap().clone();
                
            // Ottiene aumento exp dei comandi
            let exp = match command_name {
                ".play" => 100,
                _ => 0
            };
                
            if exp != 0 {
                let mut redis = mutex.lock().await;

                // Cerca risposta
                match redis.generate_response(Filter::new().tag(Tag::UserExpUp)).unwrap() {
                    Some(response) => commands::embed_increase_exp(&ctx, &msg, &response, exp).await,
                    None => eprintln!("[WARN] No response found"),
                }

                // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
                let mut anima = redis.get_anima(msg.author.id.0).unwrap();
                if let LevelChange::Delta(old, new) = anima.increase_exp(exp) 
                {
                    match redis.generate_response(Filter::new().tag(Tag::UserLevelDown)).unwrap() {
                        Some(response) => commands::embed_level_up(&ctx, &msg, &response, old, new).await,
                        None => eprintln!("[WARN] No response found"),
                    }
                }

                redis.set_anima(msg.author.id.0, &anima).unwrap();
            }
        }
    }
}
