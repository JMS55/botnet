use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use wasmtime::{Config, Engine};

/// A wrapper around [`wasmtime::Engine`] that increments an epoch every 10μs using a background thread.
pub struct WasmEngine {
    pub engine: Engine,
    epoch_increment_stop_signal: Arc<AtomicBool>,
}

impl WasmEngine {
    pub fn new() -> Self {
        let engine = Engine::new(&Config::new().epoch_interruption(true)).unwrap();

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

        Self {
            engine,
            epoch_increment_stop_signal,
        }
    }
}

impl Drop for WasmEngine {
    fn drop(&mut self) {
        self.epoch_increment_stop_signal
            .store(true, Ordering::Relaxed);
    }
}
