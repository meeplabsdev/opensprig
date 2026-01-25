#![allow(nonstandard_style)]
#![allow(dead_code)]

pub const MAX_LEVEL: u32 = 100;
pub const MAX_BRIGHTNESS: u32 = 8192;

#[repr(C)]
pub enum LED_TYPE {
    PICO,
    STATUS = 28,
    NETWORK = 4,
}

#[repr(C)]
pub struct LED {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn LED_new(pin: LED_TYPE, pwm: bool) -> *mut LED;
    pub fn LED_free(led: *mut LED);

    pub fn LED_get_brightness(led: *mut LED) -> u32;
    pub fn LED_set_brightness_bool(led: *mut LED, level: bool);
    pub fn LED_set_brightness_uint(led: *mut LED, level: u32);
    pub fn LED_set_absolute_brightness(led: *mut LED, level: u16);
    pub fn LED_blink(led: *mut LED);
}
