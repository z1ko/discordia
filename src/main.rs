
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
    commands::{Commands, misc::*, music::*},
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
                Some(response) => commands::embed_decrease_exp(&ctx, &msg, &response, OTHER_BOT_DAMANGE_EXP).await,
                None => eprintln!("[WARN] No response found"),
            }

            // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
            let mut anima = redis.get_anima(msg.author.id.0).unwrap();
            if let LevelChange::Delta(old, new) = anima.decrease_exp(OTHER_BOT_DAMANGE_EXP) 
            {
                let filter = Filter::new()
                    .tag(Tag::Anima(msg.author.id.0))
                    .tag(Tag::UserLevelDown);

                // Cerca risposta
                match redis.generate_response(filter).unwrap() {
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
    let mutex = data.get_mut::<RedisMapKey>().unwrap().clone();
    let mut redis = mutex.lock().await;

    // Calcola probabilità di non eseguire il comando
    let anima = redis.get_anima(msg.author.id.0).unwrap();
    let prob = -(2.0_f32).powf(-(0.7_f32) * anima.level as f32) + 1.0_f32;
    
    // Descide se eseguire il comando, e in base a quello
    // crea un filtro corretto 
    let execute = rand::thread_rng().gen::<f32>() > prob;
    let filter = if execute
    {
        Filter::new()
            .tag(Tag::Anima(msg.author.id.0))
            .tag(Tag::NoExec)
    }
    else
    {
        Filter::new()
            .tag(Tag::Command(Commands::from_str(cmd).unwrap()))
            .tag(Tag::Anima(msg.author.id.0))
    };

    // Printa risposta sul canale
    match redis.generate_response(filter).unwrap() {
        None => eprintln!("[WARN] No response found"),
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

    execute
}

//
// Invocato dopo l'esecuzione di un comando
//

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Err(why) => eprintln!("Command '{}' returned error {:?}", command_name, why),
        Ok(()) => {          

            // Ottiene aumento exp dei comandi
            let command = Commands::from_str(command_name).unwrap();
            let change = match command {
                Commands::Play => 50,
                _ => 0,
            };  

            if change != 0 
            {
                let mut data = ctx.data.write().await;
                let mutex = data.get_mut::<RedisMapKey>().unwrap().clone();
                let mut redis = mutex.lock().await;

                // Cerca risposta
                match redis.generate_response(Filter::new().tag(Tag::UserExpUp)).unwrap() {
                    Some(response) => commands::embed_increase_exp(&ctx, &msg, &response, change).await,
                    None => eprintln!("[WARN] No response found"),
                }

                // Ottiene anima e decrementa l'exp mostrando il risultato su Discord
                let mut anima = redis.get_anima(msg.author.id.0).unwrap();
                if let LevelChange::Delta(old, new) = anima.increase_exp(change) 
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

// Pattern di modifica riutilizzati
mod helpers
{
    use super::*;

    pub async fn decrease_exp(ctx: &Context, msg: &Message, redis: &mut Redis, anima_id: u64, delta: i32)
    {
        let mut anima = redis.get_anima(anima_id).unwrap();
        if let LevelChange::Delta(old, new) = anima.decrease_exp(delta) 
        {
            let filter = Filter::new()
                .tag(Tag::Anima(anima_id))
                .tag(Tag::UserLevelDown);

            // Cerca risposta
            match redis.generate_response(filter).unwrap() {
                Some(response) => commands::embed_level_down(&ctx, &msg, &response, old, new).await,
                None => eprintln!("[WARN] No response found"),
            }
        }
    }

    pub async fn increase_exp(ctx: &Context, msg: &Message, redis: &mut Redis, anima_id: u64, delta: i32)
    {
        let mut anima = redis.get_anima(anima_id).unwrap();
        if let LevelChange::Delta(old, new) = anima.increase_exp(delta)
        {
            let filter = Filter::new()
                .tag(Tag::Anima(anima_id))
                .tag(Tag::UserLevelUp);

            // Cerca risposta
            match redis.generate_response(filter).unwrap() {
                Some(response) => commands::embed_level_up(&ctx, &msg, &response, old, new).await,
                None => eprintln!("[WARN] No response found"),
            }
        }
    }
}