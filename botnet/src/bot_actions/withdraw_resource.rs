use crate::bot_actions::BotAction;
use crate::wasm_context::StoreData;
use botnet_api::{ActionError, Bay, Bot, EntityID, Resource};
use std::error::Error;
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 10;

fn bot_can_withdraw_resource(
    bot: &Bot,
    resource: Resource,
    x: u32,
    y: u32,
    bay: &Bay,
) -> Result<(), ActionError> {
    // Check if the bot has enough energy
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

    // Check if the bot is holding a resource
    if bot.held_resource.is_some() {
        return Err(ActionError::ActionNotPossible);
    }

    // Check if there is an antenna with the same controller at the given coordinates
    if !bay
        .get_entity_at_position(x, y)
        .map(|entity| entity.is_antenna_controlled_by(bot.controller_id))
        .unwrap_or(false)
    {
        return Err(ActionError::ActionNotPossible);
    }

    // Check if the antenna has none of the resource
    let antenna = bay
        .get_entity_at_position(x, y)
        .unwrap()
        .unwrap_as_antenna();
    let antenna_resource_count = match resource {
        Resource::Copper => antenna.stored_copper,
        Resource::Gold => antenna.stored_gold,
        Resource::Silicon => antenna.stored_silicon,
        Resource::Plastic => antenna.stored_plastic,
    };
    if antenna_resource_count == 0 {
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

pub fn apply_bot_withdraw_resource(
    bay: &mut Bay,
    bot_id: EntityID,
    resource: Resource,
    x: u32,
    y: u32,
) {
    let antenna = bay
        .get_mut_entity_at_position(x, y)
        .unwrap()
        .unwrap_mut_as_antenna();
    match resource {
        Resource::Copper => antenna.stored_copper -= 1,
        Resource::Gold => antenna.stored_gold -= 1,
        Resource::Silicon => antenna.stored_silicon -= 1,
        Resource::Plastic => antenna.stored_plastic -= 1,
    }

    let bot = &mut bay.get_bot_mut(bot_id).unwrap();
    bot.held_resource = Some(resource);
    bot.energy -= ENERGY_REQUIRED;
}

pub fn export_withdraw_resource(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, resource: u32, x: u32, y: u32| {
        let result = (|| {
            // Check if the bot has already decided on an action
            if caller.data().bot_action.is_some() {
                return Err(ActionError::AlreadyActed);
            }

            // Check if the action is possible
            let bay = caller.data().bay;
            let bot = bay.get_bot(caller.data().bot_id).unwrap();
            let resource =
                Resource::wasm_to_rust(resource).map_err(|_| ActionError::ActionNotPossible)?;
            let result = bot_can_withdraw_resource(bot, resource, x, y, bay)?;

            // Decide on the action
            caller.data_mut().bot_action = Some(BotAction::WithdrawResource { resource, x, y });
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__withdraw_resource", function)?;
    Ok(())
}
