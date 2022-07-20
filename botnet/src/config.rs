pub const NETWORK_MEMORY_SIZE: usize = 512_000; // 512kb

pub const BOT_TIME_LIMIT: u64 = 100; // ~1ms, depending on scheduler behavior and when increment_epoch() is called
pub const BOT_SETUP_TIME_LIMIT: u64 = 25; // ~0.25ms, depending on scheduler behavior and when increment_epoch() is called
pub const BOT_MEMORY_LIMIT: usize = 2_000_000; // 2mb

pub const INITIAL_BOT_ENERGY: u32 = 100;
pub const BOT_ENERGY_PER_RECHARGE: u32 = 5;
pub const BOT_ACTION_MOVE_TOWARDS_ENERGY_REQUIRED: u32 = 5;
pub const BOT_ACTION_HARVEST_RESOURCE_ENERGY_REQUIRED: u32 = 20;
pub const BOT_ACTION_DEPOSIT_RESOURCE_ENERGY_REQUIRED: u32 = 10;
pub const BOT_ACTION_WIDTHDRAW_RESOURCE_ENERGY_REQUIRED: u32 = 10;
pub const BOT_ACTION_BUILD_ENTITY_ENERGY_REQUIRED: u32 = 15;

pub const RECORDING_QUEUE_MESSAGE_LIMIT: usize = 100;
