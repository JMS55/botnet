use botnet_api::bot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
struct NetworkMemory {
    bot_paths: HashMap<u64, Option<Vec<(u32, u32)>>>,
}

#[bot(NetworkMemory)]
fn tick(bot: &ArchivedBot, bay: &ArchivedBay, network_memory: &mut NetworkMemory) {
    let bot_path = network_memory.bot_paths.entry(bot.id).or_default();

    if *bot_path == None {
        *bot_path = bot.find_path_to(RESOURCE, bay);
        log_debug(&format!("Found new path to resource: {:?}", bot_path));
    }

    if let Some(path) = bot_path {
        if path.len() == 1 {
            let (x, y) = path.pop().unwrap();
            bot.harvest_resource(x, y);
            *bot_path = None;
        } else {
            bot.move_along_path(path);
        }
    }
}
