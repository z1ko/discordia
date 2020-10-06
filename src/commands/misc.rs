
use tracing::{info, warn};
use prettytable::{Table, Cell, Row, format};
use serenity::{
    framework::standard::{
        CommandResult,
        macros::{
            command, 
            group
        },
    },
    model::prelude::*,
    prelude::*
};

use crate::{
    redis::{Redis, RedisMapKey},
};

//
// Gruppo dei comandi generici per l'utente
//

#[group]
#[commands(ping, profile, balance)]
pub struct Misc;

//
// Risponde ad un ping con un pong
//

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Executing \"ping\" command");
    msg.reply(&ctx.http, "pong!").await?;
    Ok(())
}

//
// Invia statistiche dell'anima
//

#[command]
pub async fn profile(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Executing \"profile\" command");
    
    // =====================================================================
    // Ottiene anima tramite lock

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<RedisMapKey>()
        .unwrap().clone();
            
    let mut redis = mutex.lock().await;
    let anima = match redis.get_anima(msg.author.id.0) {
        Ok(anima) => anima,
        Err(why)  => {
            panic!("[ERROR] Can't get anima from redis server: {}", why);
        }
    };

    // =====================================================================
    // Crea tabella dell'anima

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.add_row(row!["money", anima.money]);
    table.add_row(row!["level", anima.level]);
    table.add_row(row!["  exp",   anima.exp]);

    // =====================================================================
    // Crea messaggio embedded

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&msg.author.name);
            e.description(format!("```\n{}\n```", table));
            e
        })
    }).await?;

    
    Ok(())
}

//
// Invia denaro attuale
//

#[command]
pub async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Executing \"balance\" command");

    // =====================================================================
    // Ottiene anima tramite lock

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<RedisMapKey>()
        .unwrap().clone();
            
    let mut redis = mutex.lock().await;
    let anima = match redis.get_anima(msg.author.id.0) {
        Ok(anima) => anima,
        Err(why)  => {
            panic!("[ERROR] Can't get anima from redis server: {}", why);
        }
    };

    // =====================================================================
    // Crea messaggio embedded

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(format!("Soldi di *{}*", msg.author.name));
            e.description(format!("```fix\n{}\n```", anima.money));
            e.color((230, 157, 2));
            e
        })
    }).await?;

    Ok(())
}

/*

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
    //utils::increase_exp(&mut redis, state.http, &mut anima, &msg, "Aumento di esperienza forzato", FORCED_EXP_GAIN).await?;
    redis.set_anima(msg.author.id.0, &anima)?;

    Ok(CmdResult::Skip)
}
*/