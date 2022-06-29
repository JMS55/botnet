mod bay;
mod bot_actions;
mod compute_bot_action;
mod config;
mod game;
mod wasm_engine;

use crate::game::Game;
use std::thread;

fn main() {
    env_logger::init();

    let mut game = Game::new();
    loop {
        game.tick();
        thread::yield_now();
    }
}
