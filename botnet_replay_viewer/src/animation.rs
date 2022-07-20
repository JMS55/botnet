use crate::bay_renderer::{BayRenderer, EntityRenderParameters};
use crate::bot_action_animations::animation_for_bot_action;
use botnet::ReplayRecord;
use botnet_api::{Bay, EntityID};
use macroquad::prelude::{Color, Vec2};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Animation {
    entity_keyframes: Box<[EntityKeyframes]>,
    start_time: Instant,
    duration: Duration,
}

struct EntityKeyframes {
    entity_id: EntityID,
    keyframes: Box<[Keyframe]>,
    target_keyframe: usize,
}

#[derive(Default)]
pub struct Keyframe {
    pub time: f32,
    pub position: Option<Vec2>,
    pub rotation: Option<f32>,
    pub scale: Option<Vec2>,
    pub color: Option<Color>,
}

impl Animation {
    pub fn from_replay_record(
        record: &ReplayRecord,
        bay: &Bay,
        bay_renderer: &BayRenderer,
    ) -> Option<Self> {
        match record {
            ReplayRecord::GameVersion(..) => unreachable!(),
            ReplayRecord::InitialNextEntityID { .. } => unreachable!(),
            ReplayRecord::InitialBayState { .. } => unreachable!(),
            ReplayRecord::TickStart => None,
            ReplayRecord::BotAction {
                bot_id, bot_action, ..
            } => Some(animation_for_bot_action(
                bot_action,
                bay.get_bot(*bot_id).unwrap(),
                bay,
                bay_renderer,
            )),
            ReplayRecord::RechargeBots { .. } => None,
        }
    }

    pub fn new(keyframes: &[(EntityID, &[Keyframe])], bay_renderer: &BayRenderer) -> Self {
        let duration = keyframes
            .iter()
            .map(|(_, keyframes)| keyframes.last().unwrap().time)
            .max_by(f32::total_cmp)
            .unwrap();

        let keyframes = keyframes
            .iter()
            .map(|(entity_id, keyframes)| EntityKeyframes {
                entity_id: *entity_id,
                keyframes: fill_keyframes_for_entity(*entity_id, keyframes, bay_renderer),
                target_keyframe: 1,
            })
            .collect();

        Self {
            entity_keyframes: keyframes,
            start_time: Instant::now(),
            duration: Duration::from_secs_f32(duration),
        }
    }

    pub fn new_single_entity(
        entity_id: EntityID,
        keyframes: &[Keyframe],
        bay_renderer: &BayRenderer,
    ) -> Self {
        let duration = keyframes.last().unwrap().time;

        let keyframes = Box::new([EntityKeyframes {
            entity_id,
            keyframes: fill_keyframes_for_entity(entity_id, keyframes, bay_renderer),
            target_keyframe: 1,
        }]);

        Self {
            entity_keyframes: keyframes,
            start_time: Instant::now(),
            duration: Duration::from_secs_f32(duration),
        }
    }

    pub fn tick(
        &mut self,
        entity_render_parameters: &mut HashMap<EntityID, EntityRenderParameters>,
    ) -> bool {
        let elapsed_time = self.start_time.elapsed();
        let finished = elapsed_time >= self.duration;

        let elapsed_time = elapsed_time.as_secs_f32();
        for entity_keyframes in self.entity_keyframes.iter_mut() {
            if entity_keyframes.target_keyframe == entity_keyframes.keyframes.len() {
                continue;
            }

            if entity_keyframes.keyframes[entity_keyframes.target_keyframe].time < elapsed_time {
                entity_keyframes.target_keyframe += 1;
            }

            if entity_keyframes.target_keyframe == entity_keyframes.keyframes.len() {
                continue;
            }

            let previous_keyframe =
                &entity_keyframes.keyframes[entity_keyframes.target_keyframe - 1];
            let target_keyframe = &entity_keyframes.keyframes[entity_keyframes.target_keyframe];

            let ease = ease_in_out_quadratic(
                (elapsed_time - previous_keyframe.time)
                    / (target_keyframe.time - previous_keyframe.time),
            );

            let entity_render_parameters = entity_render_parameters
                .get_mut(&entity_keyframes.entity_id)
                .unwrap();

            entity_render_parameters.position = previous_keyframe.position.unwrap()
                + (target_keyframe.position.unwrap() - previous_keyframe.position.unwrap()) * ease;
            entity_render_parameters.rotation = previous_keyframe.rotation.unwrap()
                + (target_keyframe.rotation.unwrap() - previous_keyframe.rotation.unwrap()) * ease;
            entity_render_parameters.scale = previous_keyframe.scale.unwrap()
                + (target_keyframe.scale.unwrap() - previous_keyframe.scale.unwrap()) * ease;
            entity_render_parameters.color = Color::from_vec(
                previous_keyframe.color.unwrap().to_vec()
                    + (target_keyframe.color.unwrap().to_vec()
                        - previous_keyframe.color.unwrap().to_vec())
                        * ease,
            );
        }

        finished
    }
}

fn fill_keyframes_for_entity(
    entity_id: EntityID,
    keyframes: &[Keyframe],
    bay_renderer: &BayRenderer,
) -> Box<[Keyframe]> {
    keyframes
        .iter()
        .map(|keyframe| {
            let current = bay_renderer
                .entity_render_parameters
                .get(&entity_id)
                .unwrap();

            Keyframe {
                time: keyframe.time,
                position: keyframe.position.or(Some(current.position)),
                rotation: keyframe.rotation.or(Some(current.rotation)),
                scale: keyframe.scale.or(Some(current.scale)),
                color: keyframe.color.or(Some(current.color)),
            }
        })
        .collect()
}

fn ease_in_out_quadratic(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(2.0) / 2.0
    }
}
