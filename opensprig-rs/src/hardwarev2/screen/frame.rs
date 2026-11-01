use crate::{
    err,
    hardwarev2::screen::{SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH},
    types::Result,
};

pub struct Frame {
    pub pixels: [u16; SCREEN_SIZE],
    pos: (usize, usize),
    window: (usize, usize, usize, usize),
    pub bounds: (usize, usize, usize, usize),
}

impl Frame {
    pub fn new() -> Self {
        Self {
            pixels: [0u16; SCREEN_SIZE],
            pos: (0, 0),
            window: (0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
            bounds: (0, 0, 0, 0),
        }
    }

    pub fn window(&mut self, x: usize, y: usize, w: usize, h: usize) -> Result<()> {
        if x + w > SCREEN_WIDTH || y + h > SCREEN_HEIGHT {
            return err!("window is out of screen bounds");
        }

        self.window = (x, y, x + w, y + h);

        if x < self.bounds.0 {
            self.bounds.2 += self.bounds.0 - x;
            self.bounds.0 = x;
        } else if x + w > self.bounds.0 + self.bounds.2 {
            self.bounds.2 += x + w - (self.bounds.0 + self.bounds.2);
        }

        if y < self.bounds.1 {
            self.bounds.3 += self.bounds.1 - y;
            self.bounds.1 = y;
        } else if y + h > self.bounds.1 + self.bounds.3 {
            self.bounds.3 += y + h - (self.bounds.1 + self.bounds.3);
        }

        Ok(())
    }

    pub fn ram_wr(&mut self) {
        self.pos = (self.window.0 + self.window.1 * SCREEN_WIDTH, self.window.1);
    }

    pub fn ram_data(&mut self, data: &[u8]) {
        for entry in data.chunks(2) {
            self.pixels[self.pos.0] = ((entry[0] as u16) << 8) | (entry[1] as u16);

            self.pos.0 += 1;
            if self.pos.0 >= self.window.2 {
                self.pos.1 += 1;
                self.pos.0 = self.window.0 + self.window.1 * SCREEN_WIDTH;
                if self.pos.0 >= self.window.3 {
                    break;
                }
            }
        }
    }

    // TODO: only return the bits that changed
}
