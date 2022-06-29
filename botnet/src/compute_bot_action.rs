use crate::bot_actions::*;
use crate::config::{BOT_MEMORY_LIMIT, BOT_SETUP_TIME_LIMIT, BOT_TIME_LIMIT, NETWORK_MEMORY_SIZE};
use crate::game::Player;
use botnet_api::Bay;
use std::error::Error;
use wasmtime::{Engine, Linker, Store, StoreLimits, StoreLimitsBuilder};

/// Runs the player's wasm script to decide on an action for one of their bots.
pub fn compute_bot_action(
    bot_id: u64,
    engine: &Engine,
    bay: &Bay,
    player: &Player,
) -> Result<BotAction, Box<dyn Error>> {
    // Setup an instance of the bot script
    let mut store = Store::new(
        engine,
        StoreData {
            limits: StoreLimitsBuilder::new()
                .memory_size(BOT_MEMORY_LIMIT)
                .build(),
            bot_action: None,
            bot_id,
            bay,
        },
    );
    store.limiter(|data| &mut data.limits);
    let linker = setup_linker(engine)?;
    let instance = linker.instantiate(&mut store, &player.script)?;

    // Get bot script exports
    let instance_tick =
        instance.get_typed_func::<(u64, u32, u32, u32, u32), (), _>(&mut store, "__tick")?;
    let instance_memalloc = instance.get_typed_func::<u32, u32, _>(&mut store, "__memalloc")?;
    let instance_memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("No memory exported for bot instance")?;

    // Set time limit for setting up bay data and network memory
    store.set_epoch_deadline(BOT_SETUP_TIME_LIMIT);

    // Serialize and copy bay to the bot instance
    let bay = rkyv::to_bytes::<_, 25_000>(bay)?;
    let bay_size = bay.len() as u32;
    let bay_pointer = instance_memalloc.call(&mut store, bay_size)?;
    if bay_pointer == 0 {
        return Err("__memalloc returned a null pointer for bay".into());
    }
    instance_memory.write(&mut store, bay_pointer as usize, &bay)?;

    // Copy network memory from the player's account to the bot instance
    let mut network_memory = player
        .network_memory
        .lock()
        .or(Err("Player network_memory lock was poisoned"))?;
    let network_memory_pointer = instance_memalloc.call(&mut store, NETWORK_MEMORY_SIZE as u32)?;
    if network_memory_pointer == 0 {
        return Err("__memalloc returned a null pointer for network memory".into());
    }
    instance_memory.write(
        &mut store,
        network_memory_pointer as usize,
        &*network_memory,
    )?;

    // Tick the bot instance to compute an action for the bot to take
    store.set_epoch_deadline(BOT_TIME_LIMIT);
    instance_tick.call(
        &mut store,
        (
            bot_id,
            bay_pointer,
            bay_size,
            network_memory_pointer,
            NETWORK_MEMORY_SIZE as u32,
        ),
    )?;

    // Copy network memory from the bot instance back to the player's account
    instance_memory.read(
        &mut store,
        network_memory_pointer as usize,
        &mut *network_memory,
    )?;
    drop(network_memory);

    // Return the action the bot wants to take, if any
    store
        .data()
        .bot_action
        .ok_or_else(|| "Bot script did not set an action".into())
}

pub struct StoreData<'a> {
    limits: StoreLimits,
    pub bot_action: Option<BotAction>,
    pub bot_id: u64,
    pub bay: &'a Bay,
}

fn setup_linker(engine: &Engine) -> Result<Linker<StoreData>, Box<dyn Error>> {
    let mut linker = Linker::new(engine);

    export_move_towards(&mut linker)?;
    export_harvest_resource(&mut linker)?;
    export_log_debug(&mut linker)?;

    Ok(linker)
}