use botnet_api::bot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
struct NetworkMemory {
    bot_paths: HashMap<EntityID, Option<Vec<(u32, u32)>>>,
}

#[bot(NetworkMemory)]
fn tick(bot: &ArchivedBot, bay: &ArchivedBay, network_memory: &mut NetworkMemory) {
    let bot_path = network_memory.bot_paths.entry(bot.id).or_default();

    if *bot_path == None {
        let target = if bot.held_resource.is_none() {
            RESOURCE
        } else {
            ANTENNA
        };
        *bot_path = bot.find_path_to(target, bay);

        log_debug(&format!("Found new path to resource: {:?}", bot_path));
    }

    match bot_path {
        Some(path) if path.len() == 1 => {
            let (x, y) = path.pop().unwrap();
            *bot_path = None;

            if bot.held_resource.is_none() {
                bot.harvest_resource(x, y);
            } else {
                bot.deposit_resource(x, y);
            }
        }
        Some(path) => {
            let move_result = bot.move_along_path(path);
            if move_result == Err(ActionError::ActionNotPossible) {
                *bot_path = None;
            }
        }
        None => {}
    }
}
