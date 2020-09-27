
use crate::database::RedisKey;

#[derive(Debug, Clone)]
pub struct Anima 
{
    key: String,

    pub money: u32,
    pub level: u32,
    pub exp:   u32
}

impl Anima {
    pub fn new(key: &str, money: u32, level: u32, exp: u32) -> Self {
        Self {
            key: key.to_string(), 
            money, level, exp
        }
    }
}

impl RedisKey for Anima {
    fn key(&self) -> String { self.key.clone() }
}