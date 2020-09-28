
mod redis;
mod anima;
mod response;
mod commands;
mod formatting;
mod tags;

#[macro_use] 
extern crate prettytable;

use std::sync::Arc;
use std::{env, error::Error};
use tokio::stream::StreamExt;
use tokio::sync::Mutex;

use twilight_gateway::{
    cluster::{
        Cluster, ShardScheme
    }, 
    Event
};

use twilight_model::gateway::Intents;
use twilight_http::Client as HttpClient;
use twilight_cache_inmemory::{
    EventType, InMemoryCache
};

use crate::{
    redis::Redis,
    anima::Anima,
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

    let redis_url = std::env::var("REDIS_DATABASE_URL").expect("REDIS_DATABASE_URL not found in env");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found in env");

    print!("Connecting to Redis server at {} ... ", redis_url);
    let mut redis = Arc::new(Mutex::new(Redis::connect(&redis_url)
        .expect("Error connecting to Redis server")));
    println!("[OK]");

    // Crea quante shard vuole discord
    let scheme = ShardScheme::Auto;
    
    print!("Creating shard cluster ... ");
    let cluster = Cluster::builder(&token)
        .shard_scheme(scheme)
        .intents(Intents::GUILD_MESSAGES)
        .build().await?;
    println!("[OK]");
    
    // Crea client http per richieste all'API
    let http = HttpClient::new(&token);

    // Fai partile le shard in background
    print!("Starting shard cluster ... ");
    let cluster_core = cluster.clone();
    tokio::spawn(async move {
        cluster_core.up().await;
    });
    println!("[OK]");

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
    let mut events = cluster.events();
    while let Some((shard_id, event)) = events.next().await 
    {
        // Processa evento in un altro thread
        cache.update(&event);
        tokio::spawn(message(shard_id, event, Arc::clone(&redis), http.clone()));
    }

    Ok(())
}

// Handler dei messaggi
async fn message(shard_id: u64, event: Event, redis: Arc<Mutex<Redis>>, http: HttpClient) -> Failable<()> {
    match event {
        
        // Nuovo messaggio ricevuto
        Event::MessageCreate(msg)  => {
            match msg.content.as_str() {
                ".ping"  => commands::misc::ping(&msg, redis, http).await?,
                ".stats" => commands::misc::stats(&msg, redis, http).await?,
                _ => { }
            }
        }

        Event::ShardConnected(_) =>
            println!("Connected on shard {}", shard_id),

        // Other events here...
        _ => {}
    }

    Ok(())
}
