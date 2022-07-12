use crate::bot_actions::BotAction;
use crate::wasm_context::StoreData;
use botnet_api::{ActionError, Bay, Bot, Direction, EntityID, BAY_SIZE};
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
        Direction::Up if bot.y != 0 => bay.cells[bot.x][bot.y - 1].is_none(),
        Direction::Down if bot.y != BAY_SIZE - 1 => bay.cells[bot.x][bot.y + 1].is_none(),
        Direction::Left if bot.x != 0 => bay.cells[bot.x - 1][bot.y].is_none(),
        Direction::Right if bot.x != BAY_SIZE - 1 => bay.cells[bot.x + 1][bot.y].is_none(),
        _ => false,
    };

    if empty {
        Ok(())
    } else {
        Err(ActionError::ActionNotPossible)
    }
}

pub fn apply_bot_move_towards(bay: &mut Bay, bot_id: EntityID, direction: Direction) {
    let bot = bay.get_bot_mut(bot_id).unwrap();

    let (old_x, old_y) = (bot.x, bot.y);
    let (new_x, new_y) = match direction {
        Direction::Up => (bot.x, bot.y - 1),
        Direction::Down => (bot.x, bot.y + 1),
        Direction::Left => (bot.x - 1, bot.y),
        Direction::Right => (bot.x + 1, bot.y),
    };

    (bot.x, bot.y) = (new_x, new_y);
    bot.energy -= ENERGY_REQUIRED;

    bay.cells[old_x][old_y] = None;
    bay.cells[new_x][new_y] = Some(bot_id);
    let (_, x, y) = bay.entities.get_mut(&bot_id).unwrap();
    (*x, *y) = (new_x as u32, new_y as u32);
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
            let bot = bay.get_bot(caller.data().bot_id).unwrap();
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
