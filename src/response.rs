
use rand::prelude::*;
use rand::prelude::SliceRandom;

use redis::RedisResult;

use crate::{
    redis::Redis,
    anima::Anima,
    tags::{
        Tag, Affinities, Commands,
        Filter, FilterResult,
    }
};

// Ricerca il database per ottenere una risposta adeguata al filtro
pub fn generate_response(redis: &mut Redis, filter: Filter) -> RedisResult<Option<String>> {
    
    let mut winners: Vec<(String, i32)> = Vec::default();
    let mut winner_score = 1;
    
    // Ottiene le tag di tutti i gruppi e salva quelli che hanno successo
    for group in &redis.get_groups()? 
    {
        let tags = redis.get_group_tags(group)?;
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
        if let Some(result) = redis.get_group_data(&group)?.choose(&mut rand::thread_rng()) {
            return Ok(Some(result.clone()));
        }
    }

    Ok(None)
}