#![allow(nonstandard_style)]
#![allow(dead_code)]

#[repr(C)]
pub struct Speaker {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn Speaker_new() -> *mut Speaker;
    pub fn Speaker_free(speaker: *mut Speaker);

    pub fn Speaker_sine(speaker: *mut Speaker, step: u32, volume: u32);
}
