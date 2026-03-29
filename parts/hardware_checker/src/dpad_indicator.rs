use embassy_rp::peripherals::SPI0;
use opensprig_rs::{
    hardware::Screen,
    types::{Colour, Error},
};

pub struct DPadIndicator {
    x: usize,
    y: usize,
    up_pressed: bool,
    right_pressed: bool,
    down_pressed: bool,
    left_pressed: bool,
    light_color: Colour,
    neutral_color: Colour,
}

impl DPadIndicator {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            up_pressed: false,
            right_pressed: false,
            down_pressed: false,
            left_pressed: false,
            light_color: Colour::new(231, 231, 231),
            neutral_color: Colour::new(66, 66, 66),
        }
    }

    pub fn set_pressed(
        &mut self,
        up_pressed: bool,
        right_pressed: bool,
        down_pressed: bool,
        left_pressed: bool,
    ) {
        self.up_pressed = up_pressed;
        self.right_pressed = right_pressed;
        self.down_pressed = down_pressed;
        self.left_pressed = left_pressed;
    }

    pub async fn blit(&self, screen: &Screen<'_, SPI0>) -> Result<(), Error> {
        let directions = [
            (self.up_pressed, &self.light_color, &self.neutral_color),
            (self.right_pressed, &self.light_color, &self.neutral_color),
            (self.down_pressed, &self.light_color, &self.neutral_color),
            (self.left_pressed, &self.light_color, &self.neutral_color),
        ];

        let shapes = [
            // Top
            [(15, 0, 15, 12), (18, 12, 9, 3), (21, 15, 3, 3)],
            // Right
            [(32, 15, 12, 15), (30, 18, 3, 9), (27, 21, 3, 3)],
            // Down
            [(15, 33, 15, 12), (18, 30, 9, 3), (21, 27, 3, 3)],
            // Left
            [(0, 15, 12, 15), (12, 18, 3, 9), (15, 21, 3, 3)],
        ];

        for (pressed, dir) in directions.iter().zip(shapes.iter()) {
            let color = if pressed.0 { pressed.1 } else { pressed.2 };

            for &(dx, dy, w, h) in dir.iter() {
                screen
                    .draw_rectangle(color, self.x + dx, self.y + dy, w, h)
                    .await?;
            }
        }

        Ok(())
    }
}
