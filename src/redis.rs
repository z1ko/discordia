/**
 * Questo modulo permette di modificare e creare strutture sul server redis,
 * le connessioni sono singlethread per ora...
 */

use crate::Anima;
use redis::{
    Commands, Connection, Client, RedisResult
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

        Ok(Self { 
            client, 
            con
        })
    }

    // Ottiene un'anima dal suo id e se necessario la crea
    pub fn get_anima(&mut self, id: u64) -> RedisResult<Anima> {
        let key = format!("anima:{}", id);

        // Se non esiste la crea nuova
        if !self.con.exists(&key)? 
        {
            self.con.hset(&key, "money", 0)?;
            self.con.hset(&key, "level", 1)?;
            self.con.hset(&key,   "exp", 0)?;
        }

        let money = self.con.hget(&key, "money")?;
        let level = self.con.hget(&key, "level")?;
        let exp   = self.con.hget(&key,   "exp")?;

        Ok(Anima::new(money, level, exp))
    }

    // Salva o aggiorna la nuova anima nel database
    pub fn set_anima(&mut self, id: u64, anima: &Anima) -> RedisResult<()> {
        let key = format!("anima:{}", id);

        self.con.hset(&key, "money", anima.money)?;
        self.con.hset(&key, "level", anima.level)?;
        self.con.hset(&key,   "exp",   anima.exp)?;

        Ok(())
    }

    // Ottiene le risposte conosciute
    pub fn get_groups(&mut self) -> RedisResult<Vec<String>> {
        Ok(self.con.smembers("responses")?)
    }

    // Ottiene le tag di un gruppo
    pub fn get_group_tags(&mut self, group: &String) -> RedisResult<Vec<String>> {    
        Ok(self.con.smembers(format!("{}/tags", group))?)
    }

    // Ottiene le risposte di un gruppo
    pub fn get_group_data(&mut self, group: &String) -> RedisResult<Vec<String>> {
        Ok(self.con.smembers(format!("{}/data", group))?)
    }
}