use rkyv::with::Skip;
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "default")]
pub use botnet_api_derive::bot;
#[cfg(feature = "default")]
pub use rkyv;

pub const BAY_SIZE: usize = 32;

#[derive(Archive, Serialize, Deserialize)]
pub struct Bay {
    pub bots: HashMap<u64, Bot>,
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

#[derive(Archive, Serialize, Deserialize, Clone)]
pub struct Bot {
    #[with(Skip)]
    pub id: u64,
    pub player_id: u64,
    pub energy: u32,
    pub held_resource: Option<Resource>,
    pub x: usize,
    pub y: usize,
}

impl Bot {
    pub fn can_move_towards(&self, bay: &Bay, x: usize, y: usize) -> Result<(), ActionError> {
        todo!()
    }
}

impl ArchivedBot {
    pub fn move_towards(&self, x: usize, y: usize) -> Result<(), ActionError> {
        extern "C" {
            fn __move_towards(x: u32, y: u32) -> u32;
        }
        ActionError::wasm_to_host(unsafe { __move_towards(x as u32, y as u32) })
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Gold,
    Copper,
    Platinum,
    Silicon,
    Plastic,
}

pub enum ActionError {
    ActionNotPossible,
    NotEnoughEnergy,
    AlreadyActed,
}

#[doc(hidden)]
impl ActionError {
    fn wasm_to_host(result: u32) -> Result<(), ActionError> {
        match result {
            0 => Ok(()),
            1 => Err(Self::ActionNotPossible),
            2 => Err(Self::NotEnoughEnergy),
            3 => Err(Self::AlreadyActed),
            _ => unreachable!(),
        }
    }

    pub fn host_to_wasm(result: Result<(), ActionError>) -> u32 {
        match result {
            Ok(()) => 0,
            Err(Self::ActionNotPossible) => 1,
            Err(Self::NotEnoughEnergy) => 2,
            Err(Self::AlreadyActed) => 3,
        }
    }
}
