use std::cell::RefCell;

use cursive::{
    Cursive, event::Event, theme::ColorStyle, view::{Nameable, Resizable}, views::{BoxedView, Button, Canvas, LinearLayout, PaddedView, Panel}
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
    Step
}

#[derive(Debug, Clone)]
struct UserInterfaceData {
    running: bool,
}

pub struct UserInterface {
    pub root: BoxedView,
}

impl UserInterface {
    pub fn init(model_rx: Receiver<GameData>, controls_tx: Sender<ControlMessages>, cursiveRef: &mut Cursive) -> Self {
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
                        let previous_board = board.previousGeneration.clone().into_iter().flatten();
                        let next_board = board.cells.clone().into_iter().flatten();
                        let zipped_boards = next_board.zip(previous_board);
                        tracing::debug!("Drawing board.");
                        for (new_cell, old_cell) in zipped_boards {
                            if let (Some(inner), Some(old_inner)) = (new_cell.as_ref(), old_cell.as_ref()) {
                                let char_to_print = match (&inner.state, &old_inner.state) {
                                    (CellState::Alive, CellState::Alive) => '█',
                                    (CellState::Alive, CellState::Dead) => '▓',
                                    (CellState::Dead, CellState::Alive) => '▒',
                                    (CellState::Dead, CellState::Dead) => ' ',
                                };
                                
                                printer.print(
                                    (inner.x(), inner.y()),
                                    &char_to_print.to_string()
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
                    Button::new("Reset", UserInterface::reset_button_callback(controls_tx.clone()))
                        .with_name("Reset")
                        .fixed_width(10),
                ))
                .child(PaddedView::lrtb(
                    OFFSET_X,
                    OFFSET_X,
                    OFFSET_Y,
                    OFFSET_Y,
                    Button::new("Print Board", UserInterface::print_board_state_callback(model_rx.clone()))
                        .with_name("Print Board")
                        .fixed_width(10),
                ))
                .child(PaddedView::lrtb(
                    OFFSET_X,
                    OFFSET_X,
                    OFFSET_Y,
                    OFFSET_Y,
                    Button::new("Step", UserInterface::step_simulation(controls_tx.clone()))
                        .with_name("Step")
                        .fixed_width(10),
                )),


        );
        let layout = BoxedView::boxed(LinearLayout::horizontal().child(canvas).child(controls));


        let receiver_cloned = model_rx.clone();
        cursiveRef.set_on_pre_event(Event::Refresh, move |cursive: &mut Cursive| {
            let game_state = receiver_cloned.borrow();
            cursive.call_on_name("Start/Stop", |view: &mut Button| {
                view.set_label( if game_state.running { "Stop" } else { "Start" });
            });
        });

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
                    }
                } else {
                    if let Err(error) = cloned_tx.try_send(ControlMessages::Start) {
                        tracing::error!(
                            "Unable to send start message on controls sender channel. {:?}",
                            error
                        );
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

    fn step_simulation(
        controls_tx: Sender<ControlMessages>
    ) -> Box<dyn 'static + Fn(&mut Cursive)> {
        return {
            let cloned_tx = controls_tx.clone();

            Box::new(move |_s: &mut Cursive| {
                tracing::info!("Step simulation button pressed.");
                let send_result = cloned_tx.try_send(ControlMessages::Step);
                if let Err(error) = send_result {
                    tracing::error!("Error sending control message step. {error}");
                }
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
