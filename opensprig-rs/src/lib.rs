#![no_std]
#![no_main]

mod font;
mod mutex_device;

pub mod hardware;
pub mod types;

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

#[macro_export]
macro_rules! init {
    ($spawner:expr, $p:expr) => {{
        use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
        use cyw43_pio::PioSpi;
        use defmt::unwrap;
        use embassy_rp::dma::Channel;
        use embassy_rp::gpio::{Level, Output};
        use embassy_rp::peripherals::{PIO0, SPI0};
        use embassy_rp::pio::Pio;
        use embassy_rp::spi::{Async, Config, Spi};
        use embassy_rp::watchdog::Watchdog;
        use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
        use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
        use embassy_time::Timer;
        use opensprig_rs::hardware::{Screen, Speaker, Storage};
        use opensprig_rs::types::Colour;
        use opensprig_rs::{button, clm, fw, init, nvram, pwm_led_a};
        use static_cell::StaticCell;

        #[embassy_executor::task]
        async fn cyw43_task(
            runner: cyw43::Runner<
                'static,
                cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>,
            >,
        ) -> ! {
            runner.run().await
        }

        Timer::after_secs(1).await;
        let watchdog = Watchdog::new($p.WATCHDOG);
        let pwr = Output::new($p.PIN_23, Level::Low);
        let cs = Output::new($p.PIN_25, Level::High);

        let mut pio = Pio::new($p.PIO0, Irqs);
        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            $p.PIN_24,
            $p.PIN_29,
            Channel::new($p.DMA_CH0, Irqs),
        );

        static STATE: StaticCell<cyw43::State> = StaticCell::new();
        let state = STATE.init(cyw43::State::new());

        let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw!(), nvram!()).await;
        $spawner.spawn(unwrap!(cyw43_task(runner)));

        control.init(clm!()).await;
        control
            .set_power_management(cyw43::PowerManagementMode::PowerSave)
            .await;

        let mut spi_config = Config::default();
        spi_config.frequency = 20_000_000;

        static SPI: StaticCell<BlockingMutex<ThreadModeRawMutex, Spi<'static, SPI0, Async>>> =
            StaticCell::new();
        let spi: &'static BlockingMutex<ThreadModeRawMutex, Spi<'static, SPI0, Async>> =
            SPI.init(BlockingMutex::new(Spi::new(
                $p.SPI0, $p.PIN_18, $p.PIN_19, $p.PIN_16, $p.DMA_CH2, $p.DMA_CH3, Irqs, spi_config,
            )));

        static SCREEN: StaticCell<Screen<'static, SPI0>> = StaticCell::new();
        let screen: &'static mut Screen<'static, SPI0> = SCREEN.init(
            Screen::new(spi, $p.PIN_20, $p.PIN_22, $p.PIN_26, $p.PIN_17)
                .await
                .unwrap(),
        );

        screen.draw_flood(&Colour::new(0, 0, 0)).await;
        screen.blit().await.unwrap();

        static SPEAKER: StaticCell<Speaker<'static, PIO0, 1>> = StaticCell::new();
        let speaker: &'static mut Speaker<'static, PIO0, 1> = SPEAKER.init(Speaker::new(
            &mut pio.common,
            pio.sm1,
            $p.PIN_9,
            $p.PIN_10,
            $p.PIN_11,
            $p.DMA_CH1,
            Irqs,
        ));

        static STORAGE: StaticCell<Storage<'static, SPI0>> = StaticCell::new();
        let storage: &'static mut Storage<'static, SPI0> =
            STORAGE.init(Storage::new(spi, $p.PIN_21).await.unwrap());

        let button_l_up = button!($spawner, $p.PIN_5);
        let button_l_left = button!($spawner, $p.PIN_6);
        let button_l_down = button!($spawner, $p.PIN_7);
        let button_l_right = button!($spawner, $p.PIN_8);

        let button_r_up = button!($spawner, $p.PIN_12);
        let button_r_left = button!($spawner, $p.PIN_13);
        let button_r_down = button!($spawner, $p.PIN_14);
        let button_r_right = button!($spawner, $p.PIN_15);

        let led_left = pwm_led_a!($spawner, $p.PWM_SLICE6, $p.PIN_28);
        let led_right = pwm_led_a!($spawner, $p.PWM_SLICE2, $p.PIN_4);

        (
            watchdog,
            net_device,
            control,
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
        )
    }};
}

#[macro_export]
macro_rules! aligned_flash {
    ($addr:expr, $size:expr) => {{
        {
            fn fw() -> &'static cyw43::Aligned<cyw43::A4, [u8; $size]> {
                unsafe { &*($addr as *const _) }
            }

            fw()
        }
    }};
}

#[macro_export]
macro_rules! include_flash {
    ($addr:expr, $size:expr) => {{
        {
            fn fw() -> &'static [u8; $size] {
                unsafe { &*($addr as *const _) }
            }

            fw()
        }
    }};
}

#[macro_export]
macro_rules! fw {
    () => {{ opensprig_rs::aligned_flash!(0x101be000, 0x386A5) }};
}

#[macro_export]
macro_rules! clm {
    () => {{ opensprig_rs::include_flash!(0x101fe000, 0x003D8) }};
}

#[macro_export]
macro_rules! nvram {
    () => {{ opensprig_rs::aligned_flash!(0x101ff000, 0x002E6) }};
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt::{assert, *};

    #[test]
    fn assert_true() {
        info!("This test passes.");
        assert!(true);
    }
}
