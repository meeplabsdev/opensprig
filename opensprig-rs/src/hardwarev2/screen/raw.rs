use crate::{
    err,
    hardwarev2::screen::{SCREEN_HEIGHT, SCREEN_WIDTH},
    types::{DebugError, Result},
};
use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    spi::{Async, Spi},
};
use embassy_time::Timer;

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

pub(crate) struct RawScreen<'a, T>
where
    T: embassy_rp::spi::Instance,
{
    backlight: Output<'a>,
    cs: Output<'a>,
    dc: Output<'a>,
    rst: Output<'a>,
    spi: Spi<'a, T, Async>,
    window: (usize, usize, usize, usize),
}

impl<'a, T> RawScreen<'a, T>
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
        let this = Self {
            backlight: Output::new(backlight, Level::Low),
            cs: Output::new(cs, Level::High),
            dc: Output::new(dc, Level::Low),
            rst: Output::new(rst, Level::Low),
            spi,
            window: (0, 0, 0, 0),
        };

        Ok(this)
    }
    async fn cs_low(&mut self) {
        nop3!(self.cs.set_low());
    }

    async fn cs_high(&mut self) {
        nop3!(self.cs.set_high());
    }

    async fn dc_low(&mut self) {
        nop3!(self.dc.set_low());
    }

    async fn dc_high(&mut self) {
        nop3!(self.dc.set_high());
    }

    async fn rst_low(&mut self) {
        nop3!(self.rst.set_low());
    }

    async fn rst_high(&mut self) {
        nop3!(self.rst.set_high());
    }

    pub async fn spi_command(&mut self, x: u8) -> Result<()> {
        self.dc_low().await;
        self.spi.write(&[x]).await.map_err(DebugError)?;

        Ok(())
    }

    pub async fn spi_data(&mut self, x: &[u8]) -> Result<()> {
        self.dc_high().await;
        self.spi.write(x).await.map_err(DebugError)?;

        Ok(())
    }

    pub async fn write_command(&mut self, cmd: u8) -> Result<()> {
        self.cs_low().await;
        self.dc_low().await;
        self.spi.write(&[cmd]).await.map_err(DebugError)?;
        self.cs_high().await;

        Ok(())
    }

    pub async fn write_data(&mut self, data: &[u8]) -> Result<()> {
        self.cs_low().await;
        self.dc_high().await;
        self.spi.write(data).await.map_err(DebugError)?;
        self.cs_high().await;

        Ok(())
    }

    pub async fn reset(&mut self) {
        self.rst_high().await;
        Timer::after_millis(10).await;
        self.rst_low().await;
        Timer::after_millis(10).await;
        self.rst_high().await;
        Timer::after_millis(10).await;
    }

    pub async fn set_backlight(&mut self, enabled: bool) {
        if enabled {
            self.backlight.set_high();
        } else {
            self.backlight.set_low();
        }
    }

    pub async fn init_screen_tft(&mut self) -> Result<()> {
        self.reset().await;
        self.dc_low().await;

        {
            // soft reset
            self.write_command(st7735::SWRESET).await?;
            // display loads default register values so wait
            Timer::after_millis(150).await;
            // sleep out and booster on
            self.write_command(st7735::SLPOUT).await?;
            Timer::after_millis(500).await;

            // frame rate control for all modes
            self.write_command(st7735::FRMCTR1).await?;
            self.write_data(&[0x01, 0x2C, 0x2D]).await?;
            self.write_command(st7735::FRMCTR2).await?;
            self.write_data(&[0x01, 0x2D, 0x2D]).await?;
            self.write_command(st7735::FRMCTR3).await?;
            self.write_data(&[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D])
                .await?;

            // set frame inversion true for all modes
            self.write_command(st7735::INVCTR).await?;
            self.write_data(&[0x07]).await?;

            // power control just read the datasheet for these ones
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

            // same for this just settings the voltages
            self.write_command(st7735::VMCTR1).await?;
            self.write_data(&[0x0E]).await?;

            // set display inversion off
            self.write_command(st7735::INVOFF).await?;
            self.dc_high().await; // this is like sending no data

            // memory data access control
            self.write_command(st7735::MADCTL).await?;
            // read left to right top to bottom, refresh top to bottom left to right, bgr colour mode
            self.write_data(&[0x4A]).await?; // MY=0 MX=1 MV=0 ML=0 RGB=1 MH=0
            // colour bits
            self.write_command(st7735::COLMOD).await?;
            self.write_data(&[0x05]).await?; // 16 bit colour
        }

        {
            // set gamma ('+' polarity)
            self.write_command(st7735::GMCTRP1).await?;
            self.write_data(&[
                0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01,
                0x03, 0x10,
            ])
            .await?;

            // set gamma ('-' polarity)
            self.write_command(st7735::GMCTRN1).await?;
            self.write_data(&[
                0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00,
                0x02, 0x10,
            ])
            .await?;

            // normal display mode on
            self.write_command(st7735::NORON).await?;
            Timer::after_millis(10).await;
            // display on
            self.write_command(st7735::DISPON).await?;
            Timer::after_millis(100).await;
        }

        Ok(())
    }

    pub async fn window(&mut self, x: usize, y: usize, w: usize, h: usize) -> Result<()> {
        if x + w > SCREEN_WIDTH || y + h > SCREEN_HEIGHT {
            return err!("window is out of screen bounds");
        }

        self.write_command(st7735::RASET).await?;
        self.write_data(&[0, x as u8, 0, (x + w - 1) as u8]).await?;
        self.write_command(st7735::CASET).await?;
        self.write_data(&[0, y as u8, 0, (y + h - 1) as u8]).await?;
        self.window = (x, y, w, h);

        Ok(())
    }

    // make sure you set the window first
    pub async fn frame(&mut self, pixels: &[u16]) -> Result<()> {
        if pixels.len() != (self.window.2 * self.window.3) as usize {
            return err!("pixels do not match window");
        }

        self.cs_low().await;
        self.spi_command(st7735::RAMWR).await?;
        self.dc_high().await;

        let mut chunk = [0u8; 64];
        for block in pixels.chunks(32) {
            for (i, p) in block.iter().enumerate() {
                chunk[i * 2] = (p >> 8) as u8;
                chunk[i * 2 + 1] = (p & 0xff) as u8;
            }

            let n = block.len() * 2;
            self.spi.write(&chunk[..n]).await.map_err(DebugError)?;
        }

        self.cs_high().await;

        Ok(())
    }

    pub async fn cframe(&mut self, pixel: u16) -> Result<()> {
        self.cs_low().await;
        self.spi_command(st7735::RAMWR).await?;
        self.dc_high().await;

        let high = (pixel >> 8) as u8;
        let low = (pixel & 0xff) as u8;
        let mut count = self.window.2 * self.window.3 * 2;

        let mut chunk = [0u8; 64];
        for i in 0..32 {
            chunk[i * 2] = high;
            chunk[i * 2 + 1] = low;
        }

        while count >= 64 {
            self.spi.write(&chunk).await.map_err(DebugError)?;
            count -= 64;
        }

        if count > 0 {
            self.spi
                .write(&chunk[..count as usize])
                .await
                .map_err(DebugError)?;
        }

        self.cs_high().await;

        Ok(())
    }

    pub async fn pframe<P>(&mut self, mut pred: P) -> Result<()>
    where
        P: FnMut(usize) -> u16,
    {
        self.cs_low().await;
        self.spi_command(st7735::RAMWR).await?;
        self.dc_high().await;

        let count = self.window.2 * self.window.3;
        let mut index = 0;

        let mut chunk = [0u8; 64];
        while index < count {
            let mut i = 0;
            while i < 32 && index < count {
                let p = pred(index);
                chunk[i * 2] = (p >> 8) as u8;
                chunk[i * 2 + 1] = (p & 0xff) as u8;

                i += 1;
                index += 1;
            }

            self.spi.write(&chunk[..i * 2]).await.map_err(DebugError)?;
        }

        self.cs_high().await;

        Ok(())
    }

    // pub fn read_id(&mut self) {
    //     // TODO: RDDID command
    // }

    // TODO: raw memory read+write
}
