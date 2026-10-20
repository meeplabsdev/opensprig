mod colour;
mod error;

pub use anyhow::anyhow as _anyhow;
pub use colour::{Colour, rgb};
pub use error::{DebugError, DisplayError, Error, NonStdError, Result};
