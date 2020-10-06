
use std::error::Error;
use std::fmt;

use crate::anima::exp::Rank;

use serenity::{
    framework::standard::{
        CommandResult,
        macros::{
            command, 
            group
        },
    },
    model::prelude::*,
    prelude::*
};

pub mod misc;
pub mod music;

// ================================================================
// Lista di tutti i comandi utilizzabili
// ================================================================

#[derive(Debug, Clone, Copy)]
pub enum Commands
{
    Ping, Profile, Balance,                 // MISC
    Join, Leave, Play, Stop, Skip, Queue    // MUSIC
}

impl Commands {
    // Modifca all'affinità causata da ogni comando
    pub fn affinity_change(&self) -> i8 {
        match self {
            Commands::Play => 50,
            _ => 0
        }
    }

    // Da una stringa ottiene il comando associato
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "ping"    => Some(Commands::Ping),
            "profile" => Some(Commands::Profile),
            "balance" => Some(Commands::Balance),
            "join"    => Some(Commands::Join),
            "leave"   => Some(Commands::Leave),
            "play"    => Some(Commands::Play),
            "stop"    => Some(Commands::Stop),
            "skip"    => Some(Commands::Skip),
            "queue"   => Some(Commands::Queue),
            _         => None,
        }
    } 
}

// Devono essere minuscoli nel database
impl std::fmt::Display for Commands {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        let result = match self {
            Commands::Ping    => "ping",
            Commands::Profile => "profile",
            Commands::Balance => "balance",
            Commands::Join    => "join",
            Commands::Leave   => "leave",
            Commands::Play    => "play",
            Commands::Stop    => "stop",
            Commands::Skip    => "skip",
            Commands::Queue   => "queue",
        };
        write!(fmt, "{}", result)
     }
}

// ================================================================
// Risposte embedded classiche
// ================================================================

const LVL_UP_ICON:   &str = "https://www.iconfinder.com/data/icons/font-awesome/1792/level-up-512.png";
const LVL_DOWN_ICON: &str = "https://www.iconfinder.com/data/icons/font-awesome/1792/level-down-512.png";

// Visualizza aumento exp
pub async fn embed_increase_exp(ctx: &Context, msg: &Message, response: &str, exp: i32) {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.description(format!("{}\n```diff\n+{} exp\n```",response, exp));
            e
        })
    })
    .await.unwrap();
}

// Visualizza diminuzione exp
pub async fn embed_decrease_exp(ctx: &Context, msg: &Message, response: &str, exp: i32) {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.description(format!("{}\n```diff\n-{} exp\n```",response, exp));
            e
        })
    })
    .await.unwrap();
}

// Visualizza aumento livello
pub async fn embed_level_up(ctx: &Context, msg: &Message, response: &str, old: u32, new: u32) {
    let desc = format!("```diff\n+ Livello: {} → {}\n+ Grado: {}```", old, new, Rank::from_level(new));
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(format!("{} ha guadagnato un livello!", msg.author.name));
            e.description(format!("{}\n{}", response, desc));
            e.thumbnail(LVL_UP_ICON);
            e
        })
    })
    .await.unwrap();
}

// Visualizza diminuzione livello
pub async fn embed_level_down(ctx: &Context, msg: &Message, response: &str, old: u32, new: u32) {
    let desc = format!("```diff\n- Livello: {} → {}\n- Grado: {}```", old, new, Rank::from_level(new));
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(format!("{} ha perso un livello!", msg.author.name));
            e.description(format!("{}\n{}", response, desc));
            e.thumbnail(LVL_DOWN_ICON);
            e
        })
    })
    .await.unwrap();
}