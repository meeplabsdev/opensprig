mod button;
mod raw;

pub use button::Button;
pub use raw::ButtonState;

const DEBOUNCE_MS: usize = 10;
const HOLD_MS: usize = 800;
