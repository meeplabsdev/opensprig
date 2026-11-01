use crate::{
    hardwarev2::button::{ButtonState, raw::RawButton},
    types::Result,
};
use embassy_rp::{Peri, gpio::Pin};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub struct Button<'a> {
    raw: Mutex<CriticalSectionRawMutex, RawButton<'a>>,
}

impl<'a> Button<'a> {
    pub fn new(pin: Peri<'a, impl Pin>) -> Result<Self> {
        let raw = RawButton::new(pin)?;

        Ok(Self {
            raw: Mutex::new(raw),
        })
    }

    pub async fn is_down(&self) -> bool {
        self.raw.lock().await.pressed()
    }

    pub async fn next(&self) -> ButtonState {
        *self.raw.lock().await.next().await
    }
}
