use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::view::ui::ControlMessages;

use super::cell::{Cell, CellState};
use rand::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::Sender;
use tracing::{debug, info};

const TICK_RATE_PER_SECOND: f64 = 5.0;

#[derive(Debug)]
pub struct Game {
    pub size_x: isize,
    pub size_y: isize,
    pub cells: Vec<Vec<Option<Cell>>>,
    sender: Option<Sender<Vec<Vec<Option<Cell>>>>>,
    control_rx: Option<Receiver<ControlMessages>>,
    gen_num: u32,
    running: bool,
}

#[derive(Clone)]
pub struct GameRef(pub Rc<RefCell<Game>>);

impl Game {
    pub fn with_sender(mut self, tx: Sender<Vec<Vec<Option<Cell>>>>) -> Self {
        self.sender = Some(tx);

        self
    }

    pub fn with_control_rx(mut self, rx: Receiver<ControlMessages>) -> Self {
        self.control_rx = Some(rx);

        self
    }

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

        return Game {
            size_x,
            size_y,
            cells,
            gen_num: 0,
            sender: None,
            control_rx: None,
            running: false,
        };
    }

    pub async fn start(mut self) {
        let cloned_cells = self.cells.clone();

        if let Some(sender) = self.sender.clone() {
            let _ = sender.send(cloned_cells);
        }

        loop {
            let cloned_cells = self.cells.clone();

            if let Some(sender) = self.sender.clone() {
                let _ = sender.send(cloned_cells);
            }
            if self.running {
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
                        ControlMessages::Stop => self.running = false,
                        ControlMessages::Start => self.running = true,
                    }
                }
            }
        }
    }

    fn tick(&mut self) {
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
                        cloned_cells[neighbor_i as usize][neighbor_j as usize]
                            .as_ref()
                            .map(|inner| {
                                if let CellState::Alive = inner.state {
                                    alive_count += 1;
                                }
                            });
                    }
                }

                debug!("Updating cell state.");

                if let CellState::Alive =
                    cell.as_ref().map_or(&CellState::Dead, |inner| &inner.state)
                {
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
