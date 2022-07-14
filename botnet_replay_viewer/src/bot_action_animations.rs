use crate::animation::{Animation, Keyframe};
use crate::bay_renderer::BayRenderer;
use botnet::BotAction;
use botnet_api::{Bay, Bot, Direction};
use macroquad::prelude::{vec2, Color, Vec2};
use std::f32::consts::PI;

pub fn animation_for_bot_action(
    bot_action: &BotAction,
    bot: &Bot,
    bay: &Bay,
    bay_renderer: &BayRenderer,
) -> Animation {
    match bot_action {
        BotAction::MoveTowards(direction) => Animation::new_single_entity(
            bot.id,
            &[
                Keyframe {
                    time: 0.0,
                    ..Default::default()
                },
                Keyframe {
                    time: 0.1,
                    position: Some(match direction {
                        Direction::Up => vec2(bot.x as f32, (bot.y - 1) as f32),
                        Direction::Down => vec2(bot.x as f32, (bot.y + 1) as f32),
                        Direction::Left => vec2((bot.x - 1) as f32, bot.y as f32),
                        Direction::Right => vec2((bot.x + 1) as f32, bot.y as f32),
                    }),
                    rotation: Some(match direction {
                        Direction::Up => 0.0,
                        Direction::Down => PI,
                        Direction::Left => -(PI / 2.0),
                        Direction::Right => PI / 2.0,
                    }),
                    ..Default::default()
                },
            ],
            bay_renderer,
        ),

        BotAction::HarvestResource { x, y } => {
            let resource_id = bay.cells[*x as usize][*y as usize].unwrap();

            let position_offset = ((bot.x as i32 - *x as i32), (bot.y as i32 - *y as i32));
            let rotation = Some(match position_offset {
                (1, 0) => -(PI / 2.0),
                (-1, 0) => PI / 2.0,
                (0, -1) => PI,
                (0, 1) => 0.0,
                _ => unreachable!(),
            });
            let position = Some(
                match position_offset {
                    (1, 0) => vec2(-0.2, 0.0),
                    (-1, 0) => vec2(0.2, 0.0),
                    (0, -1) => vec2(0.0, 0.2),
                    (0, 1) => vec2(0.0, -0.2),
                    _ => unreachable!(),
                } + vec2(bot.x as f32, bot.y as f32),
            );

            Animation::new(
                &[
                    (
                        bot.id,
                        &[
                            Keyframe {
                                time: 0.0,
                                ..Default::default()
                            },
                            Keyframe {
                                time: 0.1,
                                rotation,
                                ..Default::default()
                            },
                            Keyframe {
                                time: 0.2,
                                rotation,
                                position,
                                ..Default::default()
                            },
                            Keyframe {
                                time: 0.5,
                                rotation,
                                ..Default::default()
                            },
                        ],
                    ),
                    (
                        resource_id,
                        &[
                            Keyframe {
                                time: 0.0,
                                ..Default::default()
                            },
                            Keyframe {
                                time: 0.1,
                                ..Default::default()
                            },
                            Keyframe {
                                time: 0.5,
                                scale: Some(Vec2::ZERO),
                                color: Some(Color::from_rgba(96, 96, 96, 0)),
                                ..Default::default()
                            },
                        ],
                    ),
                ],
                bay_renderer,
            )
        }

        BotAction::DepositResource { x, y } => {
            let position_offset = ((bot.x as i32 - *x as i32), (bot.y as i32 - *y as i32));
            let rotation = Some(match position_offset {
                (1, 0) => -(PI / 2.0),
                (-1, 0) => PI / 2.0,
                (0, -1) => PI,
                (0, 1) => 0.0,
                _ => unreachable!(),
            });
            let position = Some(
                match position_offset {
                    (1, 0) => vec2(-0.4, 0.0),
                    (-1, 0) => vec2(0.4, 0.0),
                    (0, -1) => vec2(0.0, 0.4),
                    (0, 1) => vec2(0.0, -0.4),
                    _ => unreachable!(),
                } + vec2(bot.x as f32, bot.y as f32),
            );

            Animation::new_single_entity(
                bot.id,
                &[
                    Keyframe {
                        time: 0.0,
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.1,
                        rotation,
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.2,
                        rotation,
                        position,
                        scale: Some(Vec2::splat(0.3)),
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.3,
                        rotation,
                        ..Default::default()
                    },
                ],
                bay_renderer,
            )
        }

        BotAction::WithdrawResource { x, y, .. } => {
            let position_offset = ((bot.x as i32 - *x as i32), (bot.y as i32 - *y as i32));
            let rotation = Some(match position_offset {
                (-1, 0) => -(PI / 2.0),
                (1, 0) => PI / 2.0,
                (0, 1) => PI,
                (0, -1) => 0.0,
                _ => unreachable!(),
            });
            let position = Some(
                match position_offset {
                    (1, 0) => vec2(0.2, 0.0),
                    (-1, 0) => vec2(-0.2, 0.0),
                    (0, -1) => vec2(0.0, -0.2),
                    (0, 1) => vec2(0.0, 0.2),
                    _ => unreachable!(),
                } + vec2(bot.x as f32, bot.y as f32),
            );

            Animation::new_single_entity(
                bot.id,
                &[
                    Keyframe {
                        time: 0.0,
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.2,
                        rotation,
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.3,
                        rotation,
                        position,
                        scale: Some(Vec2::splat(1.3)),
                        ..Default::default()
                    },
                    Keyframe {
                        time: 0.4,
                        rotation,
                        ..Default::default()
                    },
                ],
                bay_renderer,
            )
        }
    }
}
