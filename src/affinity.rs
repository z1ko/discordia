
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Affinity
{
    Hate,
    Annoyed,
    Neutral,
    Friend,
    Love
}

impl fmt::Display for Affinity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        let result = match self {
            Affinity::Hate    => "hate",
            Affinity::Annoyed => "annoyed",
            Affinity::Neutral => "neutral",
            Affinity::Friend  => "friend",
            Affinity::Love    => "love",
        };
        write!(f, "{}", result)
    }
}

impl Affinity {
    pub fn from_score(score: u8) -> Self {
        match score {
              0 ..=  15 => Affinity::Hate,
             16 ..=  50 => Affinity::Annoyed,
             51 ..= 150 => Affinity::Neutral,
            151 ..= 240 => Affinity::Friend,
            241 ..= 255 => Affinity::Love,
        }
    }
}

// Rappresenta un cambiamento di livello di affinità
pub enum AffinityChange 
{
    Some(Affinity, Affinity),
    None,
}

pub enum AffinityLevel {
    Hate, Annoyed,
}

// Semplifica operazioni sull'affinità
pub struct AffinityScore {
    level: AffinityLevel, 
    score: u8 
}

impl AffinityScore 
{
    // Danneggià affinità e potenzialmente ritorna cambiamento in livello affinità
    pub fn sub(&mut self, value: u8) -> Option<(Affinity, Affinity)> 
    {
        let old = Affinity::from_score(self.score);
        self.score = self.score.saturating_sub(value);
        let new = Affinity::from_score(self.score);

        if old != new { Some((old, new)) }
                 else { None }
    }

    // Ripristina affinità e potenzialmente ritorna cambiamento in livello affinità
    pub fn add(&mut self, value: u8) -> Option<(Affinity, Affinity)> 
    {
        let old = Affinity::from_score(self.score);
        self.score = self.score.saturating_add(value);
        let new = Affinity::from_score(self.score);

        if old != new { Some((old, new)) }
                 else { None }
    }
}