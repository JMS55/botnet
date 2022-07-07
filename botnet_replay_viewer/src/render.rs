use botnet_api::{Bay, Cell, BAY_SIZE};
use macroquad::prelude::{
    draw_circle, draw_circle_lines, draw_texture_ex, rand, vec2, Color, Conf, DrawTextureParams,
    Texture2D,
};
use std::f32::consts::PI;

pub const TILE_SIZE: u32 = 32;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "BotnetReplayViewer".to_owned(),
        window_width: BAY_SIZE as i32 * TILE_SIZE as i32,
        window_height: BAY_SIZE as i32 * TILE_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

pub fn draw_bay(bay: &Bay, textures: &[Texture2D]) {
    for x in 0..BAY_SIZE {
        for y in 0..BAY_SIZE {
            draw_tile(x, y, &bay.cells[x][y], textures);
        }
    }
}

pub fn draw_tile(x: usize, y: usize, cell: &Cell, textures: &[Texture2D]) {
    match cell {
        Cell::Empty => draw_circle(
            x as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
            y as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
            2.0,
            Color::from_rgba(44, 45, 42, 255),
        ),
        Cell::Wall => draw_circle_lines(
            x as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
            y as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
            6.0,
            2.0,
            Color::from_rgba(44, 45, 42, 255),
        ),
        Cell::Resource(..) => {
            rand::srand((x * y) as u64);
            let size_modifier = rand::gen_range(0.0, 6.0);
            draw_texture_ex(
                textures[1],
                x as f32 * TILE_SIZE as f32 + (size_modifier / 2.0),
                y as f32 * TILE_SIZE as f32 + (size_modifier / 2.0),
                Color::from_rgba(96, 96, 96, 255),
                DrawTextureParams {
                    dest_size: Some(vec2(
                        TILE_SIZE as f32 - size_modifier,
                        TILE_SIZE as f32 - size_modifier,
                    )),
                    rotation: rand::gen_range(0.0, 2.0) * PI,
                    ..Default::default()
                },
            );
        }
        Cell::Interconnect { .. } => todo!(),
        Cell::Antenna { .. } => todo!(),
        Cell::Bot { .. } => draw_texture_ex(
            textures[0],
            x as f32 * TILE_SIZE as f32 + 4.0,
            y as f32 * TILE_SIZE as f32 + 4.0,
            Color::from_rgba(180, 180, 180, 255),
            DrawTextureParams {
                dest_size: Some(vec2((TILE_SIZE - 8) as f32, (TILE_SIZE - 8) as f32)),
                ..Default::default()
            },
        ),
    }
}
