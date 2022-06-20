use crate::game::{Player, BOT_MEMORY_LIMIT, NETWORK_MEMORY_SIZE};
use botnet_api::{Bay, Bot, BotAction, WASMUsize};
use dashmap::DashMap;
use extension_traits::extension;
use std::error::Error;
use wasmtime::{Engine, Instance, Store, StoreLimitsBuilder};

#[extension(pub trait BotExt)]
impl Bot {
    fn compute_action(
        &self,
        bot_id: u64,
        engine: &Engine,
        players: &DashMap<u64, Player>,
        bay: &Bay,
    ) -> Result<BotAction, Box<dyn Error>> {
        // Setup an instance of the bot script
        let script =
            players.get(&self.player_id).unwrap().value().scripts[self.script_slot].clone();
        let mut store = Store::new(
            engine,
            StoreLimitsBuilder::new()
                .memory_size(BOT_MEMORY_LIMIT)
                .build(),
        );
        store.limiter(|state| state);
        let instance = Instance::new(&mut store, &script, &[])?;
        let instance_tick = instance
            .get_typed_func::<(u64, WASMUsize, WASMUsize, WASMUsize, WASMUsize), WASMUsize, _>(
                &mut store, "__tick",
            )?;
        let instance_memalloc =
            instance.get_typed_func::<WASMUsize, WASMUsize, _>(&mut store, "__memalloc")?;
        let instance_memory = instance
            .get_memory(&mut store, "memory")
            .ok_or("No memory exported for bot instance")?;

        // Serialize and copy bay to the bot instance
        let bay = rkyv::to_bytes::<_, 25_000>(bay)?;
        let bay_size = bay.len() as WASMUsize;
        let bay_ptr = instance_memalloc.call(&mut store, bay_size)?;
        if bay_ptr == 0 {
            return Err("__memalloc returned a null ptr for bay".into());
        }
        instance_memory.write(&mut store, bay_ptr as usize, &bay)?;

        // Copy network memory from the player's account to the bot instance
        let network_memory = players.get(&self.player_id).unwrap().network_memory.clone();
        let mut network_memory = network_memory.lock().unwrap();
        let network_memory_ptr =
            instance_memalloc.call(&mut store, NETWORK_MEMORY_SIZE as WASMUsize)?;
        if network_memory_ptr == 0 {
            return Err("__memalloc returned a null ptr for network memory".into());
        }
        instance_memory.write(&mut store, network_memory_ptr as usize, &*network_memory)?;

        // Tick the bot instance to compute an action for the bot to take
        let bot_action = instance_tick
            .call(
                &mut store,
                (
                    bot_id,
                    bay_ptr,
                    bay_size,
                    network_memory_ptr,
                    NETWORK_MEMORY_SIZE as WASMUsize,
                ),
            )?
            .try_into();

        // Copy network memory from the bot instance back to the player's account
        instance_memory.read(
            &mut store,
            network_memory_ptr as usize,
            &mut *network_memory,
        )?;

        bot_action
    }
}
