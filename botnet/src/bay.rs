use crate::bot_actions::*;
use crate::compute_bot_action::compute_bot_action;
use crate::config::{BOT_ENERGY_PER_RECHARGE, INITIAL_BOT_ENERGY};
use crate::game::Player;
use crate::replay::ReplayRecorder;
use crate::wasm_context::WasmContext;
use botnet_api::{Bay, Bot, Entity, EntityID, Resource, BAY_SIZE};
use extension_traits::extension;
use log::{info, trace, warn};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Methods for creating and updating a [`botnet_api::Bay`].
#[extension(pub trait BayExt)]
impl Bay {
    fn new(next_entity_id: &AtomicU64, test_player_id: EntityID) -> Self {
        let mut entities = HashMap::new();
        let mut cells = [[None; BAY_SIZE]; BAY_SIZE];
        let mut rng = thread_rng();

        for _ in 0..12 {
            loop {
                let (x, y) = (rng.gen_range(0..BAY_SIZE), rng.gen_range(0..BAY_SIZE));
                if cells[x][y] == None {
                    let entity_id = next_entity_id.fetch_add(1, Ordering::SeqCst);
                    entities.insert(
                        entity_id,
                        (
                            Entity::Bot(Bot {
                                id: entity_id,
                                controller_id: test_player_id,
                                energy: INITIAL_BOT_ENERGY,
                                held_resource: None,
                                x,
                                y,
                            }),
                            x as u32,
                            y as u32,
                        ),
                    );
                    cells[x][y] = Some(entity_id);
                    break;
                }
            }
        }

        for _ in 0..30 {
            loop {
                let (x, y) = (rng.gen_range(0..BAY_SIZE), rng.gen_range(0..BAY_SIZE));
                if cells[x][y] == None {
                    let entity_id = next_entity_id.fetch_add(1, Ordering::SeqCst);
                    entities.insert(
                        entity_id,
                        (
                            Entity::Resource(match rng.gen_range(0..4) {
                                0 => Resource::Copper,
                                1 => Resource::Gold,
                                2 => Resource::Silicon,
                                3 => Resource::Plastic,
                                _ => unreachable!(),
                            }),
                            x as u32,
                            y as u32,
                        ),
                    );
                    cells[x][y] = Some(entity_id);
                    break;
                }
            }
        }

        Self {
            entities,
            cells,
            controller_id: None,
        }
    }

    /// Update the bay.
    fn tick(
        &mut self,
        bay_id: EntityID,
        players: &HashMap<EntityID, Player>,
        next_entity_id: &AtomicU64,
        wasm_context: &WasmContext,
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        info!("Bay[{bay_id}] starting tick");
        let bot_ids = self.get_bot_ids();

        self.tick_bots(
            bay_id,
            &bot_ids,
            players,
            next_entity_id,
            wasm_context,
            replay_recorder,
        );
        self.recharge_bots(bay_id, &bot_ids, replay_recorder);
    }

    /// Compute and apply an action for each bot.
    fn tick_bots(
        &mut self,
        bay_id: EntityID,
        bot_ids: &[EntityID],
        players: &HashMap<EntityID, Player>,
        next_entity_id: &AtomicU64,
        wasm_context: &WasmContext,
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        for bot_id in bot_ids {
            if let Some(bot) = self.get_bot(*bot_id) {
                let player = &players.get(&bot.controller_id).unwrap();

                match compute_bot_action(*bot_id, &self, player, wasm_context) {
                    Ok((bot_action, script_duration)) => {
                        trace!(
                            "Bot[{bot_id}] chose action {:?} after {:?}",
                            bot_action,
                            script_duration
                        );

                        self.apply_bot_action(
                            bay_id,
                            *bot_id,
                            bot_action,
                            next_entity_id,
                            replay_recorder,
                        );
                    }
                    result => {
                        warn!("Bot[{bot_id}] did not choose an action: {:?}", result);
                    }
                }
            }
        }
    }

    /// Add some energy back to each bot.
    fn recharge_bots(
        &mut self,
        bay_id: EntityID,
        bot_ids: &[EntityID],
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        for bot_id in bot_ids {
            if let Some(bot) = self.get_bot_mut(*bot_id) {
                bot.energy += BOT_ENERGY_PER_RECHARGE;
            }
        }

        if let Some(replay_recorder) = replay_recorder {
            replay_recorder.record_recharge_bots(bay_id, bot_ids);
        }
    }

    fn apply_bot_action(
        &mut self,
        bay_id: EntityID,
        bot_id: EntityID,
        bot_action: BotAction,
        next_entity_id: &AtomicU64,
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        match bot_action {
            BotAction::MoveTowards(direction) => apply_bot_move_towards(self, bot_id, direction),
            BotAction::HarvestResource { x, y } => apply_bot_harvest_resource(self, bot_id, x, y),
            BotAction::DepositResource { x, y } => apply_bot_deposit_resource(self, bot_id, x, y),
            BotAction::WithdrawResource { resource, x, y } => {
                apply_bot_withdraw_resource(self, bot_id, resource, x, y)
            }
            BotAction::BuildEntity { entity_type, x, y } => {
                apply_bot_build_entity(self, bot_id, entity_type, x, y, next_entity_id)
            }
        }

        if let Some(replay_recorder) = replay_recorder {
            replay_recorder.record_bot_action(bay_id, bot_id, bot_action);
        }
    }
}
