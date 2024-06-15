use std::{cell::RefCell, rc::Rc};

use super::cell::{Cell, CellState};
use rand::prelude::*;
use tracing::{info, debug};

#[derive(Clone, Debug)]
pub struct Game {
    pub size_x: isize,
    pub size_y: isize,
    pub cells: Vec<Vec<Option<Cell>>>,
    gen_num: u32,
}

#[derive(Clone)]
pub struct GameRef(pub Rc<RefCell<Game>>);

impl Game {
    pub fn randomized_board(size_x: isize, size_y: isize) -> Self {
        let mut cells = vec![vec![None; size_y as usize]; size_x as usize];
        info!("Creating a randomized board.");
        for x in 0_usize..size_x as usize {
            for y in 0_usize..size_y as usize {
                let cell = Cell::new(x as u32, y as u32);
                cells[x][y] = Some(cell);
                let mut rng = rand::thread_rng();

                if rng.gen::<f64>() > 0.5 {
                    cells[x][y]
                        .as_mut()
                        .map(|inner| inner.state = CellState::Alive);
                }
            }
        }

        return Game { size_x, size_y, cells, gen_num: 0 };
    }

    pub fn tick(&mut self) {
        debug!("Ticking simulation.");
        let cloned_cells = self.cells.clone();

        self.cells.iter_mut().enumerate().for_each(|(i, column)| {
            column.iter_mut().enumerate().for_each(|(j, cell)| {
                let mut alive_count = 0;
                for delta_i in -1_isize..=1 {
                    for delta_j in -1_isize..=1 {
                        let neighbor_i = i as isize + delta_i;
                        let neighbor_j = j as isize + delta_j;

                        if neighbor_i < 0 || neighbor_i >= self.size_x {
                            debug!("X is out of bounds. X: {}", neighbor_i);
                            continue;
                        }
                        if neighbor_j < 0 || neighbor_j >= self.size_y {
                            debug!("Y is out of bounds. Y: {}", neighbor_j);
                            continue;
                        }
                        cloned_cells[neighbor_i as usize][neighbor_j as usize].as_ref().map(|inner| {
                            if let CellState::Alive = inner.state {
                                alive_count += 1;
                            }
                        });
                    }
                }

                debug!("Updating cell state.");

                if let CellState::Alive = cell.as_ref().map_or(&CellState::Dead, |inner| &inner.state) {
                    if alive_count < 2 || alive_count > 3 {
                        cell.as_mut().map(|inner| inner.state = CellState::Dead);
                    }
                } else {
                    if alive_count == 3 {
                        cell.as_mut().map(|inner| inner.state = CellState::Alive);
                    }
                }
            })
        });
    }
}
