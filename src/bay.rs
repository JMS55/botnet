use crate::bot::BotExt;
use crate::game::Player;
use botnet_api::{Bay, Bot, BotAction, Cell, Direction, BAY_SIZE};
use dashmap::DashMap;
use extension_traits::extension;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;
use wasmtime::Engine;

#[extension(pub trait BayExt)]
impl Bay {
    fn new() -> Self {
        let bot_id = thread_rng().gen();
        let bot = Bot {
            player_id: 1717,
            held_resource: None,
        };
        let mut bots = HashMap::new();
        bots.insert(bot_id, (bot, (BAY_SIZE / 2) as u8, (BAY_SIZE / 2) as u8));

        let mut cells: [[Cell; BAY_SIZE]; BAY_SIZE] = Default::default();
        for i in 0..BAY_SIZE {
            cells[i][0] = Cell::Wall;
            cells[i][BAY_SIZE - 1] = Cell::Wall;
            cells[0][i] = Cell::Wall;
            cells[BAY_SIZE - 1][i] = Cell::Wall;
        }
        cells[BAY_SIZE / 2][BAY_SIZE / 2] = Cell::Bot { id: bot_id };

        Self { bots, cells }
    }

    fn tick(&mut self, players: Arc<DashMap<u64, Player>>, engine: &Engine) {
        self.tick_bots(players, engine);
    }

    fn tick_bots(&mut self, players: Arc<DashMap<u64, Player>>, engine: &Engine) {
        let bot_ids_to_tick = self.bots.keys().copied().collect::<Vec<_>>();
        for bot_id in bot_ids_to_tick {
            if let Some((bot, bot_x, bot_y)) = self.bots.get(&bot_id) {
                let bot_action = bot
                    .compute_action(bot_id, engine, &players, &self)
                    .unwrap_or(BotAction::None);
                self.apply_bot_action(bot_action, bot_id, *bot_x, *bot_y);
            }
        }
    }

    fn apply_bot_action(&mut self, bot_action: BotAction, bot_id: u64, bot_x: u8, bot_y: u8) {
        match bot_action {
            BotAction::None => {}
            BotAction::Move(direction) => self.apply_bot_move(direction, bot_id, bot_x, bot_y),
        }
    }

    fn apply_bot_move(&mut self, direction: Direction, bot_id: u64, bot_x: u8, bot_y: u8) {
        let (target_x, target_y) = {
            let (bot_x, bot_y) = (bot_x as isize, bot_y as isize);
            let (target_x, target_y) = match direction {
                Direction::Up => (bot_x, bot_y + 1),
                Direction::Down => (bot_x, bot_y - 1),
                Direction::Left => (bot_x - 1, bot_y),
                Direction::Right => (bot_x + 1, bot_y),
            };
            let bay_bounds = 0..(BAY_SIZE as isize);
            if !(bay_bounds.contains(&target_x) && bay_bounds.contains(&target_y)) {
                return;
            }
            (target_x as usize, target_y as usize)
        };

        if self.cells[target_x][target_y] == Cell::Empty {
            self.cells[target_x][target_y] =
                mem::take(&mut self.cells[bot_x as usize][bot_y as usize]);
            let (_, bot_x, bot_y) = self.bots.get_mut(&bot_id).unwrap();
            (*bot_x, *bot_y) = (target_x as u8, target_y as u8);
        }
    }
}
