
// Tutti i comandi possibili
#[derive(Debug, Clone)]
pub enum Commands
{
    Ping,

    Play,
    Loop,
    Stop,
    Pause,
    Unpause,
}

// Devono essere minuscoli nel database
impl std::fmt::Display for Commands {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        let result = match self {
            Commands::Ping    => "ping",
            Commands::Play    => "play",
            Commands::Loop    => "loop",
            Commands::Stop    => "stop",
            Commands::Pause   => "pause",
            Commands::Unpause => "unpause",
        };
        write!(fmt, "{}", result)
     }
}

// Tutti le possibili Affinità
#[derive(Debug, Clone)]
pub enum Affinities
{
    Hate,
    Annoyance,
    Neutral,
    Friend,
    Love
}

// Devono essere minuscoli nel database
impl std::fmt::Display for Affinities {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        let result = match self {
            Affinities::Hate       => "hate",
            Affinities::Annoyance  => "annoyance",
            Affinities::Neutral    => "neutral",
            Affinities::Friend     => "friend",
            Affinities::Love       => "love",
        };
        write!(fmt, "{}", result)
     }
}


// Possibili tag usabili nei gruppi delle risposte
#[derive(Debug, Clone)]
pub enum Tag
{
    // Indica che il gruppo viene usato per le risposte speciali
    // per quando l'utente usa un altro bot
    UsedOtherBot,

    // Indica che il gruppo va utilizzato su almeno questo 
    // livello di affinità, possono essercene più di uno per gruppo
    Affinity(Affinities),

    // Indica che il gruppo va utilizzato su questo comando
    // può essercene uno solo per gruppo
    Command(Commands),

    // Indica che il gruppo va utilizzato per uno specifico utente
    // ma se non è presente non fallisce
    Anima(u32),
}

impl Tag {
    // Converte la tag in stringa per poter cercare nel database
    pub fn string(&self) -> String {
        match self {
            Tag::UsedOtherBot  => format!("used_other_bot"),
            Tag::Affinity(aff) => format!("affinity:{}", aff),
            Tag::Command(cmd)  => format!("cmd:{}", cmd),
            Tag::Anima(id)     => format!("anima:{}", id),
        }
    }

    // Indica se la tag è ripetibile
    pub fn repetable(&self) -> bool {
        match self {
            Tag::UsedOtherBot => false, 
            Tag::Affinity(_)  => true,
            Tag::Command(_)   => false,
            Tag::Anima(_)     => true,
        }
    }

    // Indica se è opzionale
    pub fn optional(&self) -> bool {
        match self {
            Tag::UsedOtherBot => false, 
            Tag::Affinity(_)  => false,
            Tag::Command(_)   => false,
            Tag::Anima(_)     => true,
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
    Passed(i32),  // Le tag sono valide per questo filtro
    Blocked       // Le tag non sono compatibili  
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
        let mut score: i32 = 0;
        for tag in &self.tags 
        {
            let contains = strings.contains(&tag.string());
            let optional = tag.optional();

            // Se non è opzionale e manca allora non può passare il filtro
            if !optional && !contains { return FilterResult::Blocked;}

            // Se è opzionale e non presente allora non aumenta lo score
            if optional && !contains { 
                println!("Optional value not found: {}", tag.string());
                score -= 1;
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