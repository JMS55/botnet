use super::BotAction;
use crate::bot_compute_action::StoreData;
use botnet_api::{ActionError, Bay, Bot, Cell, Direction, BAY_SIZE};
use std::error::Error;
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 10;

pub fn bot_can_move_towards(bot: &Bot, direction: Direction, bay: &Bay) -> Result<(), ActionError> {
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

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

pub fn apply_bot_move_towards(bay: &mut Bay, bot: &mut Bot, direction: Direction) {
    bay.cells[bot.x][bot.y] = Cell::Empty;
    let new_position = match direction {
        Direction::Up => &mut bay.cells[bot.x][bot.y - 1],
        Direction::Down => &mut bay.cells[bot.x][bot.y + 1],
        Direction::Left => &mut bay.cells[bot.x - 1][bot.y],
        Direction::Right => &mut bay.cells[bot.x + 1][bot.y],
    };
    *new_position = Cell::Bot { id: bot.id };

    bot.energy -= ENERGY_REQUIRED;
}

pub fn export_move_towards(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, direction: u32| {
        let result = (|| {
            if caller.data().bot_action.is_none() {
                return Err(ActionError::AlreadyActed);
            }

            let bay = caller.data().bay;
            let bot = bay.bots.get(&caller.data().bot_id).unwrap();
            let direction =
                Direction::wasm_to_rust(direction).map_err(|_| ActionError::ActionNotPossible)?;
            let result = bot_can_move_towards(bot, direction, bay)?;

            caller.data_mut().bot_action = Some(BotAction::MoveTowards(direction));
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__move_towards", function)?;
    Ok(())
}
