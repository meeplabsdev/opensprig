#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0, SPI0};
use embassy_rp::pio::InterruptHandler;
use embassy_rp::{bind_interrupts, dma};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use opensprig_rs::hardware::Screen;
use opensprig_rs::init;
use opensprig_rs::ui::Keyboard;
use static_cell::StaticCell;

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
        _speaker,
        _storage,
        button_l_up,
        button_l_left,
        button_l_down,
        button_l_right,
        _button_r_up,
        button_r_left,
        button_r_down,
        _button_r_right,
        _led_left,
        _led_right,
    ) = init!(spawner, p);

    static KEYBOARD: StaticCell<Mutex<ThreadModeRawMutex, Keyboard>> = StaticCell::new();
    let keyboard: &'static Mutex<ThreadModeRawMutex, Keyboard> =
        KEYBOARD.init(Mutex::new(Keyboard::new(0, 0)));

    {
        #[embassy_executor::task]
        async fn task(
            keyboard: &'static Mutex<ThreadModeRawMutex, Keyboard>,
            screen: &'static Screen<'static, SPI0>,
        ) {
            keyboard.lock().await.blit(screen).await.unwrap();
            screen.blit().await.unwrap();
            Timer::after_millis(10).await;
        }

        spawner.spawn(unwrap!(task(keyboard, screen)))
    }

    loop {
        if button_r_left.is_pressed() {
            keyboard.lock().await.press();
            button_r_left.wait_released().await;
            screen.blit().await.unwrap();
        } else if button_r_down.is_pressed() {
            keyboard.lock().await.delete();
            button_r_down.wait_released().await;
        } else if button_l_up.is_pressed() {
            keyboard.lock().await.up();
            button_l_up.wait_released().await;
        } else if button_l_left.is_pressed() {
            keyboard.lock().await.left();
            button_l_left.wait_released().await;
        } else if button_l_down.is_pressed() {
            keyboard.lock().await.down();
            button_l_down.wait_released().await;
        } else if button_l_right.is_pressed() {
            keyboard.lock().await.right();
            button_l_right.wait_released().await;
        }

        Timer::after_millis(10).await;
    }
}
