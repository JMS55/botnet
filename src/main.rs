mod bay;
mod bot_compute_action;
mod game;

use game::Game;
use std::thread;

fn main() {
    let mut game = Game::new();
    loop {
        game.tick();
        thread::yield_now();
    }
}
