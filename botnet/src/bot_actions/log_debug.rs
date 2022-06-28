use crate::compute_bot_action::StoreData;
use std::error::Error;
use wasmtime::{Caller, Linker};

pub fn export_log_debug(linker: &mut Linker<StoreData>) -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    let log_debug = |mut caller: Caller<StoreData>, pointer: u32, length: u32| {
        use log::debug;
        use wasmtime::Extern;

        let mut message_bytes = vec![0; length as usize];
        caller
            .get_export("memory")
            .map(Extern::into_memory)
            .flatten()
            .map(|memory| memory.read(&mut caller, pointer as usize, &mut message_bytes))
            .map(Result::ok)
            .expect("Failed to read debug message from bot memory");
        let message =
            String::from_utf8(message_bytes).expect("Failed to read valid debug message from bot");

        debug!(
            "Bot[{}] logged message: \"{}\"",
            caller.data().bot_id,
            message
        );
    };

    #[cfg(not(debug_assertions))]
    let log_debug = |_: Caller<StoreData>, _: u32, _: u32| {};

    linker.func_wrap("env", "__log_debug", log_debug)?;
    Ok(())
}
