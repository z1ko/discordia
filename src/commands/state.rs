
use crate::ReqwestClient;
use crate::HttpClient;
use crate::Redis;

use tokio::sync::Mutex;
use std::sync::Arc;

use twilight_gateway::cluster::Cluster;

// Contiene tutte le strutture utili ad un comando
#[derive(Clone)] 
pub struct CmdState
{
    // Interfaccia di redis
    pub redis: Arc<Mutex<Redis>>,

    // Interfaccia al cluster di twilight
    pub cluster: Cluster,

    // Interfaccia al sistema di richieste HTTP
    pub http: HttpClient,

    // Interfaccia al client reqwest
    pub reqwest: ReqwestClient,

    // Interfaccia di Lavalink
    // TODO
}