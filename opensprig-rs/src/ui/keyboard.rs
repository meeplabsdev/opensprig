use crate::{
    font,
    hardware::Screen,
    types::Error,
    ui::{COLOUR_BG, COLOUR_HOVER, COLOUR_PRESS, COLOUR_TEXT},
    utf8_from_u8,
};
use embassy_rp::peripherals::SPI0;

const MAX_CONTENT: usize = 256;
const ALPHABET: [&str; 40] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", //
    "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", //
    "A", "S", "D", "F", "G", "H", "J", "K", "L", "-", //
    "Z", "X", "C", "V", "B", "N", "M", " ", ".", "+", //
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
        self.content[self.pointer] = ALPHABET[self.target].chars().next().unwrap() as u8;

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
        if res < 0 || res >= ALPHABET.len() as isize {
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
        utf8_from_u8(&self.content)
    }

    pub async fn blit(&self, screen: &Screen<'_, SPI0>) -> Result<(), Error> {
        for (i, character) in ALPHABET.iter().enumerate() {
            let mut bg = COLOUR_BG;
            if i == self.target {
                if self.pressed {
                    bg = COLOUR_PRESS;
                } else {
                    bg = COLOUR_HOVER;
                }
            }

            let row = i / 10;
            let col = i % 10;

            screen
                .draw_rectangle(&bg, self.x + col * 6, self.y + row * 8 + 8, 6, 8)
                .await?;

            screen
                .draw_text(
                    &COLOUR_TEXT,
                    font::bdf_fourbysix().await?,
                    character,
                    self.x + col * 6 + 1,
                    self.y + row * 8 + 9,
                )
                .await?;
        }

        let mut content = self.get_content();
        if content.len() >= 14 {
            content = &content[(content.len() - 14)..];
        }

        screen
            .draw_rectangle(&COLOUR_HOVER, self.x, self.y, 60, 8)
            .await?;
        screen
            .draw_text(
                &COLOUR_TEXT,
                font::bdf_fourbysix().await?,
                content,
                self.x + 1,
                self.y + 1,
            )
            .await?;

        Ok(())
    }
}
