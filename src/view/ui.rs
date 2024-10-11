use std::cell::RefCell;

use cursive::{
    view::{Nameable, Resizable},
    views::{BoxedView, Button, Canvas, LinearLayout, PaddedView, Panel},
};
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;

use crate::state::cell::{Cell, CellState};

const OFFSET_X: usize = 5;
const OFFSET_Y: usize = 5;

#[derive(Debug)]
pub enum ControlMessages {
    Start,
    Stop,
}

#[derive(Debug)]
struct UserInterfaceData {
    running: bool,
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
                    .with_required_size(|_state, _screen_size| {
                        // TODO: Figure out a better way to get size
                        cursive::Vec2::new(50, 30)
                    })
                    .with_draw(|state, printer| {
                        let rx = state.borrow_mut();
                        let board = rx.borrow();

                        tracing::debug!("Drawing board.");
                        for cell in board.iter().flatten() {
                            if let Some(inner) = cell.as_ref() { printer.print(
                                    (inner.x(), inner.y()),
                                    match inner.state {
                                        CellState::Alive => "A",
                                        CellState::Dead => " ",
                                    },
                                ) }
                        }
                    }),
            ),
        ));

        let controls = BoxedView::boxed(
            LinearLayout::vertical().child(PaddedView::lrtb(
                OFFSET_X,
                OFFSET_X,
                OFFSET_Y,
                OFFSET_Y,
                Button::new("Start", {
                    let cloned_tx = controls_tx.clone();
                    move |_s| {
                        tracing::info!("Start/Stop button pressed.");
                        if let Some(user_data) = _s.user_data::<UserInterfaceData>() {
                            tracing::info!("Read user data {:?}", user_data);
                            if user_data.running {
                                if let Err(error) = cloned_tx.try_send(ControlMessages::Stop) {
                                    tracing::error!(
                                    "Unable to send stop message on controls sender channel. {:?}",
                                    error
                                );
                                } else {
                                    _s.set_user_data(UserInterfaceData { running: false });
                                }
                            } else if let Err(error) = cloned_tx.try_send(ControlMessages::Start) {
                                tracing::error!(
                                "Unable to send start message on controls sender channel. {:?}",
                                error
                            );
                            } else {
                                _s.set_user_data(UserInterfaceData { running: true });
                            }
                        } else if let Err(error) = cloned_tx.try_send(ControlMessages::Start) {
                            tracing::error!(
                                "Unable to send start message on controls sender channel. {:?}",
                                error
                            );
                        } else {
                            _s.set_user_data(UserInterfaceData { running: true });
                        }

                        if let Some(user_data) = _s.user_data::<UserInterfaceData>() {
                            if user_data.running {
                                _s.call_on_name("Start/Stop", |view: &mut Button| {
                                    view.set_label("Stop");
                                });
                            } else {
                                _s.call_on_name("Start/Stop", |view: &mut Button| {
                                    view.set_label("Start");
                                });
                            }
                        }
                    }
                })
                .with_name("Start/Stop")
                .fixed_width(10),
            )),
        );
        let layout = BoxedView::boxed(LinearLayout::horizontal().child(canvas).child(controls));

        Self { root: layout }
    }
}
