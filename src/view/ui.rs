use std::cell::RefCell;

use cursive::{
    views::{BoxedView, Button, Canvas, LinearLayout, PaddedView, Panel},
    View,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::state::cell::{Cell, CellState};

const OFFSET_X: usize = 5;
const OFFSET_Y: usize = 5;

pub enum ControlMessages {
    Start,
    Stop,
}

pub struct UserInterface {
    pub root: BoxedView,
}

impl UserInterface {
    pub fn init(
        model_rx: Receiver<Vec<Vec<Option<Cell>>>>,
        controls_tx: Sender<ControlMessages>,
    ) -> Self {
        let canvas = BoxedView::boxed(PaddedView::lrtb(
            OFFSET_X,
            OFFSET_X,
            OFFSET_Y,
            OFFSET_Y,
            Panel::new(
                Canvas::new(RefCell::new(model_rx))
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
                                            CellState::Dead => " ",
                                        },
                                    )
                                });
                            }
                        }
                    })
                    .with_needs_relayout(|state| !state.borrow().is_empty()),
            ),
        ));

        let controls = BoxedView::boxed(LinearLayout::vertical().child(PaddedView::lrtb(
            OFFSET_X,
            OFFSET_X,
            OFFSET_Y,
            OFFSET_Y,
            Button::new("Start", {
                let cloned_tx = controls_tx.clone();
                move |_s| {
                    if let Err(error) = cloned_tx.blocking_send(ControlMessages::Start) {
                        tracing::error!(
                            "Unable to send start message on controls sender channel. {:?}",
                            error
                        );
                    }
                }
            }),
        )));
        let layout = BoxedView::boxed(LinearLayout::horizontal().child(canvas).child(controls));

        Self { root: layout }
    }
}
