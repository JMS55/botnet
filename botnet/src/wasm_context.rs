use crate::bot_actions::*;
use botnet_api::{Bay, EntityID};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use wasmtime::{Config, Engine, Linker, StoreLimits};

pub struct WasmContext<'a> {
    pub engine: Engine,
    pub linker: Linker<StoreData<'a>>,
    epoch_increment_stop_signal: Arc<AtomicBool>,
}

pub struct StoreData<'a> {
    pub limits: StoreLimits,
    pub bot_action: Option<BotAction>,
    pub bot_id: EntityID,
    pub bay: &'a Bay,
}

impl WasmContext<'_> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let engine = Engine::new(&Config::new().epoch_interruption(true)).unwrap();
        let linker = setup_linker(engine.clone())?;

        // Increment the engine epoch every 10μs using a background thread
        let epoch_increment_stop_signal = Arc::new(AtomicBool::new(false));
        thread::spawn({
            let engine = engine.clone();
            let stop_signal = Arc::clone(&epoch_increment_stop_signal);

            move || loop {
                thread::sleep(Duration::from_micros(10));

                engine.increment_epoch();

                if stop_signal.load(Ordering::Relaxed) {
                    return;
                }
            }
        });

        Ok(Self {
            engine,
            epoch_increment_stop_signal,
            linker,
        })
    }
}

impl Drop for WasmContext<'_> {
    fn drop(&mut self) {
        self.epoch_increment_stop_signal
            .store(true, Ordering::Relaxed);
    }
}

fn setup_linker<'a>(engine: Engine) -> Result<Linker<StoreData<'a>>, Box<dyn Error>> {
    let mut linker = Linker::new(&engine);

    export_move_towards(&mut linker)?;
    export_harvest_resource(&mut linker)?;
    export_log_debug(&mut linker)?;

    Ok(linker)
}
