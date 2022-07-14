mod deposit_resource;
mod harvest_resource;
mod log_debug;
mod move_towards;
mod withdraw_resource;

pub use deposit_resource::*;
pub use harvest_resource::*;
pub use log_debug::*;
pub use move_towards::*;
pub use withdraw_resource::*;

use botnet_api::{Direction, Resource};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug)]
#[archive_attr(derive(Debug))]
pub enum BotAction {
    MoveTowards(Direction),
    HarvestResource { x: u32, y: u32 },
    DepositResource { x: u32, y: u32 },
    WithdrawResource { resource: Resource, x: u32, y: u32 },
}
