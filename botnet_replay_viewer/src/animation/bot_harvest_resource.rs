use crate::animation::{ease_in_out_quadratic, Animation};
use crate::render::{draw_tile, TILE_SIZE};
use botnet_api::{Bay, BAY_SIZE};
use macroquad::prelude::{
    draw_texture_ex, get_frame_time, rand, vec2, Color, DrawTextureParams, Texture2D,
};
use std::f32::consts::PI;

const ANIMATION_DURATION: f32 = 0.35;

pub struct BotHarvestResourceAnimation {
    x: u32,
    y: u32,
    time_elapsed: f32,
}

impl BotHarvestResourceAnimation {
    pub fn new(x: &u32, y: &u32) -> Box<dyn Animation> {
        Box::new(Self {
            x: *x,
            y: *y,
            time_elapsed: 0.0,
        })
    }
}

impl Animation for BotHarvestResourceAnimation {
    fn draw(&mut self, bay: &Bay, textures: &[Texture2D]) -> bool {
        for x in 0..BAY_SIZE {
            for y in 0..BAY_SIZE {
                if !(x == self.x as usize && y == self.y as usize) {
                    draw_tile(x, y, &bay.cells[x][y], textures);
                }
            }
        }

        rand::srand((self.x * self.y) as u64);
        let size_modifier = rand::gen_range(0.0, 6.0);
        let size = TILE_SIZE as f32 - size_modifier;
        let size = size - (ease_in_out_quadratic(self.time_elapsed / ANIMATION_DURATION) * size);
        let opacity = 255
            - (ease_in_out_quadratic(self.time_elapsed / ANIMATION_DURATION) * 255.0).round() as u8;

        draw_texture_ex(
            textures[1],
            self.x as f32 * TILE_SIZE as f32 + ((TILE_SIZE as f32 - size) / 2.0),
            self.y as f32 * TILE_SIZE as f32 + ((TILE_SIZE as f32 - size) / 2.0),
            Color::from_rgba(96, 96, 96, opacity),
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                rotation: rand::gen_range(0.0, 2.0) * PI,
                ..Default::default()
            },
        );

        self.time_elapsed += get_frame_time();
        self.time_elapsed >= ANIMATION_DURATION
    }
}
