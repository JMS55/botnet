use crate::bay::BayExt;
use crate::config::NETWORK_MEMORY_SIZE;
use crate::wasm_context::WasmContext;
use botnet_api::Bay;
use log::info;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use wasmtime::Module;

/// High level object for managing an instance of the game server.
///
/// Holds player data, and bay data.
pub struct Game<'a> {
    players: HashMap<u64, Player>,
    bays: Vec<Bay>,
    wasm_engine: WasmContext<'a>,
}

pub struct Player {
    pub network_memory: Arc<Mutex<[u8; NETWORK_MEMORY_SIZE]>>,
    pub script: Module,
}

impl Game<'_> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let wasm_engine = WasmContext::new()?;

        let mut players = HashMap::new();
        players.insert(
            1717,
            Player {
                network_memory: Arc::new(Mutex::new([0; NETWORK_MEMORY_SIZE])),
                script: Module::new(
                    &wasm_engine.engine,
                    include_bytes!(
                        "../../example_bot/target/wasm32-unknown-unknown/release/example_bot.wasm"
                    ),
                )
                .unwrap(),
            },
        );

        let bays = vec![Bay::new()];

        Ok(Self {
            players,
            bays,
            wasm_engine,
        })
    }

    /// Update the game.
    pub fn tick(&mut self) {
        // Tick each bay in parallel
        self.bays
            .par_iter_mut()
            .enumerate()
            .for_each(|(bay_id, bay)| {
                info!("Bay[{bay_id}] starting tick");

                bay.tick(&self.players, &self.wasm_engine);
            });
    }
}
