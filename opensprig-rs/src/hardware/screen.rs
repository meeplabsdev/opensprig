use crate::{
    font::FOURBYSIX,
    types::{Colour, Error},
};
use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    spi::{self, Async, Instance, Spi},
};
use embassy_sync::{
    blocking_mutex::Mutex as BlockingMutex, blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex,
};
use embassy_time::Timer;
use static_cell::StaticCell;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 128;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

#[allow(unused)]
pub mod st7735 {
    pub const NOP: u8 = 0x00;
    pub const SWRESET: u8 = 0x01;
    pub const RDDID: u8 = 0x04;
    pub const RDDST: u8 = 0x09;
    pub const SLPIN: u8 = 0x10;
    pub const SLPOUT: u8 = 0x11;
    pub const PTLON: u8 = 0x12;
    pub const NORON: u8 = 0x13;
    pub const INVOFF: u8 = 0x20;
    pub const INVON: u8 = 0x21;
    pub const DISPOFF: u8 = 0x28;
    pub const DISPON: u8 = 0x29;
    pub const CASET: u8 = 0x2A;
    pub const RASET: u8 = 0x2B;
    pub const RAMWR: u8 = 0x2C;
    pub const RAMRD: u8 = 0x2E;
    pub const PTLAR: u8 = 0x30;
    pub const VSCRDEF: u8 = 0x33;
    pub const COLMOD: u8 = 0x3A;
    pub const MADCTL: u8 = 0x36;
    pub const MADCTL_MY: u8 = 0x80;
    pub const MADCTL_MX: u8 = 0x40;
    pub const MADCTL_MV: u8 = 0x20;
    pub const MADCTL_ML: u8 = 0x10;
    pub const MADCTL_RGB: u8 = 0x00;
    pub const VSCRSADD: u8 = 0x37;
    pub const FRMCTR1: u8 = 0xB1;
    pub const FRMCTR2: u8 = 0xB2;
    pub const FRMCTR3: u8 = 0xB3;
    pub const INVCTR: u8 = 0xB4;
    pub const DISSET5: u8 = 0xB6;
    pub const PWCTR1: u8 = 0xC0;
    pub const PWCTR2: u8 = 0xC1;
    pub const PWCTR3: u8 = 0xC2;
    pub const PWCTR4: u8 = 0xC3;
    pub const PWCTR5: u8 = 0xC4;
    pub const VMCTR1: u8 = 0xC5;
    pub const RDID1: u8 = 0xDA;
    pub const RDID2: u8 = 0xDB;
    pub const RDID3: u8 = 0xDC;
    pub const RDID4: u8 = 0xDD;
    pub const PWCTR6: u8 = 0xFC;
    pub const GMCTRP1: u8 = 0xE0;
    pub const GMCTRN1: u8 = 0xE1;
}

macro_rules! nop3 {
    ($exp:expr) => {{
        cortex_m::asm::nop();
        cortex_m::asm::nop();
        cortex_m::asm::nop();
        $exp;
        cortex_m::asm::nop();
        cortex_m::asm::nop();
        cortex_m::asm::nop();
    }};
}

pub struct Screen<'a, T: Instance> {
    backlight: Mutex<ThreadModeRawMutex, Output<'a>>,
    cs: Mutex<ThreadModeRawMutex, Output<'a>>,
    dc: Mutex<ThreadModeRawMutex, Output<'a>>,
    rst: Mutex<ThreadModeRawMutex, Output<'a>>,
    spi: &'a BlockingMutex<ThreadModeRawMutex, Spi<'a, T, Async>>,
    pixels: Mutex<ThreadModeRawMutex, &'a mut [u16; SCREEN_SIZE]>,
}

