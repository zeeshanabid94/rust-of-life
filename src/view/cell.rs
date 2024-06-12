use cursive::View;

use crate::state::{cell::CellState, game::Game};

impl View for Game {
    fn draw(&self, printer: &cursive::Printer) {
        for cell in self.cells.iter().flatten() {
            cell.as_ref().map(|inner| printer.print((inner.x(), inner.y()),
                match inner.state {
                    CellState::Alive => "0",
                    CellState::Dead => "X"
                }));
        }
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        // Take up max size
        constraint
    }
}
