use crate::{
    hardwarev2::button::{DEBOUNCE_MS, HOLD_MS},
    types::Result,
};
use embassy_futures::select::{Either, select};
use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
};
use embassy_time::{Instant, Timer};

#[derive(Clone, Copy)]
pub enum ButtonState {
    Released { duration: u64 },
    Pressed,
    Held,
}

pub(crate) struct RawButton<'a> {
    pin: Input<'a>,
    state: ButtonState,
    since: Instant,
}

impl<'a> RawButton<'a> {
    pub fn new(pin: Peri<'a, impl Pin>) -> Result<Self> {
        let mut pin = Input::new(pin, Pull::Up);
        pin.set_inversion(true);

        Ok(Self {
            pin,
            state: ButtonState::Released { duration: 0 },
            since: Instant::now(),
        })
    }

    async fn wait(&mut self) {
        self.pin.wait_for_any_edge().await
    }

    pub fn pressed(&self) -> bool {
        self.pin.is_high()
    }

    pub async fn next(&mut self) -> &ButtonState {
        match self.state {
            ButtonState::Released { duration: _ } => loop {
                // check for press
                self.wait().await;
                Timer::after_millis(DEBOUNCE_MS as u64).await;

                if self.pressed() {
                    // it has been pressed
                    self.state = ButtonState::Pressed;
                    self.since = Instant::now();
                    break;
                }
            },
            ButtonState::Pressed => loop {
                match select(self.wait(), Timer::after_millis(HOLD_MS as u64)).await {
                    Either::First(_) => {
                        Timer::after_millis(DEBOUNCE_MS as u64).await;

                        if !self.pressed() {
                            // it has been released
                            self.state = ButtonState::Released {
                                duration: Instant::now().duration_since(self.since).as_millis(),
                            };
                            self.since = Instant::now();
                            break;
                        }
                    }
                    Either::Second(_) => {
                        // it has been held
                        self.state = ButtonState::Held;
                        break;
                    }
                }
            },
            ButtonState::Held => loop {
                // check for release
                self.wait().await;
                Timer::after_millis(DEBOUNCE_MS as u64).await;

                if !self.pressed() {
                    // it has been released
                    self.state = ButtonState::Released {
                        duration: Instant::now().duration_since(self.since).as_millis(),
                    };
                    self.since = Instant::now();
                    break;
                }
            },
        }

        &self.state
    }
}
