use crate::bay::BayExt;
use botnet_api::Bay;
use dashmap::DashMap;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wasmtime::{Config, Engine, Module};

pub const NETWORK_MEMORY_SIZE: usize = 512_000; // 512kb
pub const BOT_TIME_LIMIT: u64 = 100; // ~1ms, depending on scheduler behavior and when increment_epoch() is called
pub const BOT_SETUP_TIME_LIMIT: u64 = 25; // ~0.25ms, depending on scheduler behavior and when increment_epoch() is called
pub const BOT_MEMORY_LIMIT: usize = 2_000_000; // 2mb

pub struct Game {
    players: Arc<DashMap<u64, Player>>,
    bays: Vec<Bay>,
    engine: Engine,
    epoch_increment_stop_signal: Arc<AtomicBool>,
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

        let epoch_increment_stop_signal = Arc::new(AtomicBool::new(false));
        thread::spawn({
            let engine = engine.clone();
            let stop_signal = Arc::clone(&epoch_increment_stop_signal);
            move || loop {
                thread::sleep(Duration::from_micros(10));
                engine.increment_epoch();
                if stop_signal.load(Ordering::SeqCst) {
                    return;
                }
            }
        });

        Self {
            players: Arc::new(players),
            bays: vec![Bay::new()],
            engine,
            epoch_increment_stop_signal,
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
        self.epoch_increment_stop_signal
            .store(true, Ordering::SeqCst);
    }
}

pub struct Player {
    pub network_memory: Arc<Mutex<[u8; NETWORK_MEMORY_SIZE]>>,
    pub script: Module,
}
