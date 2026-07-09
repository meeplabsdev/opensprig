use embassy_rp::peripherals::SPI0;
use opensprig_rs::{
    hardware::Screen,
    types::{Colour, Error},
};

const COLOUR_BG: Colour = Colour::new(115, 115, 115);
const COLOUR_HOVER: Colour = Colour::new(140, 140, 140);
const COLOUR_PRESS: Colour = Colour::new(230, 230, 230);
const COLOUR_TEXT: Colour = Colour::new(255, 255, 255);

const MAX_CONTENT: usize = 256;
const ALPHABET: [[&str; 10]; 4] = [
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
    ["A", "S", "D", "F", "G", "H", "J", "K", "L", "-"],
    ["Z", "X", "C", "V", "B", "N", "M", " ", ".", "+"],
];

pub struct Keyboard {
    x: usize,
    y: usize,
    target: usize,
    pressed: bool,
    pointer: usize,
    content: [u8; 256],
}

impl Keyboard {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            target: 0,
            pressed: false,
            pointer: 0,
            content: [0u8; MAX_CONTENT],
        }
    }

    pub fn press(&mut self) {
        self.pressed = true;
        self.content[self.pointer] = ALPHABET[self.target / 10][self.target % 10]
            .chars()
            .next()
            .unwrap() as u8;

        if self.pointer < MAX_CONTENT {
            self.pointer += 1;
        }
    }

    pub fn unpress(&mut self) {
        self.pressed = false;
    }

    pub fn delete(&mut self) {
        if self.pointer > 0 {
            self.pointer -= 1;
        }

        self.content[self.pointer] = 0;
    }

    fn mutate_target(&mut self, diff: isize) {
        let res = diff + self.target as isize;
        if res < 0 || res >= 40 {
            return;
        }

        self.target = res.unsigned_abs();
    }

    pub fn left(&mut self) {
        self.mutate_target(-1);
    }

    pub fn right(&mut self) {
        self.mutate_target(1);
    }

    pub fn up(&mut self) {
        self.mutate_target(-10);
    }

    pub fn down(&mut self) {
        self.mutate_target(10);
    }

    pub fn get_content(&self) -> &str {
        let end = self
            .content
            .iter()
            .rposition(|&b| b != 0)
            .map_or(0, |i| i + 1);

        str::from_utf8(&self.content[..end]).unwrap()
    }

    pub async fn blit(&self, screen: &Screen<'_, SPI0>) -> Result<(), Error> {
        for (j, line) in ALPHABET.iter().enumerate() {
            for (i, character) in line.iter().enumerate() {
                let mut bg = COLOUR_BG;
                if i + j * 10 == self.target {
                    if self.pressed {
                        bg = COLOUR_PRESS;
                    } else {
                        bg = COLOUR_HOVER;
                    }
                }

                screen
                    .draw_rectangle(&bg, self.x + i * 6, self.y + j * 8 + 8, 6, 8)
                    .await?;

                screen
                    .draw_text(
                        &COLOUR_TEXT,
                        character,
                        self.x + i * 6 + 1,
                        self.y + j * 8 + 9,
                    )
                    .await?;
            }
        }

        let mut content = self.get_content();
        if content.len() >= 14 {
            content = &content[(content.len() - 14)..];
        }

        screen
            .draw_rectangle(&COLOUR_HOVER, self.x, self.y, 60, 8)
            .await?;
        screen
            .draw_text(&COLOUR_TEXT, content, self.x + 1, self.y + 1)
            .await?;

        Ok(())
    }
}
