use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::view::ui::ControlMessages;

use super::cell::{Cell, CellState};
use rand::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::Sender;
use tracing::{debug, info};

const TICK_RATE_PER_SECOND: f64 = 30.0;
type Board = Vec<Vec<Option<Cell>>>;

#[derive(Debug, Clone, Default)]
pub struct GameData {
    pub running: bool,
    pub cells: Board,
    pub previousGeneration: Board
}

#[derive(Debug)]
pub struct Game {
    pub size_x: isize,
    pub size_y: isize,
    sender: Option<Sender<GameData>>,
    control_rx: Option<Receiver<ControlMessages>>,
    game_data: Box<GameData>
}

#[derive(Clone)]
pub struct GameRef(pub Rc<RefCell<Game>>);

impl Game {
    pub fn with_sender(mut self, tx: Sender<GameData>) -> Self {
        self.sender = Some(tx);

        self
    }

    pub fn with_control_rx(mut self, rx: Receiver<ControlMessages>) -> Self {
        self.control_rx = Some(rx);

        self
    }

    fn randomize(&mut self) {
        let mut cells = vec![vec![None; self.size_y as usize]; self.size_x as usize];
        info!("Creating a randomized board.");
        for x in 0..self.size_x as usize {
            for y in 0..self.size_y as usize {
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

        // Overwrite existing cell data
        self.game_data.cells = cells;
    }

    pub fn randomized_board(size_x: isize, size_y: isize) -> Self {
        let mut cells = vec![vec![None; size_y as usize]; size_x as usize];
        info!("Creating a randomized board.");
        for x in 0_usize..size_x as usize {
            for y in 0_usize..size_y as usize {
                let cell = Cell::new(x as u32, y as u32);
                cells[x][y] = Some(cell);
            }
        }

        let mut init = Game {
            size_x,
            size_y,
            sender: None,
            control_rx: None,
            game_data: Box::new(GameData {
                running: false,
                cells: cells.clone(),
                previousGeneration: cells
            })
        };

        init.randomize();

        init
    }

    pub async fn start(mut self) {
        if let Some(sender) = self.sender.clone() {
            let _ = sender.send(*self.game_data.clone());
        }

        loop {
            if let Some(sender) = self.sender.clone() {
                let _ = sender.send(*self.game_data.clone());
            }
            if self.game_data.running {
                tracing::debug!("Simulation running");
                let tick_time: f64 = (1.0 / TICK_RATE_PER_SECOND) * 1000.0;
                tokio::time::sleep(Duration::from_millis(tick_time as u64)).await;
                self.tick();
            }

            if let Some(controls_tx) = self.control_rx.as_mut() {
                let control_message = controls_tx.try_recv();

                if let Ok(control_message) = control_message {
                    tracing::info!("Control message received: {:?}", control_message);
                    match control_message {
                        ControlMessages::Stop => self.game_data.running = false,
                        ControlMessages::Start => self.game_data.running = true,
                        ControlMessages::Reset => self.reset(),
                        ControlMessages::Step => self.tick(),
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        self.game_data.running = false; // Stop running
        self.randomize();
    }

    fn tick(&mut self) {
        debug!("Ticking simulation.");
        self.game_data.previousGeneration = self.game_data.cells.clone();
        let cloned_cells = self.game_data.cells.clone();

        self.game_data.cells.iter_mut().enumerate().for_each(|(i, column)| {
            column.iter_mut().enumerate().for_each(|(j, cell)| {
                let mut alive_count = 0;
                for delta_i in -1_isize..=1 {
                    for delta_j in -1_isize..=1 {
                        // Don't count the cell itself
                        if delta_i == 0 && delta_j == 0 {
                            continue;
                        }
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
                        cloned_cells[neighbor_i as usize][neighbor_j as usize]
                            .as_ref()
                            .map(|inner| {
                                if let CellState::Alive = inner.state {
                                    alive_count += 1;
                                }
                            });
                    }
                }

                debug!("Alive count for cell at x: {i} y: {j} is {alive_count}");
                debug!("Updating cell state.");

                if let CellState::Alive =
                    cell.as_ref().map_or(&CellState::Dead, |inner| &inner.state)
                {
                    if alive_count < 2 || alive_count > 3 {
                        cell.as_mut().map(|inner| inner.state = CellState::Dead);
                    }
                } else if alive_count == 3 {
                    cell.as_mut().map(|inner| inner.state = CellState::Alive);
                }
            })
        });
    }
}
