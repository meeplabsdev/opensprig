use atomic_float::AtomicF32;
use core::sync::atomic::Ordering;
use embassy_rp::{
    Peri,
    pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};

const MAX_PWM_BRIGHTNESS: f32 = 8192.0;

pub struct PwmLed<'a> {
    config: Mutex<ThreadModeRawMutex, Config>,
    pwm: Mutex<ThreadModeRawMutex, Pwm<'a>>,
    diff: AtomicF32,
}

impl<'a> PwmLed<'a> {
    pub fn new_a<T: Slice>(slice: Peri<'a, T>, pin: Peri<'a, impl ChannelAPin<T>>) -> Self {
        let config: Config = Config::default();
        let pwm: Pwm<'a> = Pwm::new_output_a(slice, pin, config.clone());

        Self {
            config: Mutex::new(config),
            pwm: Mutex::new(pwm),
            diff: AtomicF32::new(0.0),
        }
    }

    pub fn new_b<T: Slice>(slice: Peri<'a, T>, pin: Peri<'a, impl ChannelBPin<T>>) -> Self {
        let config: Config = Config::default();
        let pwm: Pwm<'a> = Pwm::new_output_b(slice, pin, config.clone());

        Self {
            config: Mutex::new(config),
            pwm: Mutex::new(pwm),
            diff: AtomicF32::new(0.0),
        }
    }

    pub async fn set(&self, val: f32) -> () {
        if val >= 0.0 && val <= 1.0 {
            let mut config = self.config.lock().await;

            config.compare_a = (val * MAX_PWM_BRIGHTNESS) as u16;
            self.pwm.lock().await.set_config(&config);
        }
    }

    pub async fn blink(&self, diff: f32) -> () {
        self.diff.store(diff, Ordering::Relaxed);
    }

    pub async fn _run(&self) -> ! {
        const SLEEP_DURATION: Duration = Duration::from_millis(10);

        let mut val: f32 = 0.0;
        let mut edge: i8 = 10;

        loop {
            let diff = self.diff.load(Ordering::Relaxed);
            if diff > 0.0 {
                val += diff * edge as f32;
                if val >= 1.0 || val <= 0.0 {
                    edge = -edge;
                }

                self.set(val).await;
            }

            Timer::after(SLEEP_DURATION).await;
        }
    }
}

#[macro_export]
macro_rules! pwm_led_a {
    ($spawner:expr, $slice:expr, $pin:expr) => {{
        {
            use opensprig_rs::hardware::PwmLed;

            #[embassy_executor::task]
            async fn task(led: &'static PwmLed<'static>) -> ! {
                led._run().await
            }

            static LED: StaticCell<PwmLed> = StaticCell::new();
            let led: &'static PwmLed = LED.init(PwmLed::new_a($slice, $pin));
            $spawner.spawn(unwrap!(task(led)));

            led
        }
    }};
}

#[macro_export]
macro_rules! pwm_led_b {
    ($spawner:expr, $slice:expr, $pin:expr) => {{
        {
            use opensprig_rs::hardware::PwmLed;

            #[embassy_executor::task]
            async fn task(led: &'static PwmLed<'static>) -> ! {
                led._run().await
            }

            static LED: StaticCell<PwmLed> = StaticCell::new();
            let led: &'static PwmLed = LED.init(PwmLed::new_b($slice, $pin));
            $spawner.spawn(unwrap!(task(led)));

            led
        }
    }};
}
