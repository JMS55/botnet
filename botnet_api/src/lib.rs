use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[cfg(feature = "default")]
pub use botnet_api_derive::bot;
#[cfg(feature = "default")]
pub use rkyv;

pub const BAY_SIZE: usize = 32;
pub type WASMUsize = i32;

#[derive(Archive, Serialize, Deserialize)]
pub struct Bay {
    pub bots: HashMap<u64, (Bot, u8, u8)>,
    pub cells: [[Cell; BAY_SIZE]; BAY_SIZE],
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Wall,
    Resource(Resource),
    Interconnect { next_bay_id: u64 },
    Antenna { controller_id: u64 },
    Bot { id: u64 },
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Archive, Serialize, Deserialize)]
pub struct Bot {
    pub player_id: u64,
    pub held_resource: Option<Resource>,
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Gold,
    Copper,
    Platinum,
    Silicon,
    Plastic,
}

pub enum BotAction {
    None,
    Move(Direction),
}

impl Into<WASMUsize> for BotAction {
    fn into(self) -> WASMUsize {
        match self {
            Self::None => 0,
            Self::Move(Direction::Up) => 1,
            Self::Move(Direction::Down) => 2,
            Self::Move(Direction::Left) => 3,
            Self::Move(Direction::Right) => 4,
        }
    }
}

impl TryFrom<WASMUsize> for BotAction {
    type Error = Box<dyn Error>;

    fn try_from(value: WASMUsize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Move(Direction::Up)),
            2 => Ok(Self::Move(Direction::Down)),
            3 => Ok(Self::Move(Direction::Left)),
            4 => Ok(Self::Move(Direction::Right)),
            _ => Err("Invalid value to convert to BotAction".into()),
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
