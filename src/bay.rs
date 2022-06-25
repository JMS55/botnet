use crate::bot_actions::*;
use crate::compute_bot_action::compute_bot_action;
use crate::game::Player;
use botnet_api::{Bay, Bot, Cell, BAY_SIZE};
use dashmap::DashMap;
use extension_traits::extension;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use wasmtime::Engine;

#[extension(pub trait BayExt)]
impl Bay {
    fn new() -> Self {
        let bot_id = thread_rng().gen();
        let bot = Bot {
            id: bot_id,
            player_id: 1717,
            energy: 1000,
            held_resource: None,
            x: BAY_SIZE / 2,
            y: BAY_SIZE / 2,
        };
        let mut bots = HashMap::new();
        bots.insert(bot_id, bot);

        // TODO: Randomly generate
        let mut cells = [[Cell::Empty; BAY_SIZE]; BAY_SIZE];
        for i in 0..BAY_SIZE {
            cells[i][0] = Cell::Wall;
            cells[i][BAY_SIZE - 1] = Cell::Wall;
            cells[0][i] = Cell::Wall;
            cells[BAY_SIZE - 1][i] = Cell::Wall;
        }
        cells[BAY_SIZE / 2][BAY_SIZE / 2] = Cell::Bot { id: bot_id };

        Self { bots, cells }
    }

    fn tick(&mut self, players: &DashMap<u64, Player>, engine: &Engine) {
        self.tick_bots(players, engine);
    }

    fn tick_bots(&mut self, players: &DashMap<u64, Player>, engine: &Engine) {
        let bot_ids_to_tick = self.bots.keys().copied().collect::<Vec<_>>();
        for bot_id in bot_ids_to_tick {
            if let Some(bot) = self.bots.get(&bot_id) {
                let player = &players.get(&bot.player_id).unwrap();
                if let Ok(bot_action) = compute_bot_action(bot_id, engine, &self, player) {
                    self.apply_bot_action(bot_id, bot_action);
                }
            }
        }
    }

    fn apply_bot_action(&mut self, bot_id: u64, bot_action: BotAction) {
        match bot_action {
            BotAction::MoveTowards(direction) => apply_bot_move_towards(self, bot_id, direction),
        }
    }
}
