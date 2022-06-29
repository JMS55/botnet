use crate::bay::BayExt;
use crate::config::NETWORK_MEMORY_SIZE;
use crate::wasm_engine::WasmEngine;
use botnet_api::Bay;
use dashmap::DashMap;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use wasmtime::Module;

/// High level object for managing an instance of the game server.
///
/// Holds player data, and bay data.
pub struct Game {
    players: Arc<DashMap<u64, Player>>,
    bays: Vec<Bay>,
    wasm_engine: WasmEngine,
}

pub struct Player {
    pub network_memory: Arc<Mutex<[u8; NETWORK_MEMORY_SIZE]>>,
    pub script: Module,
}

impl Game {
    pub fn new() -> Self {
        let wasm_engine = WasmEngine::new();

        let players = DashMap::new();
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

        Self {
            players: Arc::new(players),
            bays: vec![Bay::new()],
            wasm_engine,
        }
    }

    /// Ticks each bay in the game in parallel.
    pub fn tick(&mut self) {
        self.bays
            .par_iter_mut()
            .enumerate()
            .for_each(|(bay_id, bay)| {
                let players = self.players.clone();
                bay.tick(bay_id, &*players, &self.wasm_engine.engine);
            });
    }
}
