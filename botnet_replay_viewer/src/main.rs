use botnet::bay::BayExt;
use botnet::replay::ReplayRecord;
use botnet_api::{Bay, Cell, BAY_SIZE};
use macroquad::prelude::*;
use rkyv::{Deserialize, Infallible};
use std::env;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::{Duration, Instant};

#[macroquad::main(window_conf)]
async fn main() {
    // Setup replay
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
    let mut last_record_end = Instant::now();
    let mut next_record = None;
    loop {
        // Load next record if needed
        if next_record.is_none() {
            next_record = load_next_record();
        }

        // Apply record when available and it's been long enough since the last one
        if next_record.is_some() && last_record_end.elapsed() >= Duration::from_millis(300) {
            match next_record.take().unwrap() {
                ReplayRecord::GameVersion(..) => unreachable!(),
                ReplayRecord::InitialBayState { .. } => unreachable!(),
                ReplayRecord::TickStart => {}
                ReplayRecord::BotAction {
                    bay_id,
                    bot_id,
                    bot_action,
                } => {
                    bay.apply_bot_action(bay_id, bot_id, bot_action, None);
                    last_record_end = Instant::now();
                }
                ReplayRecord::RechargeBots { bay_id, bot_ids } => {
                    bay.recharge_bots(bay_id, &bot_ids, None);
                    last_record_end = Instant::now();
                }
            }
        }

        // Render the bay
        clear_background(Color::from_rgba(24, 25, 22, 255));
        for x in 0..BAY_SIZE {
            for y in 0..BAY_SIZE {
                draw_tile(x, y, &bay, &*textures);
            }
        }

        next_frame().await
    }
}

fn draw_tile(x: usize, y: usize, bay: &Bay, textures: &[Texture2D]) {
    match bay.cells[x][y] {
        Cell::Empty => draw_circle(
            x as f32 * 32.0 + 16.0,
            y as f32 * 32.0 + 16.0,
            2.0,
            Color::from_rgba(44, 45, 42, 255),
        ),
        Cell::Wall => draw_rectangle_lines(
            x as f32 * 32.0 + 8.0,
            y as f32 * 32.0 + 8.0,
            16.0,
            16.0,
            4.0,
            Color::from_rgba(44, 45, 42, 255),
        ),
        Cell::Resource(..) => {
            rand::srand((x * y) as u64);
            let size_modifier = rand::gen_range(0.0, 6.0);
            draw_texture_ex(
                textures[1],
                x as f32 * 32.0 + (size_modifier / 2.0),
                y as f32 * 32.0 + (size_modifier / 2.0),
                Color::from_rgba(96, 96, 96, 255),
                DrawTextureParams {
                    dest_size: Some(vec2(32.0 - size_modifier, 32.0 - size_modifier)),
                    rotation: rand::gen_range(0.0, 2.0) * PI,
                    ..Default::default()
                },
            );
        }
        Cell::Interconnect { .. } => todo!(),
        Cell::Antenna { .. } => todo!(),
        Cell::Bot { .. } => draw_texture_ex(
            textures[0],
            x as f32 * 32.0 + 4.0,
            y as f32 * 32.0 + 4.0,
            Color::from_rgba(190, 190, 190, 255),
            DrawTextureParams {
                dest_size: Some(vec2(24.0, 24.0)),
                ..Default::default()
            },
        ),
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "BotnetReplayViewer".to_owned(),
        window_width: BAY_SIZE as i32 * 32,
        window_height: BAY_SIZE as i32 * 32,
        window_resizable: false,
        ..Default::default()
    }
}
