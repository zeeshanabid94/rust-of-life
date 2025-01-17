
use cursive::{
    Cursive, CursiveExt,
};
use rust_of_life::{
    state::{
        cell::Cell,
        game::{Game, GameData},
    },
    view::ui::{ControlMessages, UserInterface},
};
use tracing::{info, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::FmtSubscriber;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    // Create a rolling file appender that rotates logs every hour and writes to ./logs/my_log.log
    let file_appender = RollingFileAppender::new(Rotation::MINUTELY, "./logs", "rust-of-life.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Use the tracing_subscriber crate to consume the logs and pipe them to the file
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting rust of life!");
    let (tx, rx) = tokio::sync::watch::channel::<GameData>(GameData::default());
    let (controls_tx, controls_rx) = tokio::sync::mpsc::channel::<ControlMessages>(100);

    let mut cursive_ref = Cursive::new();
    let game = Game::randomized_board(50, 30)
        .with_sender(tx)
        .with_control_rx(controls_rx);

    tokio::spawn(async move {
        tracing::info!("Starting game simulation.");
        game.start().await;
    });

    cursive_ref.add_layer(UserInterface::init(rx, controls_tx).root);

    // cursive_ref.add_layer(GameRef(Rc::new(RefCell::new(game))));
    cursive_ref.set_window_title("Rust of Life");

    cursive_ref.add_global_callback('~', Cursive::toggle_debug_console);
    cursive_ref.add_global_callback('q', |cur_ref| cur_ref.quit());
    cursive_ref.set_autorefresh(true);
    cursive_ref.set_fps(10);

    cursive_ref.run();
}
