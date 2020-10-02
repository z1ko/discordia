
use std::sync::Arc;
use serenity::{
    framework::standard::{
        CommandResult, Args,
        macros::{
            command, 
            group
        },
    },
    voice::{
        LockedAudio, 
        ytdl_search,
        ytdl
    },
    model::prelude::*,
    prelude::*
};

// Import the client's bridge to the voice manager. Since voice is a standalone
// feature, it's not as ergonomic to work with as it could be. The client
// provides a clean bridged integration with voice.
use serenity::client::bridge::voice::ClientVoiceManager;

use crate::{
    music::{
        Orchestra, 
        OrchestraMapKey
    }
};

pub struct VoiceMapKey;
impl TypeMapKey for VoiceMapKey {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

#[group]
#[commands(join, leave, play, stop, skip)]
pub struct Music;

//
// Si unisce al canale vocale
//

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {

    // Non possiamo collegarci ad un canale privato
    let guild = match msg.guild(&ctx.cache).await {
        Some(guild) => guild,
        None => {
            msg.channel_id.say(&ctx.http, "Non posso collegarmi qui").await?;
            return Ok(());
        }
    };

    // Ottiene canale vocale a cui collegarsi
    let channel = match guild.voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
    {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Ma non sei in un canale vocale").await?;
            return Ok(());
        }
    };

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<VoiceMapKey>()
        .unwrap().clone();

    let mut manager = mutex.lock().await;
    if let Some(_) = manager.join(guild.id, channel) {
        msg.channel_id.say(&ctx.http, &format!("Collegata al canale {}", channel.mention())).await?;
    }

    Ok(())
}

//
// Esce dal canale
//

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {

    // Non ha senso se è un canale privato, ottiene id della gilda
    let guild_id = match ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "Non posso collegarmi qui").await?;
            return Ok(());
        },
    };

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<VoiceMapKey>()
        .unwrap().clone();

    let mut manager = mutex.lock().await;
    if let Some(_) = manager.get(guild_id) {
        manager.remove(guild_id);

        // Resetta coda
        let mut orchestra = data.get_mut::<OrchestraMapKey>().unwrap();
        orchestra.reset().await;

        msg.channel_id.say(&ctx.http, "Ho lasciato il canale").await?;
        return Ok(());
    }

    msg.channel_id.say(&ctx.http, "Ma non sono in un canale vocale...").await?;
    Ok(())
}

//
// Riproduci musica sul canale vocale
//

#[command]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    // URL della canzone
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Devi dirmi l'URL del video").await?;
            return Ok(());
        },
    };

    // Ottiene gilda attiva
    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "Non trovo il canale").await?;
            return Ok(());
        },
    };

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<VoiceMapKey>()
        .unwrap().clone();

    let mut manager = mutex.lock().await;
    if let Some(player) = manager.get_mut(guild_id) {
        
        // Carica link o cerca su youtube
        let source = if url.starts_with("http") {
            match ytdl(&url).await {
                Ok(source) => source,
                Err(why) => {
                    msg.channel_id.say(&ctx.http, format!("Errore ottenimento risorsa; {}", why)).await?;
                    return Ok(());
                }
            }
        }
        else {
            match ytdl_search(args.rest()).await {
                Ok(source) => source,
                Err(why) => {
                    msg.channel_id.say(&ctx.http, format!("Errore ottenimento risorsa; {}", why)).await?;
                    return Ok(());
                }
            }
        };

        // Avvia la riproduzione ma pausa immediatamente, ci penserà l'orchestra
        // a far partire l'audio quando necessario
        let mut orchestra = data.get_mut::<OrchestraMapKey>().unwrap();
        let audio: LockedAudio = player.play_returning(source);
        orchestra.add(audio).await;

        return Ok(());
    }

    msg.channel_id.say(&ctx.http, "Ma non sono in un canale vocale...").await?;
    Ok(())
}

//
// Ferma la musica
//

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    
    // Non ha senso se è un canale privato, ottiene id della gilda
    let guild_id = match ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "Non posso farlo qui").await?;
            return Ok(());
        },
    };

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<VoiceMapKey>()
        .unwrap().clone();

    let mut manager = mutex.lock().await;
    if let Some(player) = manager.get_mut(guild_id) {
        player.stop();

        // Resetta coda
        let mut orchestra = data.get_mut::<OrchestraMapKey>().unwrap();
        orchestra.reset().await;

        msg.channel_id.say(&ctx.http, "Smetto di riprodurre la musica...").await?;
        return Ok(());
    }
    
    msg.channel_id.say(&ctx.http, "Ma non sto riproducendo nulla...").await?;
    Ok(())
}

//
// Salta alla prossima canzone
//

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    
    // Non ha senso se è un canale privato, ottiene id della gilda
    let guild_id = match ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "Non posso farlo qui").await?;
            return Ok(());
        },
    };

    let mut data = ctx.data.write().await;
    let mutex = data.get_mut::<VoiceMapKey>()
        .unwrap().clone();

    let mut manager = mutex.lock().await;
    if let Some(player) = manager.get_mut(guild_id) {

        // Salta il brano
        let mut orchestra = data.get_mut::<OrchestraMapKey>().unwrap();
        orchestra.skip().await;

        msg.channel_id.say(&ctx.http, "Passo al prossimo brano...").await?;
        return Ok(());
    }
    
    msg.channel_id.say(&ctx.http, "Ma non sto riproducendo nulla...").await?;
    Ok(())
}

