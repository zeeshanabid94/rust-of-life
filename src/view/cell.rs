use cursive::{
    event::{Event, EventResult},
    view::IntoBoxedView,
    View, With,
};
use tracing::debug;

use crate::state::{
    cell::CellState,
    game::{Game, GameRef},
};

const OFFSET_X: usize = 5;
const OFFSET_Y: usize = 5;

impl View for GameRef {
    fn draw(&self, printer: &cursive::Printer) {
        debug!("Drawing board.");
        let cloned_self = self.clone();
        let offset_printer = printer.offset((OFFSET_X, OFFSET_Y));
        for cell in cloned_self.0.borrow().cells.iter().flatten() {
            cell.as_ref().map(|inner| {
                offset_printer.print(
                    (inner.x(), inner.y()),
                    match inner.state {
                        CellState::Alive => "A",
                        CellState::Dead => "_",
                    },
                )
            });
        }
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        // Take up as much as the board size + 2 times the padding of x and y
        cursive::Vec2::new(self.0.borrow().size_x.unsigned_abs() + OFFSET_X * 2, self.0.borrow().size_y.unsigned_abs() + OFFSET_Y * 2)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        self.0.borrow_mut().tick();
        let cloned_self = self.clone();
        match event {
            Event::Refresh => {
                EventResult::with_cb(move |cur| cur.add_layer(cloned_self.to_owned()))
            }
            _ => EventResult::Ignored,
        }
    }
}
