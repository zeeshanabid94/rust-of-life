use std::cell::RefCell;

use cursive::{
    view::{Nameable, Resizable},
    views::{BoxedView, Button, Canvas, LinearLayout, PaddedView, Panel},
    Cursive,
};
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tracing::{debug, info};

use crate::state::{
    cell::CellState,
    game::GameData,
};

const OFFSET_X: usize = 5;
const OFFSET_Y: usize = 5;

#[derive(Debug)]
pub enum ControlMessages {
    Reset,
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
    pub fn init(model_rx: Receiver<GameData>, controls_tx: Sender<ControlMessages>) -> Self {
        let canvas = BoxedView::boxed(PaddedView::lrtb(
            OFFSET_X,
            OFFSET_X,
            OFFSET_Y,
            OFFSET_Y,
            Panel::new(
                Canvas::new(RefCell::new(model_rx.clone()))
                    .with_required_size(|_state, _screen_size| {
                        // TODO: Figure out a better way to get size
                        cursive::Vec2::new(50, 30)
                    })
                    .with_draw(|state, printer| {
                        let rx = state.borrow_mut();
                        let board = rx.borrow();

                        tracing::debug!("Drawing board.");
                        for cell in board.cells.iter().flatten() {
                            if let Some(inner) = cell.as_ref() {
                                printer.print(
                                    (inner.x(), inner.y()),
                                    match inner.state {
                                        CellState::Alive => "o",
                                        CellState::Dead => "+",
                                    },
                                )
                            }
                        }
                    }),
            ),
        ));

        let controls = BoxedView::boxed(
            LinearLayout::vertical()
                .child(PaddedView::lrtb(
                    OFFSET_X,
                    OFFSET_X,
                    OFFSET_Y,
                    OFFSET_Y,
                    Button::new(
                        "Start",
                        UserInterface::start_button_callback(controls_tx.clone(), model_rx.clone()),
                    )
                    .with_name("Start/Stop")
                    .fixed_width(10),
                ))
                .child(PaddedView::lrtb(
                    OFFSET_X,
                    OFFSET_X,
                    OFFSET_Y,
                    OFFSET_Y,
                    Button::new("Reset", UserInterface::reset_button_callback(controls_tx))
                        .with_name("Reset")
                        .fixed_width(10),
                ))
                .child(PaddedView::lrtb(
                    OFFSET_X,
                    OFFSET_X,
                    OFFSET_Y,
                    OFFSET_Y,
                    Button::new("Print Board State", UserInterface::print_board_state_callback(model_rx.clone()))
                        .with_name("Reset")
                        .fixed_width(10),
                )),

        );
        let layout = BoxedView::boxed(LinearLayout::horizontal().child(canvas).child(controls));

        Self { root: layout }
    }

    fn start_button_callback(
        controls_tx: Sender<ControlMessages>,
        model_rx: Receiver<GameData>,
    ) -> Box<dyn 'static + Fn(&mut Cursive)> {
        return {
            let cloned_tx = controls_tx.clone();
            let cloned_rx = model_rx.clone();
            Box::new(move |_s: &mut Cursive| {
                tracing::info!("Start/Stop button pressed.");
                let model_state = cloned_rx.borrow();
                if model_state.running {
                    if let Err(error) = cloned_tx.try_send(ControlMessages::Stop) {
                        tracing::error!(
                            "Unable to send stop message on controls sender channel. {:?}",
                            error
                        );
                    } else {
                        tracing::info!("Stop message sent. Changing Button Label to Start");
                        _s.call_on_name("Start/Stop", |view: &mut Button| {
                            view.set_label("Start");
                        });
                    }
                } else {
                    if let Err(error) = cloned_tx.try_send(ControlMessages::Start) {
                        tracing::error!(
                            "Unable to send start message on controls sender channel. {:?}",
                            error
                        );
                    } else {
                        tracing::info!("Start message sent. Changing Button Label to Stop.");
                        _s.call_on_name("Start/Stop", |view: &mut Button| {
                            view.set_label("Stop");
                        });
                    }
                }
            })
        };
    }

    fn print_board_state_callback(
        model_rx: Receiver<GameData>
    ) -> Box<dyn 'static + Fn(&mut Cursive)> {
        return {
            let cloned_rx = model_rx.clone();
            Box::new(move |_s: &mut Cursive| {
                tracing::info!("Print board state button pressed.");
                let model_state = cloned_rx.borrow();
                info!("Board state {model_state:?}")
            })
        }
    }

    fn reset_button_callback(
        controls_tx: Sender<ControlMessages>,
    ) -> Box<dyn 'static + Fn(&mut Cursive)> {
        return {
            let cloned_tx = controls_tx.clone();
            Box::new(move |_s: &mut Cursive| {
                tracing::info!("Reset button pressed.");
                if let Some(user_data) = _s.user_data::<UserInterfaceData>() {
                    tracing::info!("Read user data {:?}", user_data);
                    if user_data.running {
                        if let Err(error) = cloned_tx.try_send(ControlMessages::Reset) {
                            tracing::error!(
                                "Unable to send reset message on controls sender channel. {:?}",
                                error
                            );
                        } else {
                            _s.set_user_data(UserInterfaceData { running: false });
                        }
                    } else if let Err(error) = cloned_tx.try_send(ControlMessages::Reset) {
                        tracing::error!(
                            "Unable to send reset message on controls sender channel. {:?}",
                            error
                        );
                    } else {
                        _s.set_user_data(UserInterfaceData { running: true });
                    }
                } else if let Err(error) = cloned_tx.try_send(ControlMessages::Reset) {
                    tracing::error!(
                        "Unable to send reset message on controls sender channel. {:?}",
                        error
                    );
                } else {
                    _s.set_user_data(UserInterfaceData { running: true });
                }
            })
        };
    }
}
