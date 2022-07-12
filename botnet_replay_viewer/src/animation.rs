use crate::bay_renderer::EntityRenderOverrides;
use crate::bot_action_animations::animation_for_bot_action;
use botnet::ReplayRecord;
use botnet_api::{Bay, EntityID};
use macroquad::prelude::{Color, Vec2};
use std::collections::{HashMap, HashSet};
use std::ops::{Mul, Sub};
use std::time::Instant;

pub struct Animation {
    keyframes: Box<[Keyframe]>,
    start_time: Instant,
}

pub struct Keyframe {
    pub time: f32,
    pub entity_id: EntityID,
    pub entity_render_overrides: EntityRenderOverrides,
}

impl Animation {
    pub fn new(keyframes: Box<[Keyframe]>) -> Self {
        Self {
            keyframes,
            start_time: Instant::now(),
        }
    }

    pub fn from_record(record: &ReplayRecord, bay: &Bay) -> Option<Self> {
        match record {
            ReplayRecord::GameVersion(..) => unreachable!(),
            ReplayRecord::InitialBayState { .. } => unreachable!(),
            ReplayRecord::TickStart => None,
            ReplayRecord::BotAction {
                bot_id, bot_action, ..
            } => Some(animation_for_bot_action(bot_action, *bot_id, bay)),
            ReplayRecord::RechargeBots { .. } => None,
        }
    }

    pub fn tick(
        &mut self,
        entity_render_overrides: &mut HashMap<EntityID, EntityRenderOverrides>,
    ) -> bool {
        let elapsed_time = self.start_time.elapsed().as_secs_f32();
        let end_time = self.keyframes.last().unwrap().time;
        let finished = elapsed_time >= end_time;

        // Find the target keyframe for each entity in the animation
        let mut entity_found_keyframe = HashSet::new();
        let mut keyframe_indices = Vec::with_capacity(self.keyframes.len());
        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time >= elapsed_time && !entity_found_keyframe.contains(&keyframe.entity_id)
            {
                keyframe_indices.push(i);
                entity_found_keyframe.insert(keyframe.entity_id);
            }
        }

        // Tween between each entity's current EntityRenderOverrides and the one in the keyframe
        let ease = ease_in_out_quadratic(elapsed_time / end_time);
        for keyframe_index in keyframe_indices {
            let keyframe = &self.keyframes[keyframe_index];
            let entity_render_overrides = entity_render_overrides
                .entry(keyframe.entity_id)
                .or_default();

            tween(
                &mut entity_render_overrides.position_offset,
                keyframe.entity_render_overrides.position_offset,
                Vec2::ZERO,
                ease,
            );

            tween(
                &mut entity_render_overrides.rotation,
                keyframe.entity_render_overrides.rotation,
                0.0,
                ease,
            );

            tween(
                &mut entity_render_overrides.scale,
                keyframe.entity_render_overrides.scale,
                Vec2::ONE,
                ease,
            );

            // TODO
            // let mut ero_color = entity_render_overrides.color.as_ref().map(Color::to_vec);
            // let keyframe_color = keyframe
            //     .entity_render_overrides
            //     .color
            //     .as_ref()
            //     .map(Color::to_vec);
            // tween(&mut ero_color, keyframe_color, todo!(), ease);
            // entity_render_overrides.color = ero_color.map(Color::from_vec);
        }

        finished
    }
}

fn tween<T>(current: &mut Option<T>, target: Option<T>, default: T, ease: f32)
where
    T: PartialEq + Copy + Sub<Output = T> + Mul<f32, Output = T>,
{
    if *current != target {
        let c = current.unwrap_or(default);
        let t = target.unwrap_or(default);
        *current = Some((t - c) * ease);
    }
}

fn ease_in_out_quadratic(t: f32) -> f32 {
    if t < 0.35 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(2.0) / 2.0
    }
}
