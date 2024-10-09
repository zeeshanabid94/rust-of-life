use std::{cell::RefCell, rc::Rc, str::FromStr};

use cursive::{event, views::{Button, Canvas, DebugView, FixedLayout, LinearLayout, PaddedView, Panel}, Cursive, CursiveExt, Rect};
use rust_of_life::state::{cell::{Cell, CellState}, game::{Game, GameRef}};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

const OFFSET_X: usize = 5;
const OFFSET_Y: usize = 5;

#[tokio::main]
async fn main() {
    // Create a rolling file appender that rotates logs every hour and writes to ./logs/my_log.log
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, "./logs", "rust-of-life.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Use the tracing_subscriber crate to consume the logs and pipe them to the file
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting rust of life!");
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<Vec<Option<Cell>>>>(100);

    let mut cursive_ref = Cursive::new();
    let game = Game::randomized_board(50, 30).with_sender(tx);

    tokio::spawn(async move {
        tracing::info!("Starting game simulation.");
        game.start().await;
    });
    let canvas = PaddedView::lrtb(OFFSET_X, OFFSET_X, OFFSET_Y, OFFSET_Y, 
        Panel::new(Canvas::new(RefCell::new(rx))
        .with_required_size(|state, screen_size| {
            // TODO: Figure out a better way to get size
            cursive::Vec2::new(50, 30)
        })
        .with_draw(|state, printer| {
            let board = state.borrow_mut().try_recv();

            if let Ok(board) = board {
                tracing::debug!("Drawing board.");
                let cloned_self = state.clone();
                for cell in board.iter().flatten() {
                    cell.as_ref().map(|inner| {
                        printer.print(
                            (inner.x(), inner.y()),
                            match inner.state {
                                CellState::Alive => "A",
                                CellState::Dead => "_",
                            },
                        )
                    });
                }
            }
        })
        .with_needs_relayout(|state| {
            !state.borrow().is_empty()
        })));
    let controls = LinearLayout::vertical()
        .child(PaddedView::lrtb(
                OFFSET_X, OFFSET_X, OFFSET_Y, OFFSET_Y, 
                Button::new("Start", |s| s.quit()
                    )
                )
            );
    let layout = LinearLayout::horizontal()
        .child(canvas)
        .child(controls);

    cursive_ref.add_layer(layout);

    // cursive_ref.add_layer(GameRef(Rc::new(RefCell::new(game))));
    cursive_ref.set_window_title("Rust of Life");

    cursive_ref.add_global_callback('~', Cursive::toggle_debug_console);
    cursive_ref.add_global_callback('q', |cur_ref| cur_ref.quit());
    cursive_ref.set_autorefresh(true);
    cursive_ref.set_fps(10);
   
    cursive_ref.run();
}

