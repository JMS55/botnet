use crate::animation::{Animation, Keyframe};
use crate::bay_renderer::{EntityRenderOverrides, TILE_SIZE};
use botnet::BotAction;
use botnet_api::{Bay, Direction};
use macroquad::prelude::{vec2, Color, Vec2};
use std::f32::consts::PI;

pub fn animation_for_bot_action(bot_action: &BotAction, bot_id: u64, bay: &Bay) -> Animation {
    Animation::new(Box::new(match bot_action {
        BotAction::MoveTowards(direction) => [Keyframe {
            time: 0.35,
            entity_id: bot_id,
            entity_render_overrides: EntityRenderOverrides {
                position_offset: Some(match direction {
                    Direction::Up => vec2(0.0, -(TILE_SIZE as f32)),
                    Direction::Down => vec2(0.0, TILE_SIZE as f32),
                    Direction::Left => vec2(-(TILE_SIZE as f32), 0.0),
                    Direction::Right => vec2(TILE_SIZE as f32, 0.0),
                }),
                rotation: Some(match direction {
                    Direction::Up => 0.0,
                    Direction::Down => PI,
                    Direction::Left => PI / 2.0,
                    Direction::Right => PI * 1.5,
                }),
                ..Default::default()
            },
        }],

        BotAction::HarvestResource { x, y } => [Keyframe {
            time: 0.35,
            entity_id: bay.cells[*x as usize][*y as usize].unwrap(),
            entity_render_overrides: EntityRenderOverrides {
                scale: Some(Vec2::ZERO),
                color: Some(Color::from_rgba(96, 96, 96, 0)),
                ..Default::default()
            },
        }],
    }))
}
