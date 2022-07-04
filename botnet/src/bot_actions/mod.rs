mod harvest_resource;
mod log_debug;
mod move_towards;

pub use harvest_resource::*;
pub use log_debug::*;
pub use move_towards::*;

use botnet_api::Direction;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug)]
#[archive_attr(derive(Debug))]
pub enum BotAction {
    MoveTowards(Direction),
    HarvestResource { x: u32, y: u32 },
}
