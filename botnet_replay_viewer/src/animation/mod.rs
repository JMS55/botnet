mod bot_harvest_resource;
mod bot_move_towards;

use crate::animation::bot_harvest_resource::BotHarvestResourceAnimation;
use crate::animation::bot_move_towards::BotMoveTowardsAnimation;

use botnet::{BotAction, ReplayRecord};
use botnet_api::Bay;
use macroquad::prelude::Texture2D;

pub trait Animation {
    fn draw(&mut self, bay: &Bay, textures: &[Texture2D]) -> bool;
}

pub fn animation_for_record(record: &ReplayRecord) -> Option<Box<dyn Animation>> {
    match record {
        ReplayRecord::GameVersion(..) => unreachable!(),
        ReplayRecord::InitialBayState { .. } => unreachable!(),
        ReplayRecord::TickStart => None,
        ReplayRecord::BotAction {
            bot_id, bot_action, ..
        } => animation_for_bot_action(bot_action, bot_id),
        ReplayRecord::RechargeBots { .. } => None,
    }
}

fn animation_for_bot_action(bot_action: &BotAction, bot_id: &u64) -> Option<Box<dyn Animation>> {
    match bot_action {
        BotAction::MoveTowards(direction) => Some(BotMoveTowardsAnimation::new(bot_id, direction)),
        BotAction::HarvestResource { x, y } => Some(BotHarvestResourceAnimation::new(x, y)),
    }
}

pub fn ease_in_out_quadratic(t: f32) -> f32 {
    if t < 0.35 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(2.0) / 2.0
    }
}
