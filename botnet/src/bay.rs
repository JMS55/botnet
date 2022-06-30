use crate::bot_actions::*;
use crate::compute_bot_action::compute_bot_action;
use crate::game::Player;
use crate::wasm_context::WasmContext;
use botnet_api::{Bay, Bot, Cell, Resource, BAY_SIZE};
use extension_traits::extension;
use log::info;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

/// Methods for creating and updating a [`botnet_api::Bay`].
#[extension(pub trait BayExt)]
impl Bay {
    fn new() -> Self {
        let mut bots = HashMap::new();
        bots.insert(
            22,
            Bot {
                id: 22,
                player_id: 1717,
                energy: 1000,
                held_resource: None,
                x: BAY_SIZE / 2,
                y: BAY_SIZE / 2,
            },
        );

        let mut cells = [[Cell::Empty; BAY_SIZE]; BAY_SIZE];
        for i in 0..BAY_SIZE {
            cells[i][0] = Cell::Wall;
            cells[i][BAY_SIZE - 1] = Cell::Wall;
            cells[0][i] = Cell::Wall;
            cells[BAY_SIZE - 1][i] = Cell::Wall;
        }

        cells[BAY_SIZE / 2][BAY_SIZE / 2] = Cell::Bot { id: 22 };

        let mut rng = thread_rng();
        for _ in 0..100 {
            loop {
                let (x, y): (usize, usize) =
                    (rng.gen_range(0..BAY_SIZE), rng.gen_range(0..BAY_SIZE));
                if cells[x][y] == Cell::Empty {
                    cells[x][y] = Cell::Resource(Resource::Silicon);
                    break;
                }
            }
        }

        Self { bots, cells }
    }

    /// Update the bay.
    fn tick(&mut self, players: &HashMap<u64, Player>, wasm_context: &WasmContext) {
        let bots_ticked = self.tick_bots(players, wasm_context);
        self.recharge_bots(&bots_ticked);
    }

    /// Compute and apply an action for each bot.
    fn tick_bots(
        &mut self,
        players: &HashMap<u64, Player>,
        wasm_context: &WasmContext,
    ) -> Vec<u64> {
        let bot_ids_to_tick = self.bots.keys().copied().collect::<Vec<_>>();

        for bot_id in &bot_ids_to_tick {
            if let Some(bot) = self.bots.get(&bot_id) {
                let player = &players.get(&bot.player_id).unwrap();

                match compute_bot_action(*bot_id, &self, player, wasm_context) {
                    Ok(bot_action) => {
                        info!("Bot[{bot_id}] chose action {:?}", bot_action);

                        self.apply_bot_action(*bot_id, bot_action);
                    }
                    result => {
                        info!("Bot[{bot_id}] did not choose an action: {:?}", result);
                    }
                }
            }
        }

        bot_ids_to_tick
    }

    /// Add some energy back to each bot.
    fn recharge_bots(&mut self, bot_ids: &[u64]) {
        for bot_id in bot_ids {
            if let Some(bot) = self.bots.get_mut(bot_id) {
                bot.energy += 5;
            }
        }
    }

    fn apply_bot_action(&mut self, bot_id: u64, bot_action: BotAction) {
        match bot_action {
            BotAction::MoveTowards(direction) => apply_bot_move_towards(self, bot_id, direction),
            BotAction::HarvestResource { x, y } => apply_bot_harvest_resource(self, bot_id, x, y),
        }
    }
}
