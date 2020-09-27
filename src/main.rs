
mod redis;
mod anima;
mod tags;

use crate::{
    redis::Redis,
    anima::Anima,
};

fn main() 
{
    dotenv::dotenv().ok();
    let redis_url = std::env::var("REDIS_DATABASE_URL")
        .expect("REDIS_DATABASE_URL not found in enviroment");

    print!("Connecting to Redis server at {} ... ", redis_url);
    let mut redis = Redis::connect(&redis_url)
        .expect("Error connecting to Redis server");
    println!("[OK]");

    let mut anima = redis.get_anima(59685490).unwrap();
    anima.level += 1;
    redis.set_anima(59685490, &anima).unwrap();

    // Ottiene tutti i gruppi di risposte
    let groups = redis.get_groups().unwrap();
    println!("{:?}", groups);

    let tags = redis.get_group_tags(&groups[1]).unwrap();
    println!("{:?}", tags);

    println!("{:?}", anima);
}
