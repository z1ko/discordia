
use std::future::Future;
use std::pin::Pin;

use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::MessageCreate;

pub mod misc;

// Permette di mantenere una funzione async
type AsyncFn<T> = 
    fn(msg: &MessageCreate) -> Pin<Box<dyn Future<Output = T> + Send>>;

pub struct Command {
    names: Vec<String>,
    func: AsyncFn<()>
}

// Contenitore di comandi
pub struct Commands {

}