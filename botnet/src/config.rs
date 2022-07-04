pub const NETWORK_MEMORY_SIZE: usize = 512_000; // 512kb

pub const BOT_TIME_LIMIT: u64 = 100; // ~1ms, depending on scheduler behavior and when increment_epoch() is called
pub const BOT_SETUP_TIME_LIMIT: u64 = 25; // ~0.25ms, depending on scheduler behavior and when increment_epoch() is called
pub const BOT_MEMORY_LIMIT: usize = 2_000_000; // 2mb

pub const RECORDING_QUEUE_MESSAGE_LIMIT: usize = 100;
