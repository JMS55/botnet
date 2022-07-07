use crate::animation::{ease_in_out_quadratic, Animation};
use crate::render::{draw_tile, TILE_SIZE};
use botnet_api::{Bay, Cell, Direction, BAY_SIZE};
use macroquad::prelude::{
    draw_texture_ex, get_frame_time, vec2, Color, DrawTextureParams, Texture2D,
};

const ANIMATION_DURATION: f32 = 0.05;

pub struct BotMoveTowardsAnimation {
    bot_id: u64,
    move_direction: Direction,
    time_elapsed: f32,
}

impl BotMoveTowardsAnimation {
    pub fn new(bot_id: &u64, move_direction: &Direction) -> Box<dyn Animation> {
        Box::new(Self {
            bot_id: *bot_id,
            move_direction: *move_direction,
            time_elapsed: 0.0,
        })
    }
}

impl Animation for BotMoveTowardsAnimation {
    fn draw(&mut self, bay: &Bay, textures: &[Texture2D]) -> bool {
        let bot = bay.bots.get(&self.bot_id).unwrap();

        for x in 0..BAY_SIZE {
            for y in 0..BAY_SIZE {
                if x == bot.x && y == bot.y {
                } else {
                    draw_tile(x, y, &bay.cells[x][y], textures);
                }
            }
        }
        draw_tile(bot.x, bot.y, &Cell::Empty, textures);

        let x_offset = match self.move_direction {
            Direction::Left => -(TILE_SIZE as f32),
            Direction::Right => TILE_SIZE as f32,
            _ => 0.0,
        } * ease_in_out_quadratic(self.time_elapsed / ANIMATION_DURATION);
        let y_offset = match self.move_direction {
            Direction::Up => -(TILE_SIZE as f32),
            Direction::Down => TILE_SIZE as f32,
            _ => 0.0,
        } * ease_in_out_quadratic(self.time_elapsed / ANIMATION_DURATION);

        draw_texture_ex(
            textures[0],
            bot.x as f32 * TILE_SIZE as f32 + 4.0 + x_offset,
            bot.y as f32 * TILE_SIZE as f32 + 4.0 + y_offset,
            Color::from_rgba(180, 180, 180, 255),
            DrawTextureParams {
                dest_size: Some(vec2((TILE_SIZE - 8) as f32, (TILE_SIZE - 8) as f32)),
                ..Default::default()
            },
        );

        self.time_elapsed += get_frame_time();
        self.time_elapsed >= ANIMATION_DURATION
    }
}
