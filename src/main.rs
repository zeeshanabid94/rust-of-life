use std::{cell::RefCell, rc::Rc};

use cursive::{event, views::DebugView, Cursive, CursiveExt};
use rust_of_life::state::game::{Game, GameRef};

fn main() {
    let mut cursive_ref = Cursive::new();
    let game = Game::randomized_board(125, 30);

    cursive_ref.add_layer(GameRef(Rc::new(RefCell::new(game))));
    cursive_ref.set_window_title("Rust of Life");

    cursive_ref.add_global_callback('~', Cursive::toggle_debug_console);
    cursive_ref.add_global_callback('q', |cur_ref| cur_ref.quit());
    cursive_ref.set_autorefresh(true);
   
    cursive_ref.run();
}

