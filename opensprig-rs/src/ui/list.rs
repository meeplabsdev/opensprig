use crate::{
    font,
    hardware::Screen,
    types::{Colour, Error},
    ui::{COLOUR_BG, COLOUR_HOVER, COLOUR_TEXT},
};
use embassy_rp::peripherals::SPI0;

pub struct List<'i, T, U> {
    ids: &'i [T],
    items: &'i [U],
    target: usize,
    do_hover: bool,
}

impl<'i, I: Copy> List<'i, I, I> {
    pub fn new_no_hover(items: &'i [I]) -> Self {
        Self {
            ids: &items,
            items: &items,
            target: 0,
            do_hover: false,
        }
    }
}

impl<'s, 'i, T: Copy, U: Copy> List<'i, T, U> {
    pub fn new(ids: &'i [T], items: &'i [U]) -> Self {
        Self {
            ids: &ids,
            items: &items,
            target: 0,
            do_hover: true,
        }
    }

    fn mutate_target(&mut self, diff: isize) {
        let res = diff + self.target as isize;
        if res < 0 || res >= self.items.len() as isize {
            return;
        }

        self.target = res.unsigned_abs();
    }

    pub fn up(&mut self) {
        self.mutate_target(-1);
    }

    pub fn down(&mut self) {
        self.mutate_target(1);
    }

    pub fn get_target_id(&self) -> &T {
        &self.ids[self.target]
    }

    pub fn get_target(&self) -> &U {
        &self.items[self.target]
    }

    pub async fn blit(
        &self,
        screen: &Screen<'s, SPI0>,
        mut display: impl FnMut(&U) -> &str,
    ) -> Result<(), Error> {
        screen.draw_flood(&Colour::new(0, 0, 0)).await;

        for (i, path) in self.items.iter().enumerate() {
            let mut bg = COLOUR_BG;
            if i == self.target && self.do_hover {
                bg = COLOUR_HOVER;
            }

            screen.draw_rectangle(&bg, 0, i * 8, 160, 8).await?;

            screen
                .draw_text(
                    &COLOUR_TEXT,
                    font::bdf_fourbysix().await?,
                    display(path),
                    1,
                    i * 8 + 1,
                )
                .await?;
        }

        Ok(())
    }
}
