/**
 * Utilities per inviare i messaggi embeded standard 
 */

use twilight_model::channel::embed::Embed;
use twilight_embed_builder::ImageSource;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};

use prettytable::{
    format, Table
};

use crate::Anima;
use crate::anima::exp::Rank;
use crate::Failable;

mod formatting
{
    // ES: +50 exp, text: "50 exp"
    pub fn positive(text: &str) -> String {
        format!("```diff\n+ {}\n```", text)
    }

    // ES: -50 exp, text: "50 exp"
    pub fn negative(text: &str) -> String {
        format!("```diff\n- {}\n```", text)
    }

    // Incapsula in codice
    pub fn code(text: &str) -> String {
        format!("```\n{}\n```", text)
    }
}

// Crea embeded per il level up di un'anima
pub fn level_up(username: &str, message: &str, old_level: u32, new_level: u32) -> Failable<Embed>
{
    let descr = format!("```diff\n+ Livello: {} → {}\n+ Grado: {}```",
        old_level, new_level, Rank::from_level(new_level));

    let embed = EmbedBuilder::new()
        .title(format!("{} ha guadagnato un livello!", username))?
        .description(format!("{}\n{}", message, descr))?
        .thumbnail(ImageSource::url("https://www.iconfinder.com/data/icons/font-awesome/1792/level-up-512.png")?)
        .build()?;

    Ok(embed)
}

// Crea embeded per il level down di un'anima
pub fn level_down(username: &str, message: &str, old_level: u32, new_level: u32) -> Failable<Embed>
{
    let descr = format!("```diff\n- Livello: {} → {}\n- Grado: {}```",
        old_level, new_level, Rank::from_level(new_level));

    let embed = EmbedBuilder::new()
        .title(format!("{} ha perso un livello!", username))?
        .description(format!("{}\n{}", message, descr))?
        .thumbnail(ImageSource::url("https://www.iconfinder.com/data/icons/font-awesome/1792/level-down-512.png")?)
        .build()?;

    Ok(embed)
}

// Crea embeded per la perdità di exp di un'anima
pub fn decrease_exp(response: &str, exp: i32) -> Failable<Embed>
{
    let desc = formatting::negative(&format!("{} exp", exp));
    let embed = EmbedBuilder::new()
        .description(format!("{}\n{}", response, desc))?
        .build()?;

    Ok(embed)
}

// Crea embeded per l'aumento di exp di un'anima
pub fn increase_exp(response: &str, exp: i32) -> Failable<Embed>
{
    let desc = formatting::positive(&format!("{} exp", exp));
    let embed = EmbedBuilder::new()
        .description(format!("{}\n{}", response, desc))?
        .build()?;

    Ok(embed)
}

// Crea embed per le statistiche del personaggio
pub fn stats(username: &str, anima: &Anima) -> Failable<Embed>
{
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.add_row(row!["money", anima.money]);
    table.add_row(row!["level", anima.level]);
    table.add_row(row!["  exp",   anima.exp]);

    let embed = EmbedBuilder::new()
        .title(username)?
        .description(format!("{}", table))?
        .build()?;

    Ok(embed)
}