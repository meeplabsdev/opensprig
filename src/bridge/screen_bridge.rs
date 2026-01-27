#![allow(nonstandard_style)]
#![allow(dead_code)]

pub const FRAME_WIDTH: u32 = 160;
pub const FRAME_HEIGHT: u32 = 128;
pub const FRAME_SIZE: u32 = FRAME_WIDTH * FRAME_HEIGHT;

#[repr(C)]
pub struct Screen {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn Screen_new() -> *mut Screen;
    pub fn Screen_free(screen: *mut Screen);

    pub fn Screen_screen_buf(screen: *mut Screen) -> *const u16;

    pub fn Screen_blit(screen: *mut Screen);
    pub fn Screen_set_backlight(screen: *mut Screen, enabled: bool);
    pub fn Screen_set_pixel(screen: *mut Screen, colour: u16, x: i32, y: i32);
    pub fn Screen_draw_flood(screen: *mut Screen, colour: u16);
    pub fn Screen_draw_rectangle(screen: *mut Screen, colour: u16, x: i32, y: i32, w: i32, h: i32);
    pub fn Screen_draw_character(screen: *mut Screen, colour: u16, character: i8, x: i32, y: i32);
}

pub unsafe fn Screen_draw_text(screen: *mut Screen, colour: u16, contents: &str, x: i32, y: i32) {
    let mut i: i32 = 0;
    for c in contents.chars() {
        unsafe { Screen_draw_character(screen, colour, c as i8, x + i, y) };
        i += 4;
    }
}

pub unsafe fn Screen_draw_callback<F: FnMut(i32, i32) -> u16>(
    screen: *mut Screen,
    mut callback: F,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    for dx in x..x + w {
        for dy in y..y + h {
            unsafe { Screen_set_pixel(screen, callback(x, y), dx, dy) };
        }
    }
}

pub fn RGB(r: u8, g: u8, b: u8) -> u16 {
    let r = ((r as f32 / 255f32) * 31f32) as u8;
    let b = ((b as f32 / 255f32) * 31f32) as u8;
    let g = ((g as f32 / 255f32) * 63f32) as u8;

    return ((r as u16 & 0b11111000) << 8) | ((b as u16 & 0b11111100) << 3) | (g as u16 >> 3);
}
