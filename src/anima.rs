
#[derive(Debug, Clone)]
pub struct Anima 
{
    pub money: u32,
    pub level: u32,
    pub exp:   u32
}

impl Anima {
    pub fn new(money: u32, level: u32, exp: u32) -> Self {
        Self { 
            money, level, exp
        }
    }
}