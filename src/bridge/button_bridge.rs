#[repr(C)]
pub struct _Button {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn Button_new(pin: u32) -> *mut _Button;
    pub fn Button_free(button: *mut _Button);

    pub fn Button_is_pressed(button: *mut _Button) -> bool;
    pub fn Button_is_long_pressed(button: *mut _Button) -> bool;
    pub fn Button_was_pressed(button: *mut _Button) -> bool;
    pub fn Button_was_long_pressed(button: *mut _Button) -> bool;
}
