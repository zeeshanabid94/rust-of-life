use super::cell::{Cell, CellState};
use rand::prelude::*;

pub struct Game {
    pub cells: Vec<Vec<Option<Cell>>>,
    gen_num: u32
}

impl Game {
    pub fn randomized_board(size_x: usize, size_y: usize) -> Self {
        let mut cells = vec![vec![None; size_x];size_y];
        for i in 0..size_x {
            for j in 0..size_y {
                let cell = Cell::new(i as u32, j as u32);
                cells[i][j] = Some(cell);
                let mut rng = rand::thread_rng();

                if rng.gen::<f64>() > 0.5 {
                    cells[i][j].as_mut().map(|inner| inner.state = CellState::Alive);
                }
            }
            
        }
        
        return Game {
            cells,
            gen_num: 0
        }
    }
}
