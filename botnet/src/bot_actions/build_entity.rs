use crate::bot_actions::BotAction;
use crate::partial_entity::{PartialEntityExt, PartialEntityTypeExt};
use crate::wasm_context::StoreData;
use botnet_api::{
    ActionError, Bay, Bot, Entity, EntityID, PartialEntity, PartialEntityType, Resource,
};
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use wasmtime::{Caller, Linker};

const ENERGY_REQUIRED: u32 = 30;

fn bot_can_build_entity(
    bot: &Bot,
    entity_type: PartialEntityType,
    x: u32,
    y: u32,
    bay: &Bay,
) -> Result<(), ActionError> {
    // Check if the bot has enough energy
    if bot.energy < ENERGY_REQUIRED {
        return Err(ActionError::NotEnoughEnergy);
    }

    match bay.controller_id {
        Some(bay_controller_id) => {
            // Check if the bay is not controlled by the bot's controller if it has one
            if bot.controller_id != bay_controller_id {
                return Err(ActionError::ActionNotPossible);
            }
            // Check if the bay already has an antenna if the bot is trying to build one
            if entity_type == PartialEntityType::Antenna {
                return Err(ActionError::ActionNotPossible);
            }
        }
        None => {
            // Check if the bay does not have a controller while trying to build something besides an antenna
            if entity_type != PartialEntityType::Antenna {
                return Err(ActionError::ActionNotPossible);
            }
        }
    }

    match bay.get_entity_at_position(x, y) {
        // Check if there is an existing partial entity of a different type at these coordinates
        Some(Entity::PartialEntity(PartialEntity {
            entity_type: et, ..
        })) if *et != entity_type => return Err(ActionError::ActionNotPossible),
        // Check if the bot's held resource can be used towards constructing the entity
        Some(Entity::PartialEntity(partial_entity)) => match bot.held_resource {
            Some(resource) if partial_entity.needs_resource(resource) => {}
            _ => return Err(ActionError::ActionNotPossible),
        },
        // Check if there is an existing entity at these coordinates
        Some(_) => return Err(ActionError::ActionNotPossible),
        None => {}
    }

    Ok(())
}

pub fn apply_bot_build_entity(
    bay: &mut Bay,
    bot_id: EntityID,
    entity_type: PartialEntityType,
    x: u32,
    y: u32,
    next_entity_id: &AtomicU64,
) {
    let bot = bay.get_bot_mut(bot_id).unwrap();
    let bot_controller_id = bot.controller_id;
    let bot_held_resource = bot.held_resource.take().unwrap();
    bot.energy -= ENERGY_REQUIRED;

    match bay.get_mut_entity_at_position(x, y) {
        Some(Entity::PartialEntity(partial_entity)) => match bot_held_resource {
            Resource::Copper => partial_entity.contributed_copper += 1,
            Resource::Gold => partial_entity.contributed_gold += 1,
            Resource::Silicon => partial_entity.contributed_silicon += 1,
            Resource::Plastic => partial_entity.contributed_plastic += 1,
        },
        None => {
            let entity_id = next_entity_id.fetch_add(1, Ordering::SeqCst);
            bay.entities
                .insert(entity_id, (entity_type.new_partial_entity(), x, y));
            bay.cells[x as usize][y as usize] = Some(entity_id);
        }
        _ => unreachable!(),
    }

    let (entity, _, _) = bay
        .entities
        .get_mut(&bay.cells[x as usize][y as usize].unwrap())
        .unwrap();
    let partial_entity = entity.unwrap_as_partial_entity();
    if partial_entity.contributed_copper == partial_entity.required_copper
        && partial_entity.contributed_gold == partial_entity.required_gold
        && partial_entity.contributed_silicon == partial_entity.required_silicon
        && partial_entity.contributed_plastic == partial_entity.required_plastic
    {
        entity.partial_entity_into_entity(bot_controller_id);

        if entity_type == PartialEntityType::Antenna {
            bay.controller_id = Some(bot_controller_id);
        }
    }
}

pub fn export_build_entity(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    let function = |mut caller: Caller<StoreData>, entity_type: u32, x: u32, y: u32| {
        let result = (|| {
            // Check if the bot has already decided on an action
            if caller.data().bot_action.is_some() {
                return Err(ActionError::AlreadyActed);
            }

            // Check if the action is possible
            let bay = caller.data().bay;
            let bot = bay.get_bot(caller.data().bot_id).unwrap();
            let entity_type = PartialEntityType::wasm_to_rust(entity_type)
                .map_err(|_| ActionError::ActionNotPossible)?;
            let result = bot_can_build_entity(bot, entity_type, x, y, bay)?;

            // Decide on the action
            caller.data_mut().bot_action = Some(BotAction::BuildEntity { entity_type, x, y });
            Ok(result)
        })();
        ActionError::rust_to_wasm(result)
    };

    linker.func_wrap("env", "__build_entity", function)?;
    Ok(())
}
