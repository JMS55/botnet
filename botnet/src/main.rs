use botnet::game::Game;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    let mut game = Game::new(true).unwrap();
    let t = Instant::now();
    loop {
        game.tick();
        if t.elapsed() > Duration::from_secs(1) {
            return;
        }
        thread::yield_now();
    }
}
