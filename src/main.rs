
mod database;

fn main() 
{
    dotenv::dotenv().ok();
    let redis_url = std::env::var("REDIS_DATABASE_URL")
        .expect("REDIS_DATABASE_URL not found in enviroment");

    print!("Connecting to Redis server at {} ... ", redis_url);
    let mut redis = database::Redis::connect(&redis_url)
        .expect("Error connecting to Redis server");
    println!("[OK]");

    let mut anima = redis.get_anima(59685490).unwrap();
    anima.level += 1;
    redis.set_anima(&anima).unwrap();

    println!("{:?}", anima);
}
