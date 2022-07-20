use botnet_api::bot;
use botnet_api::rkyv::option::ArchivedOption;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
struct NetworkMemory {
    bot_paths: HashMap<EntityID, Option<Vec<(u32, u32)>>>,
    antenna_location: Option<(u32, u32)>,
}

#[bot(NetworkMemory)]
fn tick(bot: &ArchivedBot, bay: &ArchivedBay, network_memory: &mut NetworkMemory) {
    // Choose a location for an antenna
    if network_memory.antenna_location == None {
        'l: for x in 10..BAY_SIZE {
            for y in 10..BAY_SIZE {
                if bay.cells[x][y].is_none() {
                    network_memory.antenna_location = Some((x as u32, y as u32));
                    break 'l;
                }
            }
        }
    }

    // Find a new path when none cached
    let bot_path = network_memory.bot_paths.entry(bot.id).or_default();
    if *bot_path == None {
        match &bot.held_resource {
            // If bot has no resource, pathfind to a new one
            ArchivedOption::None => {
                *bot_path = bot.find_path_to(RESOURCE, bay);
                log_debug(&format!("Found new path to resource: {:?}", bot_path));
            }
            // If bot has a resource, pathfind to the partial antenna if can help build it,
            // or a complete antenna to deposit into it
            ArchivedOption::Some(held_resource) => {
                let (antenna_x, antenna_y) = network_memory.antenna_location.unwrap();
                let can_use_resource = match bay.get_entity_at_position(antenna_x, antenna_y) {
                    None => true,
                    Some(ArchivedEntity::PartialEntity(partial_entity)) => {
                        partial_entity.needs_resource(held_resource)
                    }
                    Some(ArchivedEntity::Antenna(_)) => true,
                    _ => false,
                };
                if can_use_resource {
                    *bot_path = bot.find_path_to(POSITION(antenna_x, antenna_y), bay);
                    log_debug(&format!("Found new path to antenna: {:?}", bot_path));
                }
            }
        }
    }

    match bot_path {
        // If at a resource/antenna, perform the appropriate action on it
        Some(path) if path.len() == 1 => {
            let (x, y) = path.pop().unwrap();
            *bot_path = None;

            if bot.held_resource.is_none() {
                bot.harvest_resource(x, y);
            } else {
                bot.deposit_resource(x, y);
                bot.build_entity(PartialEntityType::Antenna, x, y);
            }
        }
        // Otherwise move along the cached path
        Some(path) => {
            let move_result = bot.move_along_path(path);
            if move_result == Err(ActionError::ActionNotPossible) {
                *bot_path = None;
            }
        }
        None => {}
    }
}
