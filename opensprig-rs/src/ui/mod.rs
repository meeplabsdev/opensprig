use crate::types::Colour;

const COLOUR_BG: Colour = Colour::new(115, 115, 115);
const COLOUR_HOVER: Colour = Colour::new(140, 140, 140);
const COLOUR_PRESS: Colour = Colour::new(230, 230, 230);
const COLOUR_TEXT: Colour = Colour::new(255, 255, 255);

mod keyboard;
mod list;

pub use keyboard::Keyboard;
pub use list::List;
