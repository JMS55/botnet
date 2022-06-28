mod harvest_resource;
mod log_debug;
mod move_towards;

pub use harvest_resource::*;
pub use log_debug::*;
pub use move_towards::*;

use botnet_api::Direction;

#[derive(Clone, Copy, Debug)]
pub enum BotAction {
    MoveTowards(Direction),
    HarvestResource { x: u32, y: u32 },
}
