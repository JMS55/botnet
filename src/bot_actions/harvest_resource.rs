use super::BotAction;
use crate::compute_bot_action::StoreData;
use botnet_api::{ActionError, Bay, Bot, Cell, BAY_SIZE};
use std::error::Error;
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 30;

fn bot_can_harvest_resource(bot: &Bot, x: u32, y: u32, bay: &Bay) -> Result<(), ActionError> {
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

    if bot.held_resource.is_some() {
        return Err(ActionError::ActionNotPossible);
    }

    let (x, y) = (x as usize, y as usize);
    if x >= BAY_SIZE || y >= BAY_SIZE {
        return Err(ActionError::ActionNotPossible);
    }

    match bay.cells[x][y] {
        Cell::Resource(_) => {}
        _ => return Err(ActionError::ActionNotPossible),
    }

    let (xo, yo) = (
        (bot.x as i32 - x as i32).abs(),
        (bot.y as i32 - y as i32).abs(),
    );
    if xo + yo != 1 {
        return Err(ActionError::ActionNotPossible);
    }

    Ok(())
}

pub fn apply_bot_harvest_resource(bay: &mut Bay, bot_id: u64, x: u32, y: u32) {
    let bot = bay.bots.get_mut(&bot_id).unwrap();

    let resource = match bay.cells[x as usize][y as usize] {
        botnet_api::Cell::Resource(resource) => resource,
        _ => unreachable!(),
    };

    bay.cells[x as usize][y as usize] = Cell::Empty;
    bot.held_resource = Some(resource);

    bot.energy -= ENERGY_REQUIRED;
}

pub fn export_harvest_resource(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, x: u32, y: u32| {
        let result = (|| {
            if caller.data().bot_action.is_some() {
                return Err(ActionError::AlreadyActed);
            }

            let bay = caller.data().bay;
            let bot = bay.bots.get(&caller.data().bot_id).unwrap();
            let result = bot_can_harvest_resource(bot, x, y, bay)?;

            caller.data_mut().bot_action = Some(BotAction::HarvestResource { x, y });
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__harvest_resource", function)?;
    Ok(())
}