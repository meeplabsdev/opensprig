#![no_std]
#![no_main]

mod dpad_indicator;

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

use crate::dpad_indicator::DPadIndicator;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0, SPI0};
use embassy_rp::pio::InterruptHandler;
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use opensprig_rs::hardware::{Speaker, Storage};
use opensprig_rs::init;
use opensprig_rs::types::Colour;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let (
        _watchdog,
        _net_device,
        _control,
        screen,
        speaker,
        storage,
        button_l_up,
        button_l_left,
        button_l_down,
        button_l_right,
        button_r_up,
        button_r_left,
        button_r_down,
        button_r_right,
        led_left,
        led_right,
    ) = init!(spawner, p);

    screen.draw_flood(&Colour::new(0, 0, 0)).await;

    screen
        .draw_text(&Colour::new(255, 255, 255), "Hello, World!", 5, 5)
        .await
        .unwrap();

    screen.blit().await.unwrap();

    let mut left_dpad_indicator = DPadIndicator::new(12, 40);
    let mut right_dpad_indicator = DPadIndicator::new(103, 40);

    led_left.blink(1.0 / 1024.0).await;
    led_right.blink(1.0 / 1024.0).await;

    {
        #[embassy_executor::task]
        async fn task(
            speaker: &'static mut Speaker<'static, PIO0, 1>,
            storage: &'static mut Storage<'static, SPI0>,
        ) -> ! {
            speaker.play_pcm(storage, "audio/believer.pcm", 32).await;
            loop {}
        }

        spawner.spawn(unwrap!(task(speaker, storage)));
    }

    loop {
        left_dpad_indicator.set_pressed(
            button_l_up.is_pressed(),
            button_l_right.is_pressed(),
            button_l_down.is_pressed(),
            button_l_left.is_pressed(),
        );

        right_dpad_indicator.set_pressed(
            button_r_up.is_pressed(),
            button_r_right.is_pressed(),
            button_r_down.is_pressed(),
            button_r_left.is_pressed(),
        );

        left_dpad_indicator.blit(screen).await.unwrap();
        right_dpad_indicator.blit(screen).await.unwrap();
        screen.blit().await.unwrap();
        Timer::after_millis(5).await;
    }
}
