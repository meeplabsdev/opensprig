use crate::bridge::button_bridge::*;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum ButtonType {
    BOOTSEL,
    L_UP = 5,
    L_DOWN = 7,
    L_LEFT = 6,
    L_RIGHT = 8,
    R_UP = 12,
    R_DOWN = 14,
    R_LEFT = 13,
    R_RIGHT = 15,
}

pub struct Button {
    ptr: *mut _Button,
}

impl Button {
    pub fn new(pin: ButtonType) -> Self {
        let ptr = unsafe { Button_new(pin as u32) };
        Button { ptr }
    }
}

impl Drop for Button {
    fn drop(&mut self) {
        unsafe { Button_free(self.ptr) };
    }
}

impl Button {
    pub fn is_pressed(&mut self) -> bool {
        unsafe { Button_is_pressed(self.ptr) }
    }

    pub fn is_long_pressed(&mut self) -> bool {
        unsafe { Button_is_long_pressed(self.ptr) }
    }

    pub fn was_pressed(&mut self) -> bool {
        unsafe { Button_was_pressed(self.ptr) }
    }

    pub fn was_long_pressed(&mut self) -> bool {
        unsafe { Button_was_long_pressed(self.ptr) }
    }
}
