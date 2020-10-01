
use std::sync::Arc;
use serenity::{
    framework::standard::{
        CommandResult, Args,
        macros::{
            command, 
            group
        },
    },
    voice::ytdl,
    model::prelude::*,
    prelude::*
};

// Import the client's bridge to the voice manager. Since voice is a standalone
// feature, it's not as ergonomic to work with as it could be. The client
// provides a clean bridged integration with voice.
use serenity::client::bridge::voice::ClientVoiceManager;

pub struct VoiceMapKey;
impl TypeMapKey for VoiceMapKey {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

#[group]
#[commands(join, leave, play)]
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

    // Non ha senso se è un canale privato
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
    if let Some(_) = manager.get(guild_id) 
    {
        manager.remove(guild_id);
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

    if !url.starts_with("http") {
        msg.channel_id.say(&ctx.http, "Serve un URL valido").await?;
        return Ok(());
    }

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
        let source = match ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                msg.channel_id.say(&ctx.http, format!("Errore ottenimento risorsa; {}", why)).await?;
                return Ok(());
            },
        };

        player.play(source);
        msg.channel_id.say(&ctx.http, "Riproduco {}").await?;
        return Ok(());
    }

    msg.channel_id.say(&ctx.http, "Ma non sono in un canale vocale...").await?;
    Ok(())
}