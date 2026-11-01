#![no_std]
#![no_main]
extern crate alloc;

use alloc::format;
use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts, dma,
    peripherals::{DMA_CH0, DMA_CH1},
    spi,
    spi::Spi,
};
use embedded_alloc::LlffHeap as Heap;
use opensprig_rs::{
    font,
    hardwarev2::{Button, ButtonState, Screen},
    types::Colour,
};

bind_interrupts!(
    struct Irqs {
        DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    #[global_allocator]
    static HEAP: Heap = Heap::empty();
    unsafe {
        embedded_alloc::init!(HEAP, 65536);
    }

    let p = embassy_rp::init(Default::default());

    let mut spi_config = spi::Config::default();
    spi_config.frequency = 20_000_000;
    let spi = Spi::new(
        p.SPI0, p.PIN_18, p.PIN_19, p.PIN_16, p.DMA_CH0, p.DMA_CH1, Irqs, spi_config,
    );

    let fourbysix = font::bdf_fourbysix().await.unwrap();
    let screen = Screen::new(spi, p.PIN_20, p.PIN_22, p.PIN_26, p.PIN_17)
        .await
        .unwrap();

    let button = Button::new(p.PIN_6).unwrap();

    loop {
        match button.next().await {
            ButtonState::Released { duration } => {
                screen
                    .rectangle(4, 4, 156, 6, &Colour::BLACK)
                    .await
                    .unwrap();

                screen
                    .text(
                        4,
                        4,
                        &format!("released after {duration}ms"),
                        fourbysix,
                        &Colour::WHITE,
                    )
                    .await
                    .unwrap();
            }
            ButtonState::Pressed => {
                screen
                    .rectangle(4, 4, 156, 6, &Colour::BLACK)
                    .await
                    .unwrap();

                screen
                    .text(4, 4, "pressed", fourbysix, &Colour::WHITE)
                    .await
                    .unwrap();
            }
            ButtonState::Held => {
                screen
                    .rectangle(4, 4, 156, 6, &Colour::BLACK)
                    .await
                    .unwrap();

                screen
                    .text(4, 4, "held", fourbysix, &Colour::WHITE)
                    .await
                    .unwrap();
            }
        }
    }
}
