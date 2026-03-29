#![no_std]
#![no_main]

mod dpad_indicator;

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

use crate::dpad_indicator::DPadIndicator;
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use opensprig_rs::hardware::{Button, PwmLed, Screen};
use opensprig_rs::types::Colour;
use opensprig_rs::{button, clm, fw, nvram, pwm_led_a};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let p = embassy_rp::init(Default::default());

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        dma::Channel::new(p.DMA_CH0, Irqs),
    );

    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw!(), nvram!()).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));

    control.init(clm!()).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let button_l_up = button!(spawner, p.PIN_5);
    let button_l_left = button!(spawner, p.PIN_6);
    let button_l_down = button!(spawner, p.PIN_7);
    let button_l_right = button!(spawner, p.PIN_8);

    let button_r_up = button!(spawner, p.PIN_12);
    let button_r_left = button!(spawner, p.PIN_13);
    let button_r_down = button!(spawner, p.PIN_14);
    let button_r_right = button!(spawner, p.PIN_15);

    let led_left = pwm_led_a!(spawner, p.PWM_SLICE6, p.PIN_28);
    let led_right = pwm_led_a!(spawner, p.PWM_SLICE2, p.PIN_4);

    let screen = Screen::new(
        p.PIN_17, p.PIN_20, p.PIN_22, p.PIN_26, p.SPI0, p.PIN_18, p.PIN_19, p.DMA_CH1, p.PIN_16,
        p.DMA_CH2, Irqs,
    )
    .await
    .unwrap();

    let mut left_dpad_indicator = DPadIndicator::new(12, 40);
    let mut right_dpad_indicator = DPadIndicator::new(103, 40);

    screen.draw_flood(&Colour::new(0, 0, 0)).await;

    screen
        .draw_text(&Colour::new(255, 255, 255), "Hello, World!", 5, 5)
        .await
        .unwrap();

    screen.blit().await.unwrap();

    led_left.blink(1.0 / 1024.0).await;
    led_right.blink(1.0 / 1024.0).await;

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

        left_dpad_indicator.blit(&screen).await.unwrap();
        right_dpad_indicator.blit(&screen).await.unwrap();
        screen.blit().await.unwrap();
        Timer::after_millis(5).await;

        // button_l_left.wait_held().await;
        // control.gpio_set(0, true).await;
        // Timer::after_millis(5).await;
        // control.gpio_set(0, false).await;
        // button_l_left.wait_released().await;
    }
}
