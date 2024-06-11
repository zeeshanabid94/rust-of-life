// An enum to indicate the cell state. The inner value
// of the enum is the number of cells alive around the cell.
#[derive(Debug, Clone)]
pub enum CellState {
    Alive,
    Dead
}

// Represent a single cell on the board.
#[derive(Debug, Clone)]
pub struct Cell {
    state: CellState,
    pos: (u32, u32)
}

impl Cell {
    pub fn new(x: u32, y:u32) -> Cell {
        Cell {
            state: CellState::Dead,
            pos: (x, y)
        }
    }

    pub fn kill(&mut self) {
        self.state = CellState::Dead;
    }

    pub fn reanimate(&mut self) {
        self.state = CellState::Alive;
    }

    pub fn x(self) -> u32 {
        self.pos.0
    }

    pub fn y(self) -> u32 {
        self.pos.1
    }
}
