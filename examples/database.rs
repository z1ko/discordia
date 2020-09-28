
mod redis;
mod anima;
mod response;
mod tags;

use crate::{
    redis::Redis,
    anima::Anima,
    response::generate_response,
    tags::{
        Tag, Filter, Commands
    }
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
    println!("{:?}", anima);

    let filter = tags::Filter::new()
        .tag(Tag::Command(Commands::Ping))
        .tag(Tag::Anima(15));

    if let Some(response) = generate_response(&mut redis, filter).unwrap() {
        println!("{}", response);
    }   
}
