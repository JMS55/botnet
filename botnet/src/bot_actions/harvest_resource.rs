use crate::bot_actions::BotAction;
use crate::wasm_context::StoreData;
use botnet_api::{ActionError, Bay, Bot, Entity, EntityID};
use std::error::Error;
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 30;

fn bot_can_harvest_resource(bot: &Bot, x: u32, y: u32, bay: &Bay) -> Result<(), ActionError> {
    // Check if the bot has enough energy
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

    // Check if the bot is already holding a resource
    if bot.held_resource.is_some() {
        return Err(ActionError::ActionNotPossible);
    }

    // Check if there is a resource at the given coordinates
    if !bay
        .get_entity_at_position(x, y)
        .map(Entity::is_resource)
        .unwrap_or(false)
    {
        return Err(ActionError::ActionNotPossible);
    }

    // Check if the coordinates are adjacent to the bot
    let (xo, yo) = (
        (bot.x as i32 - x as i32).abs(),
        (bot.y as i32 - y as i32).abs(),
    );
    if xo + yo != 1 {
        return Err(ActionError::ActionNotPossible);
    }

    Ok(())
}

pub fn apply_bot_harvest_resource(bay: &mut Bay, bot_id: EntityID, x: u32, y: u32) {
    let resource = *bay
        .get_entity_at_position(x, y)
        .unwrap()
        .unwrap_as_resource();
    bay.entities
        .remove(&bay.cells[x as usize][y as usize].unwrap());
    bay.cells[x as usize][y as usize] = None;

    let bot = bay.get_bot_mut(bot_id).unwrap();
    bot.held_resource = Some(resource);
    bot.energy -= ENERGY_REQUIRED;
}

pub fn export_harvest_resource(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, x: u32, y: u32| {
        let result = (|| {
            // Check if the bot has already decided on an action
            if caller.data().bot_action.is_some() {
                return Err(ActionError::AlreadyActed);
            }

            // Check if the action is possible
            let bay = caller.data().bay;
            let bot = bay.get_bot(caller.data().bot_id).unwrap();
            let result = bot_can_harvest_resource(bot, x, y, bay)?;

            // Decide on the action
            caller.data_mut().bot_action = Some(BotAction::HarvestResource { x, y });
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__harvest_resource", function)?;
    Ok(())
}
