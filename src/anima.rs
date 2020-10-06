
use crate::affinity::{Affinity, AffinityChange};

#[derive(Debug, Clone)]
pub struct Anima 
{
    pub money: u32,
    pub affinity_score: u8,
}

impl Anima {
    pub fn new(money: u32, affinity_score: u8) -> Self {
        Self {
            money, affinity_score,
        }
    }

    pub fn affinity_sub(&mut self, value: u8) -> AffinityChange 
    {
        let old = Affinity::from_score(self.affinity_score);
        self.affinity_score = self.affinity_score.saturating_sub(value);
        let new = Affinity::from_score(self.affinity_score);

        if old != new { AffinityChange::Some(old, new) } 
                 else { AffinityChange::None }
    }

    pub fn affinity_add(&mut self, value: u8) -> AffinityChange 
    {
        let old = Affinity::from_score(self.affinity_score);
        self.affinity_score = self.affinity_score.saturating_add(value);
        let new = Affinity::from_score(self.affinity_score);
    
        if old != new { AffinityChange::Some(old, new) } 
                 else { AffinityChange::None }
    }
}

// Tutto legato al levelling 
pub mod exp
{
    // L'aumento di exp può causare un aumento di libello
    pub enum LevelChange 
    {
        Delta(u32, u32), // old, new
        None
    }

    // Oggetti che possono contenere esperienza
    pub trait Levelling
    {
        fn increase_exp(&mut self, delta: i32) -> LevelChange;
        fn decrease_exp(&mut self, delta: i32) -> LevelChange;

        // Esperienza per il prossimo livello
        fn experience(&self) -> i32;
    }

    // Grado per identificare in modo diverso i livelli
    pub enum Rank
    {
        E,
        D,
        C,
        B,
        A,
        S,
        SS,
        SSS,
    }

    impl Rank {
        pub fn from_level(level: u32) -> Rank {
            match level {
                0  ..=  5 => Rank::E,
                6  ..= 10 => Rank::D,
                11 ..= 15 => Rank::C,
                16 ..= 20 => Rank::B,
                21 ..= 25 => Rank::A,
                26 ..= 30 => Rank::S,
                31 ..= 35 => Rank::SS,
                        _ => Rank::SSS,
            }
        }
    }

    // Per poterlo usare negli embed
    impl std::fmt::Display for Rank {
        fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
            match self {
                Rank::E   => write!(fmt, "E"),
                Rank::D   => write!(fmt, "D"),
                Rank::C   => write!(fmt, "C"),
                Rank::B   => write!(fmt, "B"),
                Rank::A   => write!(fmt, "A"),
                Rank::S   => write!(fmt, "S"),
                Rank::SS  => write!(fmt, "SS"),
                Rank::SSS => write!(fmt, "SSS"),
            }
        }
    }

    // Fattore di crescita
    const K: f32 = 1.5;

    // Quantità di esperienza richiesta per il primo livello
    const I: f32 = 200.0;

    // Esperienza necessaria per raggiungere un livello
    // E(x + 1) = E(x) + K * E(x)
    pub fn experience(level: u32) -> i32
    {
        let a = level as f32;
        let b = I * (1.0 - K.powf(a)) / (1.0 - K);
       
        b as i32
    }

    // Livello corrispondente all'esperienza presente
    pub fn level(exp: i32) -> u32
    {
        let a = exp as f32;
        let b = 1.0 / K.ln();
        let c = ((I + (K - 1.0) * a) / I).ln();
        
        (b * c) as u32
    }
}

/*
// L'anima ovviamente può salire di livello
impl exp::Levelling for Anima
{
    fn increase_exp(&mut self, delta: i32) -> exp::LevelChange
    {
        self.exp += delta;
        let prev_level = self.level;
        self.level = exp::level(self.exp);
        
        // Aumento di livello!
        if prev_level != self.level 
        {
            exp::LevelChange::Delta(prev_level, self.level)
        }
        else 
        {
            exp::LevelChange::None
        }
    }

    fn decrease_exp(&mut self, delta: i32) -> exp::LevelChange
    {
        self.exp = std::cmp::max(0, self.exp - delta);
        let prev_level = self.level;
        self.level = exp::level(self.exp);

        // Diminuzione di livello!
        if prev_level != self.level 
        {
            exp::LevelChange::Delta(prev_level, self.level)
        }
        else 
        {
            exp::LevelChange::None
        }
    }

    // Esperienza per il prossimo livello
    fn experience(&self) -> i32 { exp::experience(self.level) }
}
*/