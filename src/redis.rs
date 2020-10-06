/**
 * Questo modulo permette di modificare e creare strutture sul server redis,
 * le connessioni sono singlethread per ora...
 */


use redis::{
    Commands, Client, RedisResult, Connection
};

use rand::prelude::*;
use rand::prelude::SliceRandom;

use crate::{
    anima::Anima,
    tags::{
        Tag, Filter, FilterResult,
    }
};

// Rappresenta il server e una fabbrica di DAO
pub struct Redis 
{
    client: Client,
    con: Connection
}

impl Redis {
    // Connette al server
    pub fn connect(url: &str) -> RedisResult<Redis> 
    {
        let client = Client::open(url)?;
        let con = client.get_connection()?;

        Ok(Self { client, con })
    }

    // Ottiene un'anima dal suo id e se necessario la crea
    pub fn get_anima(&mut self, id: u64) -> RedisResult<Anima> {
        let key = format!("anima:{}", id);

        // Se non esiste la crea nuova
        if !self.con.exists(&key)? 
        {
            self.con.hset(&key, "money",            0)?;
            self.con.hset(&key, "affinity_score", 127)?;
            self.con.hset(&key, "level",            1)?;
            self.con.hset(&key, "exp",              0)?;
        }

        let money = self.con.hget(&key, "money")?;
        let score = self.con.hget(&key, "affinity_score")?;
        let level = self.con.hget(&key, "level")?;
        let exp   = self.con.hget(&key, "exp")?;

        Ok(Anima::new(money, level, exp, score))
    }

    // Salva o aggiorna la nuova anima nel database
    pub fn set_anima(&mut self, id: u64, anima: &Anima) -> RedisResult<()> {
        let key = format!("anima:{}", id);

        self.con.hset(&key, "money",          anima.money)?;
        self.con.hset(&key, "affinity_score", anima.affinity_score)?;
        self.con.hset(&key, "level",          anima.level)?;
        self.con.hset(&key, "exp",            anima.exp)?;

        Ok(())
    }

    // Ottiene le risposte conosciute
    fn get_groups(&mut self) -> RedisResult<Vec<String>> {
        Ok(self.con.smembers("responses")?)
    }

    // Ottiene le tag di un gruppo
    fn get_group_tags(&mut self, group: &String) -> RedisResult<Vec<String>> {    
        Ok(self.con.smembers(format!("{}/tags", group))?)
    }

    // Ottiene le risposte di un gruppo
    fn get_group_data(&mut self, group: &String) -> RedisResult<Vec<String>> {
        Ok(self.con.smembers(format!("{}/data", group))?)
    }

    // Ricerca il database per ottenere una risposta adeguata al filtro
    pub fn generate_response(&mut self, filter: Filter) -> RedisResult<Option<String>> {
        
        let mut winners: Vec<(String, i32)> = Vec::default();
        let mut winner_score = 1;
        
        // Ottiene le tag di tutti i gruppi e salva quelli che hanno successo
        for group in &self.get_groups()? 
        {
            let tags = self.get_group_tags(group)?;
            if let FilterResult::Passed(score) = filter.check(&tags) {
                if score >= winner_score 
                {
                    winners.push((group.clone(), score)); 
                    winner_score = score; 
                }
            }
        }

        // Ottiene i gruppi che hanno lo score più alto
        let winners = winners.into_iter().filter(|(_, score)| *score == winner_score)
            .map(|(group, _)| group).collect::<Vec<String>>();

        // Ne sceglie uno a caso e ottiene anche una risposta a caso
        if let Some(group) = winners.choose(&mut rand::thread_rng()) {
            if let Some(result) = self.get_group_data(&group)?.choose(&mut rand::thread_rng()) {
                return Ok(Some(result.clone()));
            }
        }

        Ok(None)
    }
}

use serenity::prelude::*;
use std::sync::Arc;

// Permette l'inserimento nei dati di serenity
pub struct RedisMapKey;
impl TypeMapKey for RedisMapKey { 
    type Value = Arc<Mutex<Redis>>;
}