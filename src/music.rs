
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::voice::LockedAudio;
use serenity::prelude::*;

use std::collections::VecDeque;
use std::sync::Arc;

//
// Gestisce la musica e la coda delle canzoni
//

#[derive(Clone)]
pub struct Orchestra {
    // Coda delle canzoni pronte
    pub queue: Arc<Mutex<VecDeque<LockedAudio>>>,
}

impl Orchestra {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    // Aggiunge musica alla coda mutandola
    pub async fn add(&mut self, audio: LockedAudio) {
        audio.lock().await.pause();
        
        let mut queue = self.queue.lock().await; 
        queue.push_back(audio.clone());
    }

    // Resetta coda
    pub async fn reset(&mut self) {
        let mut queue = self.queue.lock().await;
        queue.clear();
    }

    // Salta il pezzo corrente nella coda
    pub async fn skip(&mut self) {
        let mut queue = self.queue.lock().await;
        if queue.len() != 0
        {
            let audio = queue.front_mut().unwrap();
            let mut audio = audio.lock().await;
            audio.pause(); // TODO Trova un modo per rimuoverla veramente
        }

        queue.pop_front();
    }

    // Aggiorna lista e prova a riprodurre una nuova musica nella coda
    pub async fn update(&mut self) {
        
        // Prova ad acquisire la coda musicale salta altrimenti
        if let Ok(mut queue) = self.queue.try_lock() {

            let mut pop = false;
            if let Some(audio) = queue.front_mut() 
            {
                // Se il primo elemento è finito lo toglie
                let mut audio = audio.lock().await;
                if audio.finished { 
                    pop = true; 
                } 
            }

            // Elimina elemento frontale se finito
            if pop { queue.pop_front(); }

            // Riproduce l'elemento frontale
            if let Some(audio) = queue.front_mut() {
                let mut audio = audio.lock().await;
                if !audio.playing { 
                    audio.play(); 
                }
            }
        }
    }
}

// Permette l'inserimento nelle risorse di serenity
pub struct OrchestraMapKey;
impl TypeMapKey for OrchestraMapKey {
    type Value = Orchestra;
}

