use core::fmt;

use crate::sprite::Sprites;
use serde::{Deserialize, Serialize};

// Structs holding pokemon data

// Data
// Basic data that every object have (name + url)
#[derive(Serialize, Deserialize, Debug)]
struct Data {
    name: String,
    url: String,
}

// Abilities
#[derive(Deserialize, Debug)]
struct Ability {
    ability: Data,
    is_hidden: bool,
    slot: u8,
}

// Cries
#[derive(Deserialize, Debug)]
struct Cries {
    latest: String,
    legacy: String,
}

// Game Indice
#[derive(Deserialize, Debug)]
struct GameIndice {
    game_index: u32,
    version: Data,
}

// Item
#[derive(Deserialize, Debug)]
struct Item {
    item: Data,
    version_details: Vec<VersionDetails>,
}

#[derive(Deserialize, Debug)]
struct VersionDetails {
    rarity: u32,
    version: Data,
}

// Move
#[derive(Deserialize, Debug)]
struct Move {
    #[serde(rename = "move")]
    move_data: Data,
    version_group_details: Vec<VersionGroupDetails>,
}
#[derive(Deserialize, Debug)]
struct VersionGroupDetails {
    level_learned_at: u8,
    move_learn_method: Data,
    version_group: Data,
}

// Past Type
#[derive(Deserialize, Debug)]
struct PastType {
    generation: Data,
    types: PokemonType,
}

// Stat
#[derive(Serialize, Deserialize, Debug)]
pub struct Stat {
    base_stat: u32,
    effort: u32,
    stat: Data,
}

// Type
#[derive(Serialize, Deserialize, Debug)]
pub struct PokemonType {
    slot: u8,
    #[serde(rename = "type")]
    type_info: Data,
}

#[derive(Deserialize, Debug)]
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
        write!(f, "{}\nID : {}\n", self.name, self.id)?;

        write!(f, "- Type(s) : \n")?;
        for t in &self.types {
            write!(f, "{}\n", t.type_info.name)?;
        }

        write!(f, "- Stats : \n")?;
        for s in &self.stats {
            write!(f, "{} {}\n", s.stat.name, s.base_stat)?;
        }

        write!(f, "- Abilities : \n")?;
        for a in &self.abilities {
            if a.is_hidden {
                write!(f, "{} (hidden)\n", a.ability.name)?;
            } else {
                write!(f, "{}\n", a.ability.name)?;
            }
        }

        write!(f, "- Moves : \n")?;
        for m in &self.moves {
            write!(f, "{}\n", m.move_data.name)?;
        }

        Ok(())
    }
}

impl Into<String> for Pokemon {
    fn into(self) -> String {
        self.name
    }
}

impl Into<DbPoke> for Pokemon {
    fn into(self) -> DbPoke {
        DbPoke {
            id: self.id as i64,
            name: self.name,
            types: self.types,
            base_experience: self.base_experience as i64,
            stats: self.stats,
            is_shiny: false,
        }
    }
}

impl fmt::Display for DbPoke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\nID : {},\nbase exerperience: {}\n",
            self.name, self.id, self.base_experience
        )?;

        write!(f, "- Type(s) : \n")?;
        for t in &self.types {
            write!(f, "{}\n", t.type_info.name)?;
        }

        write!(f, "- Stats : \n")?;
        for s in &self.stats {
            write!(f, "{} {}\n", s.stat.name, s.base_stat)?;
        }

        write!(f, "- Shiny : {}\n", self.is_shiny)?;

        Ok(())
    }
}
