use core::fmt;

use crate::sprite::Sprites;
use serde::{Deserialize, Serialize};

// Structs holding pokemon data

// Data
// Basic data that every object have (name + url)
#[derive(Serialize, Deserialize)]
struct Data {
    name: String,
    url: String,
}

// Abilities
#[derive(Deserialize)]
struct Ability {
    ability: Data,
    is_hidden: bool,
    slot: u8,
}

// Cries
#[derive(Deserialize)]
struct Cries {
    latest: String,
    legacy: String,
}

// Game Indice
#[derive(Deserialize)]
struct GameIndice {
    game_index: u32,
    version: Data,
}

// Item
#[derive(Deserialize)]
struct Item {
    item: Data,
    version_details: Vec<VersionDetails>,
}

#[derive(Deserialize)]
struct VersionDetails {
    rarity: u32,
    version: Data,
}

// Move
#[derive(Deserialize)]
struct Move {
    #[serde(rename = "move")]
    move_data: Data,
    version_group_details: Vec<VersionGroupDetails>,
}
#[derive(Deserialize)]
struct VersionGroupDetails {
    level_learned_at: u8,
    move_learn_method: Data,
    version_group: Data,
}

// Past Type
#[derive(Deserialize)]
struct PastType {
    generation: Data,
    types: PokemonType,
}

// Stat
#[derive(Serialize, Deserialize)]
pub struct Stat {
    base_stat: u32,
    effort: u32,
    stat: Data,
}

// Type
#[derive(Serialize, Deserialize)]
pub struct PokemonType {
    slot: u8,
    #[serde(rename = "type")]
    type_info: Data,
}

#[derive(Deserialize)]
pub struct Pokemon {
    abilities: Vec<Ability>,
    base_experience: u32,
    cries: Cries,
    forms: Vec<Data>,
    game_indices: Vec<GameIndice>,
    height: u32,
    held_items: Option<Vec<Item>>,
    id: u32,
    is_default: bool,
    location_area_encounters: String,
    moves: Vec<Move>,
    name: String,
    order: u32,
    past_abilities: Option<Vec<Data>>, // Need to find an example (placeholder for the moment)
    past_types: Option<Vec<PastType>>,
    species: Data,
    sprites: Sprites,
    stats: Vec<Stat>,
    types: Vec<PokemonType>,
    weight: u32,
}

pub struct DbPoke {
    pub id: i64,
    pub name: String,
    pub types: Vec<PokemonType>,
    pub base_experience: i64,
    pub stats: Vec<Stat>,
    pub is_shiny: bool,
}

impl fmt::Display for Pokemon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\nID : {}\n", self.name, self.id)?;

        writeln!(f, "- Type(s) : ")?;
        for t in &self.types {
            writeln!(f, "{}", t.type_info.name)?;
        }

        writeln!(f, "- Stats : ")?;
        for s in &self.stats {
            writeln!(f, "{} {}", s.stat.name, s.base_stat)?;
        }

        writeln!(f, "- Abilities : ")?;
        for a in &self.abilities {
            if a.is_hidden {
                writeln!(f, "{} (hidden)", a.ability.name)?;
            } else {
                writeln!(f, "{}", a.ability.name)?;
            }
        }

        writeln!(f, "- Moves : ")?;
        for m in &self.moves {
            writeln!(f, "{}", m.move_data.name)?;
        }

        Ok(())
    }
}

impl From<Pokemon> for String {
    fn from(val: Pokemon) -> Self {
        val.name
    }
}

impl From<Pokemon> for DbPoke {
    fn from(val: Pokemon) -> Self {
        DbPoke {
            id: val.id as i64,
            name: val.name,
            types: val.types,
            base_experience: val.base_experience as i64,
            stats: val.stats,
            is_shiny: false,
        }
    }
}

impl fmt::Display for DbPoke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}\nID : {},\nbase exerperience: {}",
            self.name, self.id, self.base_experience
        )?;

        writeln!(f, "- Type(s) : ")?;
        for t in &self.types {
            writeln!(f, "{}", t.type_info.name)?;
        }

        writeln!(f, "- Stats : ")?;
        for s in &self.stats {
            writeln!(f, "{} {}", s.stat.name, s.base_stat)?;
        }

        writeln!(f, "- Shiny : {}", self.is_shiny)?;

        Ok(())
    }
}
