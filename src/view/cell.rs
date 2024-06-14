use cursive::{
    event::{Event, EventResult},
    view::IntoBoxedView,
    View, With,
};

use crate::state::{
    cell::CellState,
    game::{Game, GameRef},
};

impl View for GameRef {
    fn draw(&self, printer: &cursive::Printer) {
        let offset_printer = printer.offset((5, 5));
        for cell in self.0.borrow().cells.iter().flatten() {
            cell.as_ref().map(|inner| {
                offset_printer.print(
                    (inner.x(), inner.y()),
                    match inner.state {
                        CellState::Alive => "0",
                        CellState::Dead => "-",
                    },
                )
            });
        }
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        // Take up max size
        constraint
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