impl<'a, T: Instance> Screen<'a, T> {
    pub async fn new(
        spi: &'a BlockingMutex<ThreadModeRawMutex, Spi<'a, T, Async>>,
        cs: Peri<'a, impl Pin>,
        dc: Peri<'a, impl Pin>,
        rst: Peri<'a, impl Pin>,
        backlight: Peri<'a, impl Pin>,
    ) -> Result<Self, Error> {
        static PIXELS: StaticCell<[u16; SCREEN_SIZE]> = StaticCell::new();
        let pixels = PIXELS.init([0u16; SCREEN_SIZE]);

        let screen = Self {
            backlight: Mutex::new(Output::new(backlight, Level::Low)),
            cs: Mutex::new(Output::new(cs, Level::High)),
            dc: Mutex::new(Output::new(dc, Level::Low)),
            rst: Mutex::new(Output::new(rst, Level::Low)),
            spi: spi,
            pixels: Mutex::new(pixels),
        };

        screen.init_screen_tft().await?;
        screen.blit().await?;
        Timer::after_millis(50).await;
        screen.set_backlight(true).await;

        Ok(screen)
    }

    async fn cs_low(&self) -> () {
        nop3!(self.cs.lock().await.set_low());
    }

    async fn cs_high(&self) -> () {
        nop3!(self.cs.lock().await.set_high());
    }

    async fn dc_low(&self) -> () {
        nop3!(self.dc.lock().await.set_low());
    }

    async fn dc_high(&self) -> () {
        nop3!(self.dc.lock().await.set_high());
    }

    async fn rst_low(&self) -> () {
        nop3!(self.rst.lock().await.set_low());
    }

    async fn rst_high(&self) -> () {
        nop3!(self.rst.lock().await.set_high());
    }

    async fn spi_command(&self, x: u8) -> Result<(), spi::Error> {
        self.dc_low().await;
        unsafe { self.spi.lock_mut(|s| s.blocking_write(&[x])) }
    }

    async fn spi_data(&self, x: &[u8]) -> Result<(), spi::Error> {
        self.dc_high().await;
        unsafe { self.spi.lock_mut(|s| s.blocking_write(x)) }
    }

    async fn write_command(&self, cmd: u8) -> Result<(), Error> {
        self.cs_low().await;
        self.dc_low().await;
        (unsafe { self.spi.lock_mut(|s| s.blocking_write(&[cmd]))? });
        self.cs_high().await;

        Ok(())
    }

    async fn write_data(&self, data: &[u8]) -> Result<(), Error> {
        self.cs_low().await;
        self.dc_high().await;
        (unsafe { self.spi.lock_mut(|s| s.blocking_write(data))? });
        self.cs_high().await;

        Ok(())
    }

    async fn reset(&self) -> () {
        self.rst_high().await;
        Timer::after_millis(10).await;
        self.rst_low().await;
        Timer::after_millis(10).await;
        self.rst_high().await;
        Timer::after_millis(10).await;
    }

    async fn init_screen_tft(&self) -> Result<(), Error> {
        self.reset().await;
        self.dc_low().await;

        {
            // read the datasheet to understand
            self.write_command(st7735::SWRESET).await?;
            Timer::after_millis(150).await;
            self.write_command(st7735::SLPOUT).await?;
            Timer::after_millis(500).await;

            self.write_command(st7735::FRMCTR1).await?;
            self.write_data(&[0x01, 0x2C, 0x2D]).await?;
            self.write_command(st7735::FRMCTR2).await?;
            self.write_data(&[0x01, 0x2D, 0x2D]).await?;
            self.write_command(st7735::FRMCTR3).await?;
            self.write_data(&[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D])
                .await?;

            self.write_command(st7735::INVCTR).await?;
            self.write_data(&[0x07]).await?;

            self.write_command(st7735::PWCTR1).await?;
            self.write_data(&[0xA2, 0x02, 0x84]).await?;
            self.write_command(st7735::PWCTR2).await?;
            self.write_data(&[0xC5]).await?;
            self.write_command(st7735::PWCTR3).await?;
            self.write_data(&[0x0A, 0x00]).await?;
            self.write_command(st7735::PWCTR4).await?;
            self.write_data(&[0x8A, 0x2A]).await?;
            self.write_command(st7735::PWCTR5).await?;
            self.write_data(&[0x8A, 0xEE]).await?;

            self.write_command(st7735::VMCTR1).await?;
            self.write_data(&[0x0E]).await?;

            self.write_command(st7735::INVOFF).await?;
            self.dc_high().await;

            self.write_command(st7735::MADCTL).await?;
            self.write_data(&[0x4A]).await?;
            self.write_command(st7735::COLMOD).await?;
            self.write_data(&[0x05]).await?;
        }

        {
            self.spi_command(st7735::CASET).await?;
            self.spi_data(&[0x00, 0x00, 0x00, 0x7F]).await?;
            self.spi_command(st7735::RASET).await?;
            self.spi_data(&[0x00, 0x00, 0x00, 0x9F]).await?;
        }

        {
            self.write_command(st7735::GMCTRP1).await?;
            self.write_data(&[
                0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01,
                0x03, 0x10,
            ])
            .await?;

            self.write_command(st7735::GMCTRN1).await?;
            self.write_data(&[
                0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00,
                0x02, 0x10,
            ])
            .await?;

            self.write_command(st7735::NORON).await?;
            Timer::after_millis(10).await;
            self.write_command(st7735::DISPON).await?;
            Timer::after_millis(100).await;
        }

        Ok(())
    }

