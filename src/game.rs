use crate::bay::BayExt;
use botnet_api::Bay;
use dashmap::DashMap;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use wasmtime::{Config, Engine, Module};

pub const NETWORK_MEMORY_SIZE: usize = 1_000_000; // 1mb
pub const BOT_TIME_LIMIT: u64 = 5; // 5ms
pub const BOT_MEMORY_LIMIT: usize = 4_000_000; // 4mb

pub struct Game {
    players: Arc<DashMap<u64, Player>>,
    bays: Vec<Bay>,
    engine: Engine,
    epoch_increment_thread: Option<JoinHandle<()>>,
}

impl Game {
    pub fn new() -> Self {
        let engine = Engine::new(&Config::new().epoch_interruption(true)).unwrap();

        let players = DashMap::new();
        players.insert(
            1717,
            Player {
                network_memory: Arc::new(Mutex::new([0; NETWORK_MEMORY_SIZE])),
                script: Module::new(
                    &engine,
                    include_bytes!(
                        "../example_bot/target/wasm32-unknown-unknown/release/example_bot.wasm"
                    ),
                )
                .unwrap(),
            },
        );

        let epoch_increment_thread = thread::spawn({
            let engine = engine.clone();
            move || {
                thread::sleep(Duration::from_millis(1));
                engine.increment_epoch();
            }
        });

        Self {
            players: Arc::new(players),
            bays: vec![Bay::new()],
            engine,
            epoch_increment_thread: Some(epoch_increment_thread),
        }
    }

    pub fn tick(&mut self) {
        self.bays
            .par_iter_mut()
            .enumerate()
            .for_each(|(bay_id, bay)| {
                let players = self.players.clone();
                bay.tick(bay_id, &*players, &self.engine);
            });
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.epoch_increment_thread.take().unwrap().join().unwrap();
    }
}

pub struct Player {
    pub network_memory: Arc<Mutex<[u8; NETWORK_MEMORY_SIZE]>>,
    pub script: Module,
}
