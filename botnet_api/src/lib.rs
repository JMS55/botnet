mod api;
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
    pub entities: HashMap<EntityID, Entity>,
    pub cells: [[Option<EntityID>; BAY_SIZE]; BAY_SIZE],
}

pub type EntityID = u64;

#[derive(Archive, Serialize, Deserialize, Clone)]
pub enum Entity {
    Wall,
    Bot(Bot),
    Resource(Resource),
    Interconnect { next_bay_id: EntityID },
    Antenna { controller_id: EntityID },
}

#[derive(Archive, Serialize, Deserialize, Clone)]
pub struct Bot {
    pub entity_id: EntityID,
    pub player_id: EntityID,
    pub energy: u32,
    pub held_resource: Option<Resource>,
    pub x: usize,
    pub y: usize,
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[archive(compare(PartialEq))]
pub enum Resource {
    Gold,
    Copper,
    Platinum,
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
