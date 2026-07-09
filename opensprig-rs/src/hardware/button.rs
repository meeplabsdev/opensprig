use core::sync::atomic::{AtomicBool, Ordering};
use embassy_futures::select::select;
use embassy_rp::{
    Peri,
    gpio::{Flex, Pin, Pull},
};
use embassy_time::{Duration, Timer};

const DEBOUNCE_MS: Duration = Duration::from_millis(50);
const HOLD_MS: Duration = Duration::from_millis(800);

pub struct Button<'a> {
    pin: Flex<'a>,
    held: AtomicBool,
}

impl<'a> Button<'a> {
    pub fn new(pin: Peri<'a, impl Pin>) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(Pull::Up);
        pin.set_input_inversion(true);

        Self {
            pin: pin,
            held: AtomicBool::new(false),
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pin.is_high()
    }

    pub fn is_held(&self) -> bool {
        self.held.load(Ordering::Relaxed)
    }

    pub async fn wait_pressed(&self) -> () {
        loop {
            if self.is_pressed() {
                break;
            }

            Timer::after(DEBOUNCE_MS).await;
        }
    }

    pub async fn wait_released(&self) -> () {
        loop {
            if !self.is_pressed() {
                break;
            }

            Timer::after(DEBOUNCE_MS).await;
        }
    }

    pub async fn wait_held(&self) -> () {
        loop {
            self.wait_pressed().await;

            if select(self.wait_released(), Timer::after(HOLD_MS))
                .await
                .is_second()
            {
                break;
            }
        }
    }

    pub async fn _run(&self) -> ! {
        loop {
            self.held.store(false, Ordering::Relaxed);
            self.wait_held().await;

            self.held.store(true, Ordering::Relaxed);
            self.wait_released().await;
        }
    }
}

#[macro_export]
macro_rules! button {
    ($spawner:expr, $pin:expr) => {{
        {
            use opensprig_rs::hardware::Button;

            #[embassy_executor::task]
            async fn task(button: &'static Button<'static>) -> ! {
                button._run().await
            }

            static BUTTON: StaticCell<Button> = StaticCell::new();
            let button: &'static Button = BUTTON.init(Button::new($pin));
            $spawner.spawn(unwrap!(task(button)));

            button
        }
    }};
}
