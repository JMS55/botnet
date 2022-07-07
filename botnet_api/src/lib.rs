mod api;
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
    pub bots: HashMap<u64, Bot>,
    pub cells: [[Cell; BAY_SIZE]; BAY_SIZE],
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[archive(compare(PartialEq))]
pub enum Cell {
    Empty,
    Wall,
    Resource(Resource),
    Interconnect { next_bay_id: u64 },
    Antenna { controller_id: u64 },
    Bot { id: u64 },
}

#[derive(Archive, Serialize, Deserialize, Clone)]
pub struct Bot {
    pub id: u64,
    pub player_id: u64,
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
