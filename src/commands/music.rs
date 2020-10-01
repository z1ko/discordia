use std::sync::Arc;
use std::error::Error;
use tokio::sync::Mutex;
use std::convert::TryInto;

use prettytable::{Table, Row, Cell, format};

use reqwest::Client as ReqwestClient;

use twilight_lavalink::Lavalink;
use twilight_lavalink::http::LoadedTracks;
use twilight_lavalink::model::Destroy;
use twilight_lavalink::model::Play;
use twilight_model::id::ChannelId;
use twilight_model::user::User;
use twilight_model::gateway::{
    payload::MessageCreate,
};

use twilight_gateway::{
    Shard
};

use crate::{
    HttpClient, 
    Redis
};

// Per evitare di scrivere sto schifo
type Failable<T> = Result<T, Box<dyn Error + Send + Sync>>;

// Si unisce al canale discord
pub async fn join(msg: &MessageCreate, shard: Shard, http: HttpClient) -> Failable<()> {
    println!("[INFO] join cmd in channel {} by {}", msg.channel_id, msg.author.name);

    let args: Vec<&str> = msg.content.split(' ').collect();
    if args.len() == 2 {
        if let Ok(channel_id) = args[1].parse::<u64>() 
        {
            // Comando per unirsi al canale
            println!("[INFO] Collegamento al canale {}", channel_id);
            shard.command(&serde_json::json!({
                "op": 4, "d": 
                {
                    "channel_id": channel_id,
                    "guild_id": msg.guild_id,
                    "self_mute": false,
                    "self_deaf": false,
                }
            }))
            .await?;

            http.create_message(msg.channel_id)
                .content(format!("Collegata al canale <#{:?}>!", channel_id))?
                .await?;
        } 
        else
        {
            println!("[INFO] Fallimento collegamento: Canale non valido");
            http.create_message(msg.channel_id)
                .content("Il canale specificato non è corretto")?
                .await?;
        }
    }
    else
    {
        println!("[INFO] Fallimento collegamento: Canale non specificato");
        http.create_message(msg.channel_id)
            .content("Non hai specificato il canale")?
            .await?;
    }

    Ok(())
}

// Lascia il canale vocale
pub async fn leave(msg: &MessageCreate, shard: Shard, lavalink: Lavalink, http: HttpClient) -> Failable<()> {
    println!("[INFO] leave cmd in channel {} by {}", msg.channel_id, msg.author.name);

    let guild_id = msg.guild_id.unwrap();
    if let Some(player) = lavalink.players().get(&guild_id) {

        // Rimuove player di lavalink dal canale
        println!("[INFO] Rimozione del Lavalink player dalla guild {}", guild_id);
        player.send(Destroy::from(guild_id))?;

        // Rimuove bot dal canale
        println!("[INFO] Rimozione del bot dalla guild {}", guild_id);
        shard.command(&serde_json::json!({
            "op": 4, "d": 
            {
                "channel_id": None::<ChannelId>,
                "guild_id": msg.guild_id,
                "self_mute": false,
                "self_deaf": false,
            }
        }))
        .await?;

        http.create_message(msg.channel_id)
            .content("Ho lasciato il canale vocale!")?
            .await?;
    }
    else
    {
        println!("[INFO] Rimozione fallita: Lavalink player non presente nel canale specificato");
        http.create_message(msg.channel_id)
            .content("Non sto riproducendo musica in quel canale")?
            .await?;
    }

    Ok(())
}

// Riproduce una musica nel canale vocale
pub async fn play(msg: &MessageCreate, lavalink: Lavalink, http: HttpClient) -> Failable<()> {
    println!("[INFO] play cmd in channel {} by {}", msg.channel_id, msg.author.name);

    let args: Vec<&str> = msg.content.split(' ').collect();
    if args.len() == 2 
    {
        let search = args[1];
        println!("[INFO] Ricerca di \"{}\"", search);

        let guild_id = msg.guild_id.unwrap();
        if let Some(player) = lavalink.players().get(&guild_id) {
            println!("[INFO] Lavalink player found for guild {}", guild_id);

            // Ricerca il link nel web tramite lavalink
            let config = player.node().config();
            let request = twilight_lavalink::http::load_track(config.address, &search, &config.authorization)?
                .try_into()?;
                
            // Invia richiesta HTTP a lavalink
            // TODO Conserva client nello stato dell'app
            let client = ReqwestClient::new();
            let result = client.execute(request).await?;
            let result = result.json::<LoadedTracks>().await?;

            // Riproduce la canzone se abbiamo trovato qualcosa
            if let Some(track) = result.tracks.first() 
            {
                println!("[INFO] \"{:?}\" ottenuto pronto alla riproduzione", track.info.title);
                player.send(Play::from((guild_id, &track.track)))?;

                let content = format!("Riproducendo **{:?}** di **{:?}**", track.info.title, track.info.author);
                http.create_message(msg.channel_id)
                    .content(content)?
                    .await?;
            }
            else
            {
                println!("[INFO] Ricerca fallita: nessun risultato");
                http.create_message(msg.channel_id)
                    .content("Non ho trovato risultati...")?
                    .await?;
            }
        }
        else
        {
            println!("[INFO] Ricerca fallita: nessun player per questa gilda");
                http.create_message(msg.channel_id)
                    .content("Non sono in un canale vocale...")?
                    .await?;
        }
    }
    else
    {
        println!("[INFO] Riproduzione fallita: nessun argomento al comando");
        http.create_message(msg.channel_id)
            .content("Non hai specificato cosa riprodurre")?
            .await?;
    }

    Ok(())
}