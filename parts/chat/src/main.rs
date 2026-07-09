#![no_std]
#![no_main]

mod keyboard;

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

use crate::keyboard::Keyboard;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0};
use embassy_rp::pio::InterruptHandler;
use embassy_rp::{bind_interrupts, dma};
use embassy_time::{Duration, Timer};
use opensprig_rs::init;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let (
        mut w,
        _net_device,
        _control,
        screen,
        _speaker,
        _storage,
        button_l_up,
        button_l_left,
        button_l_down,
        button_l_right,
        button_r_up,
        button_r_left,
        button_r_down,
        button_r_right,
        _led_left,
        _led_right,
    ) = init!(spawner, p);

    let mut keyboard = Keyboard::new(0, 0);
    keyboard.blit(screen).await.unwrap();
    screen.blit().await.unwrap();

    loop {
        if button_r_up.is_held() && button_r_right.is_held() {
            w.trigger_reset();
            loop {}
        }

        if button_r_left.is_pressed() {
            keyboard.press();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            button_r_left.wait_released().await;
            keyboard.unpress();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
        } else if button_r_down.is_pressed() {
            keyboard.delete();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            button_r_down.wait_released().await;
        } else if button_l_up.is_pressed() {
            keyboard.up();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            button_l_up.wait_released().await;
        } else if button_l_left.is_pressed() {
            keyboard.left();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            button_l_left.wait_released().await;
        } else if button_l_down.is_pressed() {
            keyboard.down();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            button_l_down.wait_released().await;
        } else if button_l_right.is_pressed() {
            keyboard.right();
            keyboard.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            button_l_right.wait_released().await;
        }

        Timer::after(Duration::from_millis(5)).await;
    }
}
