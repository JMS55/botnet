pub mod move_towards;

pub use move_towards::*;

use botnet_api::Direction;

#[derive(Clone, Copy)]
pub enum BotAction {
    MoveTowards(Direction),
}
