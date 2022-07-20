use crate::bay::BayExt;
use crate::config::NETWORK_MEMORY_SIZE;
use crate::replay::ReplayRecorder;
use crate::wasm_context::WasmContext;
use botnet_api::{Bay, EntityID};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use wasmtime::Module;

/// High level object for managing an instance of the game server.
pub struct Game<'a> {
    players: HashMap<EntityID, Player>,
    bays: Vec<Bay>,
    next_entity_id: Arc<AtomicU64>,
    wasm_engine: WasmContext<'a>,
    replay_recorder: Option<ReplayRecorder>,
}

pub struct Player {
    pub network_memory: Arc<Mutex<[u8; NETWORK_MEMORY_SIZE]>>,
    pub script: Module,
}

impl Game<'_> {
    pub fn new(record_replay: bool) -> Result<Self, Box<dyn Error>> {
        let wasm_engine = WasmContext::new()?;

        let next_entity_id = Arc::new(AtomicU64::new(0));

        let mut players = HashMap::new();
        let test_player_id = next_entity_id.fetch_add(1, Ordering::SeqCst);
        players.insert(
            test_player_id,
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

        let bays = vec![Bay::new(Arc::clone(&next_entity_id), test_player_id)];

        let replay_recorder = record_replay.then(|| {
            ReplayRecorder::new(
                &bays
                    .iter()
                    .enumerate()
                    .map(|(entity_id, bay)| (entity_id as EntityID, bay))
                    .collect::<Box<[_]>>(),
                next_entity_id.load(Ordering::SeqCst),
            )
        });

        Ok(Self {
            players,
            bays,
            next_entity_id,
            wasm_engine,
            replay_recorder,
        })
    }

    /// Update the game.
    pub fn tick(&mut self) {
        // Record tick start in the replay
        if let Some(replay_recorder) = &self.replay_recorder {
            replay_recorder.record_tick_start();
        }

        // Tick each bay in parallel
        self.bays
            .par_iter_mut()
            .enumerate()
            .for_each(|(bay_id, bay)| {
                bay.tick(
                    bay_id as EntityID,
                    &self.players,
                    Arc::clone(&self.next_entity_id),
                    &self.wasm_engine,
                    self.replay_recorder.as_ref(),
                );
            });
    }
}
