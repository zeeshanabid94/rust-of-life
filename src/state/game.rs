use std::{cell::RefCell, rc::Rc};

use super::cell::{Cell, CellState};
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct Game {
    pub size_x: usize,
    pub size_y: usize,
    pub cells: Vec<Vec<Option<Cell>>>,
    gen_num: u32,
}

#[derive(Clone)]
pub struct GameRef(pub Rc<RefCell<Game>>);

impl Game {
    pub fn randomized_board(size_x: usize, size_y: usize) -> Self {
        let mut cells = vec![vec![None; size_y]; size_x];
        for x in 0..size_x {
            for y in 0..size_y {
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
        let cloned_cells = self.cells.clone();

        self.cells.iter_mut().enumerate().for_each(|(i, column)| {
            column.iter_mut().enumerate().for_each(|(j, cell)| {
                let mut alive_count = 0;
                for delta_i in -1_i8..=1 {
                    for delta_j in -1_i8..=1 {
                        let neighbor_i = i as i8 + delta_i;
                        let neighbor_j = j as i8 + delta_j;

                        if neighbor_i < 0 || neighbor_i >= self.size_x as i8 || neighbor_j < 0 || neighbor_j >= self.size_y as i8 {
                            continue;
                        }
                        cloned_cells[neighbor_i as usize][neighbor_j as usize].as_ref().map(|inner| {
                            if let CellState::Alive = inner.state {
                                alive_count += 1;
                            }
                        });
                    }
                }

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
