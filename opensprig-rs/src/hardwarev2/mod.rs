mod screen;

pub use screen::Screen;

// pub struct Sprig<'a> {
//     screen: Screen<'a, SPI0>,
// }

// impl Sprig<'_> {
//     pub async fn new() -> Result<Self> {
//         let spi = Spi::new(
//                 $p.SPI0, $p.PIN_18, $p.PIN_19, $p.PIN_16, $p.DMA_CH2, $p.DMA_CH3, Irqs, spi_config,
//             );

//         let screen = Screen::new(spi, cs, dc, rst, backlight).await?;

//         Ok(Self { screen })
//     }
// }
