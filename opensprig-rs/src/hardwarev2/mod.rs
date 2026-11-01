mod button;
mod screen;

// TODO: this will move into the Sprig struct eventually
pub use button::{Button, ButtonState};
pub use screen::Screen;

use crate::types::Result;
use embassy_rp::spi;

pub struct Sprig<'a, S>
where
    S: spi::Instance,
{
    screen: Option<Screen<'a, S>>,
}

impl<S> Sprig<'_, S>
where
    S: spi::Instance,
{
    pub async fn new() -> Result<Self> {
        Ok(Self { screen: None })
    }

    pub fn screen(self) -> Result<Self> {
        // TODO

        Ok(self)
    }
}
