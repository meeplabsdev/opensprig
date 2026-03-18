use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};

const DEBOUNCE_MS: u64 = 50;
const HOLD_MS: u64 = 800;

pub struct Button<'a> {
    input: Input<'a>,
    pressed: Signal<CriticalSectionRawMutex, ()>,
    held: Signal<CriticalSectionRawMutex, ()>,
}

impl<'a> Button<'a> {
    pub fn new(pin: Peri<'a, impl Pin>) -> Self {
        let mut input = Input::new(pin, Pull::Up);
        input.set_inversion(true);

        Self {
            input,
            pressed: Signal::new(),
            held: Signal::new(),
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed.signaled()
    }

    pub fn is_held(&self) -> bool {
        self.held.signaled()
    }

    pub async fn wait_pressed(&self) -> () {
        self.pressed.wait().await
    }

    pub async fn wait_held(&self) -> () {
        self.held.wait().await
    }

    pub async fn debounce(&self) -> ! {
        const SLEEP_DURATION: Duration = Duration::from_millis(1);

        let mut started_time = Instant::now();
        let mut started_high = false;
        let mut is_held = false;

        loop {
            let action_duration = started_time.elapsed().as_millis();
            let is_high = self.input.is_high();

            if is_high && started_high
            // holding the button
            && action_duration > HOLD_MS
            {
                started_time = Instant::now();
                started_high = false;
                is_held = true;

                self.held.signal(());
            } else if !is_high && is_held
            // release the button after holding it
            {
                is_held = false;

                self.held.reset();
            } else if !is_high && started_high
            // quickly pressing the button
            && action_duration < HOLD_MS
            {
                started_time = Instant::now();
                started_high = false;

                self.pressed.signal(());
            } else if is_high && !started_high && !is_held
            // action starting
            && action_duration > DEBOUNCE_MS
            {
                started_time = Instant::now();
                started_high = true;

                self.pressed.reset();
            }

            Timer::after(SLEEP_DURATION).await;
        }
    }
}

#[macro_export]
macro_rules! button {
    ($spawner:expr, $pin:expr) => {{
        {
            static BUTTON: StaticCell<Button> = StaticCell::new();
            let button: &'static Button = BUTTON.init(Button::new($pin));
            $spawner.spawn(unwrap!(debounce_task(button)));

            button
        }
    }};
}