    pub async fn blit(&self) -> Result<(), Error> {
        self.cs_low().await;

        self.spi_command(st7735::CASET).await?;
        self.spi_data(&[0x00, 0x00, 0x00, 0x7F]).await?;

        self.spi_command(st7735::RASET).await?;
        self.spi_data(&[0x00, 0x00, 0x00, 0x9F]).await?;

        self.spi_command(st7735::RAMWR).await?;
        self.dc_high().await; // for empty data

        unsafe {
            let data = core::slice::from_raw_parts(
                self.pixels.lock().await.as_ptr() as *const u8,
                SCREEN_SIZE * 2,
            );

            self.spi.lock_mut(|s| s.blocking_write(data))?
        };

        self.cs_high().await;

        Ok(())
    }

    pub async fn set_backlight(&self, enabled: bool) -> () {
        if enabled {
            self.backlight.lock().await.set_high();
        } else {
            self.backlight.lock().await.set_low();
        }
    }

    pub async fn set_pixel(&self, colour: &Colour, x: usize, y: usize) -> Result<(), Error> {
        if x > SCREEN_WIDTH || y > SCREEN_HEIGHT {
            return Err(Error::new("Position out of screen bounds"));
        }

        (&mut (*self.pixels.lock().await))[x * SCREEN_HEIGHT + y] = colour.as_16bit();

        Ok(())
    }

    pub async fn draw_flood(&self, colour: &Colour) -> () {
        self.pixels.lock().await.fill(colour.as_16bit());
    }

    pub async fn draw_rectangle(
        &self,
        colour: &Colour,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Result<(), Error> {
        if x > SCREEN_WIDTH || y > SCREEN_HEIGHT {
            return Err(Error::new("Position out of screen bounds"));
        } else if x + w > SCREEN_WIDTH || y + h > SCREEN_HEIGHT {
            return Err(Error::new("Rectangle out of screen bounds"));
        } else if w == 0 || h == 0 {
            return Err(Error::new("Rectangle size out of bounds"));
        }

        for col in x..(x + w) {
            let i: usize = col * SCREEN_HEIGHT + y;
            let row = &mut (*self.pixels.lock().await)[i..(i + h)];
            row.fill(colour.as_16bit());
        }

        Ok(())
    }

    pub async fn draw_character(
        &self,
        colour: &Colour,
        ord: usize,
        x: usize,
        y: usize,
    ) -> Result<(), Error> {
        if ord - 32 >= FOURBYSIX.len() {
            return Err(Error::new("Ordinal out of range"));
        }

        let character = FOURBYSIX[ord - 32];
        for i in 0..24 {
            if ((character >> i) & 1) == 1 {
                let cx = x + 4 - (i % 4);
                let cy = y + 6 - (i / 4);
                self.set_pixel(colour, cx, cy).await?;
            }
        }

        Ok(())
    }

    pub async fn draw_text(
        &self,
        colour: &Colour,
        content: &str,
        x: usize,
        y: usize,
    ) -> Result<(), Error> {
        for (i, c) in content.char_indices() {
            self.draw_character(colour, c as usize, x + i * 4, y)
                .await?;
        }

        Ok(())
    }
}
