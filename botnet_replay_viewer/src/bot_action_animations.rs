use crate::animation::{Animation, Keyframe};
use crate::bay_renderer::BayRenderer;
use botnet::BotAction;
use botnet_api::{Bay, Direction};
use macroquad::prelude::{vec2, Color, Vec2};
use std::f32::consts::PI;

pub fn animation_for_bot_action(
    bot_action: &BotAction,
    bot_id: u64,
    bay: &Bay,
    bay_renderer: &BayRenderer,
) -> Animation {
    match bot_action {
        BotAction::MoveTowards(direction) => {
            let bot = bay.get_bot(bot_id).unwrap();
            Animation::from_keyframes_single_entity(
                bot_id,
                &[
                    Keyframe {
                        time: 0.0,
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.35,
                        position: Some(match direction {
                            Direction::Up => vec2(bot.x as f32, (bot.y - 1) as f32),
                            Direction::Down => vec2(bot.x as f32, (bot.y + 1) as f32),
                            Direction::Left => vec2((bot.x - 1) as f32, bot.y as f32),
                            Direction::Right => vec2((bot.x + 1) as f32, bot.y as f32),
                        }),
                        rotation: Some(match direction {
                            Direction::Up => 0.0,
                            Direction::Down => PI,
                            Direction::Left => PI / 2.0,
                            Direction::Right => PI * 1.5,
                        }),
                        ..Default::default()
                    },
                ],
                bay_renderer,
            )
        }

        BotAction::HarvestResource { x, y } => {
            let resource_id = bay.cells[*x as usize][*y as usize].unwrap();
            Animation::from_keyframes_single_entity(
                resource_id,
                &[
                    Keyframe {
                        time: 0.0,
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.35,
                        scale: Some(Vec2::ZERO),
                        color: Some(Color::from_rgba(96, 96, 96, 0)),
                        ..Default::default()
                    },
                ],
                bay_renderer,
            )
        }
    }
}
