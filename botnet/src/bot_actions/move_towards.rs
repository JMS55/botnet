use super::BotAction;
use crate::compute_bot_action::StoreData;
use botnet_api::{ActionError, Bay, Bot, Cell, Direction, BAY_SIZE};
use std::error::Error;
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 10;

fn bot_can_move_towards(bot: &Bot, direction: Direction, bay: &Bay) -> Result<(), ActionError> {
    // Check if the bot has enough energy
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

    // Check if the adjacent cell is empty
    let empty = match direction {
        Direction::Up if bot.y != 0 => bay.cells[bot.x][bot.y - 1] == Cell::Empty,
        Direction::Down if bot.y != BAY_SIZE - 1 => bay.cells[bot.x][bot.y + 1] == Cell::Empty,
        Direction::Left if bot.x != 0 => bay.cells[bot.x - 1][bot.y] == Cell::Empty,
        Direction::Right if bot.x != BAY_SIZE - 1 => bay.cells[bot.x + 1][bot.y] == Cell::Empty,
        _ => false,
    };

    if empty {
        Ok(())
    } else {
        Err(ActionError::ActionNotPossible)
    }
}

pub fn apply_bot_move_towards(bay: &mut Bay, bot_id: u64, direction: Direction) {
    let bot = bay.bots.get_mut(&bot_id).unwrap();

    bay.cells[bot.x][bot.y] = Cell::Empty;

    let (new_x, new_y) = match direction {
        Direction::Up => (bot.x, bot.y - 1),
        Direction::Down => (bot.x, bot.y + 1),
        Direction::Left => (bot.x - 1, bot.y),
        Direction::Right => (bot.x + 1, bot.y),
    };
    bay.cells[new_x][new_y] = Cell::Bot { id: bot.id };
    (bot.x, bot.y) = (new_x, new_y);

    bot.energy -= ENERGY_REQUIRED;
}

pub fn export_move_towards(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, direction: u32| {
        let result = (|| {
            // Check if the bot has already decided on an action
            if caller.data().bot_action.is_some() {
                return Err(ActionError::AlreadyActed);
            }

            // Check if the action is possible
            let bay = caller.data().bay;
            let bot = bay.bots.get(&caller.data().bot_id).unwrap();
            let direction =
                Direction::wasm_to_rust(direction).map_err(|_| ActionError::ActionNotPossible)?;
            let result = bot_can_move_towards(bot, direction, bay)?;

            // Decide on the action
            caller.data_mut().bot_action = Some(BotAction::MoveTowards(direction));
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__move_towards", function)?;
    Ok(())
}
