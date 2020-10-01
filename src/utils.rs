use twilight_model::gateway::payload::MessageCreate;

use crate::Filter;
use crate::Failable;
use crate::Anima;
use crate::Redis;

use crate::{
    response,
    tags::Tag,
    anima::exp::*,
    embed,
};
/*
/**
 * Insieme di patterns usati spesso
 */

/**
 * Aumenta livello anima, printa risultato e possibile level up su discord
 * @param message Messaggio da visualizzare nell'embed di aumento exp 
 */
pub async fn increase_exp(redis: &mut Redis, http: HttpClient, anima: &mut Anima,
    msg: &MessageCreate, message: &str, delta: i32) -> Failable<()>
{
    // Mostra sul canale discord
    let embed = embed::increase_exp(message, delta)?;
    http.create_message(msg.channel_id).embed(embed)?.await?;

    // Aumenta exp e se aumenta di livello mostra level ups
    if let LevelChange::Delta(old, new) = anima.increase_exp(delta) 
    {
        // Ottiene risposta per il level up
        let filter = Filter::new().tag(Tag::UserLevelUp);
        let response = match response::generate_response(redis, filter)? {
            None => String::from("Grazie della considerazione"),
            Some(response) => response,
        };    

        // Mostra sul canale discord
        let embed = embed::level_up(&msg.author.name, &response, old, new)?;
        http.create_message(msg.channel_id).embed(embed)?.await?;
    }

    Ok(())
}

/**
* Diminuisce livello anima, printa risultato e possibile level down su discord
* @param message Messaggio da visualizzare nell'embed di diminuzione exp 
*/
pub async fn decrease_exp(redis: &mut Redis, http: HttpClient, anima: &mut Anima,
    msg: &MessageCreate, message: &str, delta: i32) -> Failable<()>
{
    // Mostra sul canale discord
    let embed = embed::decrease_exp(message, delta)?;
    http.create_message(msg.channel_id).embed(embed)?.await?;

    // Diminuisce exp e se aumenta di livello mostra level ups
    if let LevelChange::Delta(old, new) = anima.decrease_exp(delta) 
    {
        // Ottiene risposta per il level up
        let filter = Filter::new().tag(Tag::UserLevelDown);
        let response = match response::generate_response(redis, filter)? {
            None => String::from("Complimenti pirla"),
            Some(response) => response,
        };    

        // Mostra sul canale discord
        let embed = embed::level_down(&msg.author.name, &response, old, new)?;
        http.create_message(msg.channel_id).embed(embed)?.await?;
    }

    Ok(())
}
*/