mod animation;
mod bay_renderer;
mod bot_action_animations;

use crate::animation::Animation;
use crate::bay_renderer::{window_conf, BayRenderer};
use botnet::{BayExt, ReplayRecord};
use botnet_api::Bay;
use macroquad::prelude::next_frame;
use rkyv::{Deserialize, Infallible};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[macroquad::main(window_conf)]
async fn main() {
    // Setup replay file
    let replay_path = env::args().nth(1).expect("No replay file provided");
    let mut replay_file = BufReader::new(File::open(replay_path).unwrap());
    let mut record_bytes = Vec::new();

    let mut load_next_record = || -> Option<ReplayRecord> {
        // Read record size
        let mut record_len = [0u8; 8];
        if replay_file.read_exact(&mut record_len).is_err() {
            return None;
        };
        let record_len = u64::from_le_bytes(record_len);

        // Then load a record of that size
        record_bytes.resize(record_len as usize, 0);
        replay_file.read_exact(&mut record_bytes).unwrap();
        Some(
            unsafe { rkyv::archived_root::<ReplayRecord>(&record_bytes) }
                .deserialize(&mut Infallible)
                .unwrap(),
        )
    };

    // Check replay game version
    match load_next_record() {
        Some(ReplayRecord::GameVersion(version)) if &*version == env!("CARGO_PKG_VERSION") => {}
        _ => panic!("This replay is not compatible with this version of BotnetReplayViewer"),
    }

    // Load initial next entity id
    let next_entity_id = Arc::new(AtomicU64::new(match load_next_record() {
        Some(ReplayRecord::InitialNextEntityID(initial_entity_id)) => initial_entity_id,
        _ => unreachable!(),
    }));

    // Load initial bay state
    let mut bay = match load_next_record() {
        Some(ReplayRecord::InitialBayState { bay, .. }) => *bay,
        _ => unreachable!(),
    };

    // Setup bay renderer
    let mut bay_renderer = BayRenderer::new();
    bay_renderer.prepare(&bay);

    // Main loop
    let mut current_record = None;
    loop {
        // Load next record if needed
        if current_record.is_none() {
            current_record = load_next_record();

            if let Some(current_record) = &current_record {
                bay_renderer.animation =
                    Animation::from_replay_record(current_record, &bay, &bay_renderer);
            }
        }

        // Render the bay
        bay_renderer.draw_bay(&bay);

        // Apply record when available and no animation is playing
        if current_record.is_some() && bay_renderer.animation.is_none() {
            apply_record(
                current_record.take().unwrap(),
                &mut bay,
                Arc::clone(&next_entity_id),
            );

            bay_renderer.prepare(&bay);
        }

        next_frame().await
    }
}

fn apply_record(record: ReplayRecord, bay: &mut Bay, next_entity_id: Arc<AtomicU64>) {
    match record {
        ReplayRecord::GameVersion { .. } => unreachable!(),
        ReplayRecord::InitialNextEntityID { .. } => unreachable!(),
        ReplayRecord::InitialBayState { .. } => unreachable!(),
        ReplayRecord::TickStart => {}
        ReplayRecord::BotAction {
            bay_id,
            bot_id,
            bot_action,
        } => {
            bay.apply_bot_action(bay_id, bot_id, bot_action, next_entity_id, None);
        }
        ReplayRecord::RechargeBots { bay_id, bot_ids } => {
            bay.recharge_bots(bay_id, &bot_ids, None);
        }
    }
}
