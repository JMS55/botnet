use crate::bot_actions::BotAction;
use crate::wasm_context::StoreData;
use botnet_api::{ActionError, Bay, Bot, EntityID, Resource};
use std::error::Error;
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 5;

fn bot_can_deposit_resource(bot: &Bot, x: u32, y: u32, bay: &Bay) -> Result<(), ActionError> {
    // Check if the bot has enough energy
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

    // Check if the bot is not holding a resource
    if bot.held_resource == None {
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

    // Check if the antenna has no room for the resource
    let antenna = bay
        .get_entity_at_position(x, y)
        .unwrap()
        .unwrap_as_antenna();
    let antenna_resource_count = match bot.held_resource.unwrap() {
        Resource::Copper => antenna.stored_copper,
        Resource::Gold => antenna.stored_gold,
        Resource::Silicon => antenna.stored_silicon,
        Resource::Plastic => antenna.stored_plastic,
    };
    if antenna_resource_count == u8::MAX {
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

pub fn apply_bot_deposit_resource(bay: &mut Bay, bot_id: EntityID, x: u32, y: u32) {
    let bot = &mut bay.get_bot_mut(bot_id).unwrap();
    let bot_resource = bot.held_resource.take();
    bot.energy -= ENERGY_REQUIRED;

    let antenna = bay
        .get_mut_entity_at_position(x, y)
        .unwrap()
        .unwrap_mut_as_antenna();
    match bot_resource {
        Some(Resource::Copper) => antenna.stored_copper += 1,
        Some(Resource::Gold) => antenna.stored_gold += 1,
        Some(Resource::Silicon) => antenna.stored_silicon += 1,
        Some(Resource::Plastic) => antenna.stored_plastic += 1,
        None => unreachable!(),
    }
}

pub fn export_deposit_resource(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, x: u32, y: u32| {
        let result = (|| {
            // Check if the bot has already decided on an action
            if caller.data().bot_action.is_some() {
                return Err(ActionError::AlreadyActed);
            }

            // Check if the action is possible
            let bay = caller.data().bay;
            let bot = bay.get_bot(caller.data().bot_id).unwrap();
            let result = bot_can_deposit_resource(bot, x, y, bay)?;

            // Decide on the action
            caller.data_mut().bot_action = Some(BotAction::DepositResource { x, y });
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__deposit_resource", function)?;
    Ok(())
}
