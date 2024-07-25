use std::{cell::RefCell, rc::Rc};

use cursive::{event, views::DebugView, Cursive, CursiveExt};
use rust_of_life::state::game::{Game, GameRef};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_appender::rolling::{RollingFileAppender, Rotation};


fn main() {
    // Create a rolling file appender that rotates logs every hour and writes to ./logs/my_log.log
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, "./logs", "rust-of-life.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Use the tracing_subscriber crate to consume the logs and pipe them to the file
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting rust of life!");

    let mut cursive_ref = Cursive::new();
    let game = Game::randomized_board(200, 30);

    cursive_ref.add_layer(GameRef(Rc::new(RefCell::new(game))));
    cursive_ref.set_window_title("Rust of Life");

    cursive_ref.add_global_callback('~', Cursive::toggle_debug_console);
    cursive_ref.add_global_callback('q', |cur_ref| cur_ref.quit());
    cursive_ref.set_autorefresh(true);
   
    cursive_ref.run();
}

