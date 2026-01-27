#![no_std]
#![no_main]

use opensprig_rs::bridge::button_bridge::*;
use opensprig_rs::bridge::led_bridge::*;
use opensprig_rs::bridge::screen_bridge::*;
use opensprig_rs::bridge::speaker_bridge::*;
use opensprig_rs::bridge::storage_bridge::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        let led: *mut LED = LED_new(LED_TYPE::STATUS, true);
        let button: *mut Button = Button_new(BUTTON_TYPE::L_LEFT);
        let screen: *mut Screen = Screen_new();
        let speaker: *mut Speaker = Speaker_new();
        let storage: *mut Storage = Storage_new();

        Storage_mount(storage);
        Screen_set_backlight(screen, true);
        Screen_draw_flood(screen, RGB(100, 100, 255));
        Screen_blit(screen);
        Speaker_sine(speaker, 0x800000, 16);

        loop {
            let pressed = Button_is_pressed(button);
            LED_set_brightness_bool(led, pressed);
        }
    }
}
