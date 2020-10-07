
use std::sync::Arc;
use serenity::prelude::Mutex;
use std::collections::VecDeque;

use serenity::prelude::TypeMapKey;
use serenity::model::prelude::{GuildId, ChannelId};
use serenity::voice::{LockedAudio, AudioSource, Handler};
use serenity::client::bridge::voice::ClientVoiceManager;

//
// Informazioni sulla track
//

#[derive(Clone)]
pub struct Track {
    audio: LockedAudio,
    //url:   String,
}

//
// Gestore della musica
// Struttura clonabile senza problemi
//

#[derive(Clone)]
pub struct Orchestra {
    manager: Arc<Mutex<ClientVoiceManager>>, // Gestore delle connesioni audio
    queue:   Arc<Mutex<VecDeque<Track>>>,    // Coda delle canzoni
    looping: bool,                           // Indica se la canzone attuale è in loop
}

impl Orchestra {

    // Ottiene controllo del manager di serenity
    pub fn new(manager: Arc<Mutex<ClientVoiceManager>>) -> Self {
        Self {
            manager, looping: false,
            queue: Arc::new(Mutex::new(VecDeque::default()))
        }
    }

    // Aggiorna coda musicale
    pub async fn update(&mut self) {
        let mut queue = self.queue.lock().await;

        // Controlla se il primo è finito 
        let finished = match queue.front() {
            Some(track) => {
                let audio = track.audio.lock().await;
                audio.finished
            }
            None => false
        };

        // Rimuove il promo se finito
        if finished { queue.pop_front(); }

        // Controlla se il primo sta riproducendo e 
        // nel caso non lo sia lo fa partire
        match queue.front_mut() {
            Some(track) => {
                let mut audio = track.audio.lock().await;
                if !audio.playing { audio.play(); }
            }
            _ => { }
        };
    }

    // Aggiunge alla coda
    pub async fn insert(&mut self, guild: GuildId, source: Box<dyn AudioSource>) -> bool 
    {
        let mut manager = self.manager.lock().await;
        if let Some(player) = manager.get_mut(guild) {

            let lock = player.play_returning(source);
            let mut audio = lock.lock().await;
            audio.pause();

            self.queue.lock().await
                .push_front(Track{ audio: lock.clone() });

            return true;
        }
        return false;
    }

    // Salta il brano corrente
    pub async fn skip(&mut self, guild: GuildId) {
        let mut queue = self.queue.lock().await;

        // Controlla se il primo brano esiste e in tal caso
        // lo rimuove dal player
        let remove = match queue.front() {
            Some(_) => {
                let mut manager = self.manager.lock().await;
                if let Some(player) = manager.get_mut(guild) {
                    player.stop();
                }
                true
            }
            None => false,
        };

        // Rimuove primo elemento e stoppa la sua riproduzione
        if remove { queue.pop_front(); }
    }

    // Resetta coda rimuovendo tutto
    pub async fn clear(&mut self) {
        self.queue.lock().await
            .clear();
    }

    // Si collega alla gilda e al canale specificato
    pub async fn join(&mut self, guild: GuildId, channel: ChannelId) -> bool {
        self.clear().await;
        if let Some(_) = self.manager.lock().await.join(guild, channel) {
            return true;
        }
        false
    }

    // Lascia gilda specificata
    pub async fn leave(&mut self, guild: GuildId) -> bool {
        self.clear().await;
        self.manager.lock().await
            .leave(guild)
            .is_some()
    }
}

//
// Permette l'inserimento nelle risorse di serenity
//

pub struct OrchestraMapKey;
impl TypeMapKey for OrchestraMapKey {
    type Value = Orchestra;
}