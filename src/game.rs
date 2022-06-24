use crate::bay::BayExt;
use botnet_api::Bay;
use dashmap::DashMap;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use wasmtime::{Engine, Module};

pub const NETWORK_MEMORY_SIZE: usize = 1_000_000; // 1mb
pub const BOT_MEMORY_LIMIT: usize = 4_000_000; // 4mb

pub struct Game {
    players: Arc<DashMap<u64, Player>>,
    bays: Vec<Bay>,
    engine: Engine,
}

impl Game {
    pub fn new() -> Self {
        let engine = Engine::default();

        let players = DashMap::new();
        players.insert(
            1717,
            Player {
                network_memory: Arc::new(Mutex::new([0; NETWORK_MEMORY_SIZE])),
                script: Module::new(
                    &engine,
                    include_bytes!(
                        "../example_bot/target/wasm32-unknown-unknown/debug/example_bot.wasm"
                    ),
                )
                .unwrap(),
            },
        );

        Self {
            players: Arc::new(players),
            bays: vec![Bay::new()],
            engine,
        }
    }

    pub fn tick(&mut self) {
        self.bays.par_iter_mut().for_each(|bay| {
            let players = self.players.clone();
            bay.tick(&*players, &self.engine);
        });
    }
}

pub struct Player {
    pub network_memory: Arc<Mutex<[u8; NETWORK_MEMORY_SIZE]>>,
    pub script: Module,
}
