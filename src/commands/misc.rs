

use std::sync::Arc;
use std::error::Error;
use tokio::sync::Mutex;

use prettytable::{Table, Row, Cell, format};

use twilight_model::channel::embed::Embed;
use twilight_model::user::User;
use twilight_model::gateway::{
    payload::MessageCreate,
};

use crate::{
    utils,
    embed,
    CmdState,
    anima::Anima,
    commands::CmdResult,
    anima::exp::Levelling,
    anima::exp::LevelChange,
    HttpClient, Redis
};

// Per evitare di scrivere sto schifo
type Failable<T> = Result<T, Box<dyn Error + Send + Sync>>;

// Risponse con un pong!
pub async fn ping(msg: &MessageCreate, state: CmdState) -> Failable<CmdResult> {
    state.http.create_message(msg.channel_id).content("pong!")?.await?;
    Ok(CmdResult::Skip)
}

// Invia statistiche dell'anima
pub async fn stats(msg: &MessageCreate, state: CmdState) -> Failable<CmdResult> {

    let anima: Anima = 
    {
        let mut redis = state.redis.lock().await;
        redis.get_anima(msg.author.id.0)?
    };

    let embed = embed::stats(&msg.author.name, &anima)?;
    state.http.create_message(msg.channel_id).embed(embed)?.await?;
    
    Ok(CmdResult::Skip)
}

const FORCED_EXP_GAIN: i32 = 500;

// Aumenta exp di un giocatore
pub async fn exp(msg: &MessageCreate, state: CmdState) -> Failable<CmdResult> {
    let mut redis = state.redis.lock().await;

    let mut anima = redis.get_anima(msg.author.id.0)?;
    utils::increase_exp(&mut redis, state.http, &mut anima, &msg, "Aumento di esperienza forzato", FORCED_EXP_GAIN).await?;
    redis.set_anima(msg.author.id.0, &anima)?;

    Ok(CmdResult::Skip)
}