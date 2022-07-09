use crate::bot_actions::*;
use crate::compute_bot_action::compute_bot_action;
use crate::game::Player;
use crate::replay::ReplayRecorder;
use crate::wasm_context::WasmContext;
use botnet_api::{Bay, Bot, Entity, EntityID, Resource, BAY_SIZE};
use extension_traits::extension;
use log::info;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Methods for creating and updating a [`botnet_api::Bay`].
#[extension(pub trait BayExt)]
impl Bay {
    fn new(next_entity_id: Arc<AtomicU64>, test_player_id: EntityID) -> Self {
        let mut entities = HashMap::new();

        let mut cells = [[None; BAY_SIZE]; BAY_SIZE];
        for i in 0..BAY_SIZE {
            let entity_id1 = next_entity_id.fetch_add(1, Ordering::SeqCst);
            let entity_id2 = next_entity_id.fetch_add(1, Ordering::SeqCst);
            let entity_id3 = next_entity_id.fetch_add(1, Ordering::SeqCst);
            let entity_id4 = next_entity_id.fetch_add(1, Ordering::SeqCst);

            entities.insert(entity_id1, Entity::Wall);
            entities.insert(entity_id2, Entity::Wall);
            entities.insert(entity_id3, Entity::Wall);
            entities.insert(entity_id4, Entity::Wall);

            cells[i][0] = Some(entity_id1);
            cells[i][BAY_SIZE - 1] = Some(entity_id2);
            cells[0][i] = Some(entity_id3);
            cells[BAY_SIZE - 1][i] = Some(entity_id4);
        }

        let mut rng = thread_rng();

        for _ in 0..12 {
            loop {
                let (x, y) = (rng.gen_range(0..BAY_SIZE), rng.gen_range(0..BAY_SIZE));
                if cells[x][y] == None {
                    let entity_id = next_entity_id.fetch_add(1, Ordering::SeqCst);

                    entities.insert(
                        entity_id,
                        Entity::Bot(Bot {
                            entity_id,
                            player_id: test_player_id,
                            energy: 1000,
                            held_resource: None,
                            x,
                            y,
                        }),
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

                    entities.insert(entity_id, Entity::Resource(Resource::Silicon));

                    cells[x][y] = Some(entity_id);

                    break;
                }
            }
        }

        Self { entities, cells }
    }

    /// Update the bay.
    fn tick(
        &mut self,
        bay_id: EntityID,
        players: &HashMap<EntityID, Player>,
        wasm_context: &WasmContext,
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        info!("Bay[{bay_id}] starting tick");
        let bot_ids = self.get_bot_ids();

        self.tick_bots(bay_id, &bot_ids, players, wasm_context, replay_recorder);
        self.recharge_bots(bay_id, &bot_ids, replay_recorder);
    }

    /// Compute and apply an action for each bot.
    fn tick_bots(
        &mut self,
        bay_id: EntityID,
        bot_ids: &[EntityID],
        players: &HashMap<EntityID, Player>,
        wasm_context: &WasmContext,
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        for bot_id in bot_ids {
            if let Some(bot) = self.get_bot(*bot_id) {
                let player = &players.get(&bot.player_id).unwrap();

                match compute_bot_action(*bot_id, &self, player, wasm_context) {
                    Ok(bot_action) => {
                        info!("Bot[{bot_id}] chose action {:?}", bot_action);

                        self.apply_bot_action(bay_id, *bot_id, bot_action, replay_recorder);
                    }
                    result => {
                        info!("Bot[{bot_id}] did not choose an action: {:?}", result);
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
                bot.energy += 5;
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
        replay_recorder: Option<&ReplayRecorder>,
    ) {
        match bot_action {
            BotAction::MoveTowards(direction) => apply_bot_move_towards(self, bot_id, direction),
            BotAction::HarvestResource { x, y } => apply_bot_harvest_resource(self, bot_id, x, y),
        }

        if let Some(replay_recorder) = replay_recorder {
            replay_recorder.record_bot_action(bay_id, bot_id, bot_action);
        }
    }
}
