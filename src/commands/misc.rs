
use std::sync::Arc;
use std::error::Error;
use tokio::sync::Mutex;

use prettytable::{Table, Row, Cell, format};
use twilight_model::user::User;
use twilight_model::gateway::{
    payload::MessageCreate,
};


use crate::{
    formatting,
    anima::Anima,
    HttpClient, Redis
};

// Per evitare di scrivere sto schifo
type Failable<T> = Result<T, Box<dyn Error + Send + Sync>>;

// Risponse con un pong!
pub async fn ping(msg: &MessageCreate, _: Arc<Mutex<Redis>>, http: HttpClient) -> Failable<()> {
    http.create_message(msg.channel_id).content("pong!")?.await?;
    Ok(())
}

// Invia statistiche dell'anima
pub async fn stats(msg: &MessageCreate, redis: Arc<Mutex<Redis>>, http: HttpClient) -> Failable<()> {

    let anima: Anima = 
    {
        let mut redis = redis.lock().await;
        redis.get_anima(msg.author.id.0)
            .unwrap()
    };

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.add_row(row![    &msg.author.name]);
    table.add_row(row!["money", anima.money]);
    table.add_row(row!["level", anima.level]);
    table.add_row(row![  "exp",   anima.exp]);

    let result = formatting::code(&format!("{}", table));
    http.create_message(msg.channel_id).content(result)?.await?;
    Ok(())
}