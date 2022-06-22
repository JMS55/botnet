use crate::{ActionError, ArchivedBot, Bay, Bot};

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
