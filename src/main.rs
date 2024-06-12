use cursive::{Cursive, CursiveExt};
use rust_of_life::state::game::Game;

fn main() {
    let mut cursive_ref = Cursive::new();
    let screen_size = cursive_ref.screen_size();
    let game = Game::randomized_board(30, 30);

    cursive_ref.add_fullscreen_layer(game);

    cursive_ref.add_global_callback('q', |cur_ref| cur_ref.quit());

    cursive_ref.run();
}

