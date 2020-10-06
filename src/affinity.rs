
use std::fmt;

#[derive(Debug, Clone, Copy)]
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