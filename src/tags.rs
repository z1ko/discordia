
use std::fmt;

use crate::affinity::Affinity;
use crate::commands::Commands;

// Possibili tag usabili nei gruppi delle risposte
#[derive(Debug, Clone, Copy)]
pub enum Tag
{
    // Indica che il gruppo viene usato per le risposte speciali
    // per quando l'utente usa un altro bot
    UsedOtherBot,

    // Per quando un utente sale o scende di livello
    UserLevelUp,
    UserLevelDown,

    // Per quando un utente prende o perde exp
    UserExpUp,
    UserExpDown,

    // Per quando il comando non viene eseguito
    NoExec,

    // Indica che il gruppo va utilizzato su almeno questo 
    // livello di affinità, possono essercene più di uno per gruppo
    Affinity(Affinity),

    // Indica che il gruppo va utilizzato su questo comando
    // può essercene uno solo per gruppo
    Command(Commands),

    // Indica che il gruppo va utilizzato per uno specifico utente
    // ma se non è presente non fallisce
    Anima(u64),
}

impl Tag {
    // Converte la tag in stringa per poter cercare nel database
    pub fn string(&self) -> String {
        match self {
            Tag::UsedOtherBot  => format!("used-other-bot"),
            Tag::UserLevelUp   => format!("user-level-up"),
            Tag::UserLevelDown => format!("user-level-down"),
            Tag::UserExpUp     => format!("user-exp-up"),
            Tag::UserExpDown   => format!("user-exp-down"),
            Tag::Affinity(aff) => format!("affinity:{}", aff),
            Tag::Command(cmd)  => format!("cmd:{}", cmd),
            Tag::Anima(id)     => format!("anima:{}", id),
            Tag::NoExec        => format!("no-exec"),
        }
    }

    // Indica se la tag è ripetibile
    pub fn repetable(&self) -> bool {
        match self {
            Tag::UsedOtherBot  => false, 
            Tag::UserLevelUp   => false,
            Tag::UserLevelDown => false,
            Tag::UserExpUp     => false,
            Tag::UserExpDown   => false,
            Tag::Affinity(_)   => true,
            Tag::Command(_)    => false,
            Tag::Anima(_)      => true,
            Tag::NoExec        => false, 
        }
    }

    // Indica se è opzionale
    pub fn optional(&self) -> bool {
        match self {
            Tag::UsedOtherBot  => false, 
            Tag::UserLevelUp   => false,
            Tag::UserLevelDown => false,
            Tag::UserExpUp     => false,
            Tag::UserExpDown   => false,
            Tag::Affinity(_)   => false,
            Tag::Command(_)    => false,
            Tag::Anima(_)      => true,
            Tag::NoExec        => false,
        }
    }
}

// Indica se la stringa corrisponde alla tag
impl std::cmp::PartialEq<String> for Tag {
    fn eq(&self, other: &String) -> bool {
        self.string() == *other
     }
}

// Un filtro delle tag, facilita il filtraggio
#[derive(Debug)]
pub struct Filter 
{
    tags: Vec<Tag>
}

// Risultato del filtro
#[derive(Debug, PartialEq)]
pub enum FilterResult 
{
    Passed(u32),  // Le tag sono valide per questo filtro
    Blocked       // Le tag non sono compatibili  
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.tags)
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            tags: vec![]
        }
    }

    // Aggiunge una tag al filtro
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    // Analizza un'insieme di stringhe tag
    pub fn check(&self, strings: &[String]) -> FilterResult 
    {
        let mut score: u32 = 0;
        for tag in &self.tags 
        {
            let contains = strings.contains(&tag.string());
            let optional = tag.optional();

            // Se non è opzionale e manca allora non può passare il filtro
            if !optional && !contains { return FilterResult::Blocked;}

            // Se è opzionale e non presente allora non aumenta lo score
            if optional && !contains { 
                println!("Optional value not found: {}", tag.string());
                score = score.saturating_sub(1);
                continue;
            }

            // Tutti gli altri casi aumenta il punteggio
            score += 2;
        }

        FilterResult::Passed(score)
    }
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn filter()
    {
        let filter1 = Filter::new()
            .tag(Tag::Command(Commands::Ping))
            .tag(Tag::Anima(59685490));

        let filter2 = Filter::new()
            .tag(Tag::Command(Commands::Ping));

        let tags1 = ["cmd:ping".to_string(), "anima:59685490".to_string()];
        let tags2 = ["cmd:ping".to_string()];

        assert_eq!(filter1.check(&tags1), FilterResult::Passed(2));
        assert_eq!(filter2.check(&tags1), FilterResult::Passed(1));
        assert_eq!(filter1.check(&tags2), FilterResult::Passed(1));
    }  
}
