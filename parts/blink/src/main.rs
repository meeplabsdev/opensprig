#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0};
use embassy_rp::pio::InterruptHandler;
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use opensprig_rs::init;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>,dma::InterruptHandler<DMA_CH1>,dma::InterruptHandler<DMA_CH2>,dma::InterruptHandler<DMA_CH3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let (
        _watchdog,
        _net_device,
        mut control,
        _screen,
        _speaker,
        _storage,
        _button_l_up,
        _button_l_left,
        _button_l_down,
        _button_l_right,
        _button_r_up,
        _button_r_left,
        _button_r_down,
        _button_r_right,
        _led_left,
        _led_right,
    ) = init!(spawner, p);

    loop {
        info!("led on!");
        control.gpio_set(0, true).await;
        Timer::after_secs(1).await;

        info!("led off!");
        control.gpio_set(0, false).await;
        Timer::after_secs(1).await;
    }
}
