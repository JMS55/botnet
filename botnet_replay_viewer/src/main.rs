mod animation;
mod render;

use crate::animation::{animation_for_record, Animation};
use crate::render::{draw_bay, window_conf};
use botnet::{BayExt, ReplayRecord};
use botnet_api::Bay;
use macroquad::prelude::{clear_background, next_frame, Color, Texture2D};
use rkyv::{Deserialize, Infallible};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

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

    // Load initial bay state
    let mut bay = match load_next_record() {
        Some(ReplayRecord::InitialBayState { bay, .. }) => *bay,
        _ => unreachable!(),
    };

    // Load textures
    let bot_texture =
        Texture2D::from_file_with_format(include_bytes!("../assets/ship_E.png"), None);
    let resource_texture = Texture2D::from_file_with_format(
        include_bytes!("../assets/meteor_detailedLarge.png"),
        None,
    );
    let textures = Box::new([bot_texture, resource_texture]);

    // Main loop
    let mut current_record = None;
    let mut current_animation: Option<Box<dyn Animation>> = None;
    loop {
        // Load next record if needed
        if current_record.is_none() {
            current_record = load_next_record();
            if let Some(current_record) = &current_record {
                current_animation = animation_for_record(current_record);
            }
        }

        // Apply record when available and no animation is playing
        if current_record.is_some() && current_animation.is_none() {
            apply_record(current_record.take().unwrap(), &mut bay);
        }

        // Render the bay
        clear_background(Color::from_rgba(24, 25, 22, 255));
        match current_animation.as_deref_mut() {
            // Delegate render to an animation
            Some(animation) => {
                let finished = animation.draw(&bay, &*textures);
                if finished {
                    current_animation = None;
                }
            }
            // Render the bay state exactly
            None => {
                draw_bay(&bay, &*textures);
            }
        }

        next_frame().await
    }
}

fn apply_record(record: ReplayRecord, bay: &mut Bay) {
    match record {
        ReplayRecord::GameVersion(..) => unreachable!(),
        ReplayRecord::InitialBayState { .. } => unreachable!(),
        ReplayRecord::TickStart => {}
        ReplayRecord::BotAction {
            bay_id,
            bot_id,
            bot_action,
        } => {
            bay.apply_bot_action(bay_id, bot_id, bot_action, None);
        }
        ReplayRecord::RechargeBots { bay_id, bot_ids } => {
            bay.recharge_bots(bay_id, &bot_ids, None);
        }
    }
}
