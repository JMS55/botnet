use crate::{ActionError, ArchivedBot, Direction, PartialEntityType, Resource};

pub const BOTNET_API_VERSION: [u32; 3] = [0, 1, 0];

impl ArchivedBot {
    pub fn move_towards(&self, direction: Direction) -> Result<(), ActionError> {
        extern "C" {
            fn __move_towards(direction: u32) -> u32;
        }
        ActionError::wasm_to_rust(unsafe { __move_towards(direction.rust_to_wasm()) }).unwrap()
    }

    pub fn harvest_resource(&self, x: u32, y: u32) -> Result<(), ActionError> {
        extern "C" {
            fn __harvest_resource(x: u32, y: u32) -> u32;
        }
        ActionError::wasm_to_rust(unsafe { __harvest_resource(x, y) }).unwrap()
    }

    pub fn deposit_resource(&self, x: u32, y: u32) -> Result<(), ActionError> {
        extern "C" {
            fn __deposit_resource(x: u32, y: u32) -> u32;
        }
        ActionError::wasm_to_rust(unsafe { __deposit_resource(x, y) }).unwrap()
    }

    pub fn withdraw_resource(&self, resource: Resource, x: u32, y: u32) -> Result<(), ActionError> {
        extern "C" {
            fn __withdraw_resource(resource: u32, x: u32, y: u32) -> u32;
        }
        ActionError::wasm_to_rust(unsafe { __withdraw_resource(resource.rust_to_wasm(), x, y) })
            .unwrap()
    }

    pub fn build_entity(
        &self,
        entity_type: PartialEntityType,
        x: u32,
        y: u32,
    ) -> Result<(), ActionError> {
        extern "C" {
            fn __build_entity(entity_type: u32, x: u32, y: u32) -> u32;
        }
        ActionError::wasm_to_rust(unsafe { __build_entity(entity_type.rust_to_wasm(), x, y) })
            .unwrap()
    }

    pub fn move_along_path(&self, path: &mut Vec<(u32, u32)>) -> Result<(), ActionError> {
        match path.last().copied() {
            Some((x, y)) => {
                let x = x as i32 - self.x as i32;
                let y = self.y as i32 - y as i32;
                let direction = match (x, y) {
                    (1, 0) => Direction::Right,
                    (-1, 0) => Direction::Left,
                    (0, 1) => Direction::Up,
                    (0, -1) => Direction::Down,
                    _ => return Err(ActionError::ActionNotPossible),
                };
                let move_result = self.move_towards(direction);
                if move_result.is_ok() {
                    path.pop().unwrap();
                }
                move_result
            }
            _ => Err(ActionError::ActionNotPossible),
        }
    }
}

#[cfg(feature = "default")]
pub fn log_debug(message: &str) {
    extern "C" {
        fn __log_debug(pointer: u32, length: u32);
    }
    unsafe {
        __log_debug(message.as_ptr() as u32, message.len() as u32);
    }
}

#[doc(hidden)]
impl ActionError {
    pub fn rust_to_wasm(result: Result<(), Self>) -> u32 {
        match result {
            Ok(()) => 0,
            Err(Self::ActionNotPossible) => 1,
            Err(Self::NotEnoughEnergy) => 2,
            Err(Self::AlreadyActed) => 3,
        }
    }

    pub fn wasm_to_rust(result: u32) -> Result<Result<(), Self>, ()> {
        match result {
            0 => Ok(Ok(())),
            1 => Ok(Err(Self::ActionNotPossible)),
            2 => Ok(Err(Self::NotEnoughEnergy)),
            3 => Ok(Err(Self::AlreadyActed)),
            _ => Err(()),
        }
    }
}

#[doc(hidden)]
impl Direction {
    pub fn rust_to_wasm(&self) -> u32 {
        match self {
            Self::Up => 0,
            Self::Down => 1,
            Self::Left => 2,
            Self::Right => 3,
        }
    }

    pub fn wasm_to_rust(direction: u32) -> Result<Self, ()> {
        match direction {
            0 => Ok(Self::Up),
            1 => Ok(Self::Down),
            2 => Ok(Self::Left),
            3 => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

#[doc(hidden)]
impl Resource {
    pub fn rust_to_wasm(&self) -> u32 {
        match self {
            Self::Copper => 0,
            Self::Gold => 1,
            Self::Silicon => 2,
            Self::Plastic => 3,
        }
    }

    pub fn wasm_to_rust(direction: u32) -> Result<Self, ()> {
        match direction {
            0 => Ok(Self::Copper),
            1 => Ok(Self::Gold),
            2 => Ok(Self::Silicon),
            3 => Ok(Self::Plastic),
            _ => Err(()),
        }
    }
}

#[doc(hidden)]
impl PartialEntityType {
    pub fn rust_to_wasm(&self) -> u32 {
        match self {
            Self::Antenna => 0,
        }
    }

    pub fn wasm_to_rust(direction: u32) -> Result<Self, ()> {
        match direction {
            0 => Ok(Self::Antenna),

            _ => Err(()),
        }
    }
}
