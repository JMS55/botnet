pub mod api;
mod entity_helpers;
pub mod pathfinding;

use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "default")]
pub use api::log_debug;
#[cfg(feature = "default")]
pub use bincode;
#[cfg(feature = "default")]
pub use botnet_api_derive::bot;
#[cfg(feature = "default")]
pub use rkyv;

pub const BAY_SIZE: usize = 24;

#[derive(Archive, Serialize, Deserialize, Clone)]
pub struct Bay {
    pub entities: HashMap<EntityID, (Entity, u32, u32)>,
    pub cells: [[Option<EntityID>; BAY_SIZE]; BAY_SIZE],
}

pub type EntityID = u64;

#[derive(Archive, Serialize, Deserialize, Clone)]
pub enum Entity {
    Bot(Bot),
    Antenna(Antenna),
    Interconnect { next_bay_id: EntityID },
    Resource(Resource),
}

#[derive(Archive, Serialize, Deserialize, Clone)]
pub struct Bot {
    pub id: EntityID,
    pub controller_id: EntityID,
    pub energy: u32,
    pub held_resource: Option<Resource>,
    pub x: usize,
    pub y: usize,
}

#[derive(Archive, Serialize, Deserialize, Clone)]
pub struct Antenna {
    pub controller_id: EntityID,
    pub stored_copper: u8,
    pub stored_gold: u8,
    pub stored_silicon: u8,
    pub stored_plastic: u8,
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug))]
pub enum Resource {
    Copper,
    Gold,
    Silicon,
    Plastic,
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug)]
#[archive_attr(derive(Debug))]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum ActionError {
    ActionNotPossible,
    NotEnoughEnergy,
    AlreadyActed,
}
