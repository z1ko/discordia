
use tracing::{info, warn};
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

use crate::{
    orchestra::{
        OrchestraMapKey
    }
};

#[group]
#[commands(join, leave, play, skip)]
pub struct Music;

//
// Si unisce al canale vocale
//

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Executing \"join\" command");

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
    let orchestra = data.get_mut::<OrchestraMapKey>()
        .unwrap();

    if orchestra.join(guild.id, channel).await {
        msg.channel_id.say(&ctx.http, &format!("Collegata al canale {}", channel.mention())).await?;
    }

    Ok(())
}

//
// Esce dal canale
//

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Executing \"leave\" command");

    // Non ha senso se è un canale privato, ottiene id della gilda
    let guild_id = match ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "Non posso collegarmi qui").await?;
            return Ok(());
        },
    };

    let mut data = ctx.data.write().await;
    let orchestra = data.get_mut::<OrchestraMapKey>()
        .unwrap();

    if orchestra.leave(guild_id).await {
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
    info!("Executing \"play\" command");

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

    // Carica link o cerca su youtube
    let source = if url.starts_with("http") {
        match ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                msg.channel_id.say(&ctx.http, format!("Errore ottenimento risorsa: {}", why)).await?;
                return Ok(());
            }
        }
    }
    else {
        match ytdl_search(args.rest()).await {
            Ok(source) => source,
            Err(why) => {
                msg.channel_id.say(&ctx.http, format!("Errore ottenimento risorsa: {}", why)).await?;
                return Ok(());
            }
        }
    };

    let mut data = ctx.data.write().await;
    let orchestra = data.get_mut::<OrchestraMapKey>()
        .unwrap();

    if orchestra.insert(guild_id, source).await {
        msg.channel_id.say(&ctx.http, "Riproduco la canzone").await?;
        return Ok(());
    }

    msg.channel_id.say(&ctx.http, "Ma non sono in un canale vocale...").await?;
    Ok(())
}

//
// Salta alla prossima canzone
//

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Executing \"skip\" command");
    
    // Non ha senso se è un canale privato, ottiene id della gilda
    let guild_id = match ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "Non posso farlo qui").await?;
            return Ok(());
        },
    };

    let mut data = ctx.data.write().await;
    let orchestra = data.get_mut::<OrchestraMapKey>()
        .unwrap();

    orchestra.skip(guild_id).await;
    msg.channel_id.say(&ctx.http, "Passo al prossimo brano...").await?;
    Ok(())
}