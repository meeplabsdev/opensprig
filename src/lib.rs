#![no_std]
#![no_main]
#![cfg(not(test))]

mod bridge;
use crate::bridge::led_bridge;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        let led: *mut led_bridge::LED = led_bridge::LED_new(led_bridge::LED_TYPE::STATUS, true);
        led_bridge::LED_set_brightness_bool(led, true);
    }

    loop {}
}
