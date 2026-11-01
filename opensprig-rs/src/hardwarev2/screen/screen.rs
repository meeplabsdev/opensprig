use crate::{
    hardwarev2::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, raw::RawScreen},
    types::{Colour, Error, Result},
};
use bdf_parser::{Font, Glyph};
use embassy_rp::{
    Peri,
    gpio::Pin,
    spi::{Async, Spi},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub struct Screen<'a, T>
where
    T: embassy_rp::spi::Instance,
{
    raw: Mutex<CriticalSectionRawMutex, RawScreen<'a, T>>,
}

impl<'a, T> Screen<'a, T>
where
    T: embassy_rp::spi::Instance,
{
    pub async fn new(
        spi: Spi<'a, T, Async>,
        cs: Peri<'a, impl Pin>,
        dc: Peri<'a, impl Pin>,
        rst: Peri<'a, impl Pin>,
        backlight: Peri<'a, impl Pin>,
    ) -> Result<Self> {
        let mut raw = RawScreen::new(spi, cs, dc, rst, backlight).await?;
        raw.init_screen_tft().await?;
        raw.window(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT).await?;
        raw.cframe(0).await?;
        raw.set_backlight(true).await;

        Ok(Self {
            raw: Mutex::new(raw),
        })
    }

    pub async fn raw(&self, x: usize, y: usize, w: usize, h: usize, data: &[u16]) -> Result<()> {
        let mut raw = self.raw.lock().await;
        raw.window(x, y, w, h).await?;
        raw.frame(data).await?;

        Ok(())
    }

    pub async fn pixel(&self, x: usize, y: usize, colour: &Colour) -> Result<()> {
        self.raw(x, y, 1, 1, &[colour.as_16bit()]).await
    }

    pub async fn rectangle(
        &self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        colour: &Colour,
    ) -> Result<()> {
        let mut raw = self.raw.lock().await;
        raw.window(x, y, w, h).await?;
        raw.cframe(colour.as_16bit()).await?;

        Ok(())
    }

    pub async fn flood(&self, colour: &Colour) -> Result<()> {
        self.rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, colour)
            .await
    }

    async fn glyph(
        &self,
        x: usize,
        y: usize,
        font: &Font<'a>,
        glyph: &Glyph,
        colour: &Colour,
    ) -> Result<()> {
        let mut raw = self.raw.lock().await;
        let left = x as isize + glyph.offset.0;
        let top = y as isize + font.ascent - glyph.offset.1 - glyph.size.1 as isize;

        for dy in 0..glyph.size.1 {
            for dx in 0..glyph.size.0 {
                if font.pixel(glyph, dx, dy)? {
                    let cx = (left + dx as isize).max(0) as usize;
                    let cy = (top + dy as isize).max(0) as usize;

                    raw.window(cx, cy, 1, 1).await?;
                    raw.frame(&[colour.as_16bit()]).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn text(
        &self,
        x: usize,
        y: usize,
        content: &str,
        font: &Font<'a>,
        colour: &Colour,
    ) -> Result<()> {
        // self.raw.lock().await.skip_spi = true;

        let mut cx = x;
        for c in content.chars() {
            let glyph = font.glyph(c).ok_or(Error::msg("failed to get glyph"))?;

            self.glyph(cx, y, font, glyph, colour).await?;
            cx += glyph.dwidth.0;
        }

        // let mut raw = self.raw.lock().await;
        // let pixels = &raw.frame.pixels.clone();

        // raw.skip_spi = false;
        // raw.window(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT).await?;
        // raw.frame(pixels).await?;

        Ok(())
    }
}
