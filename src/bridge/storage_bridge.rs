#![allow(nonstandard_style)]
#![allow(dead_code)]

use core::ffi::{c_char, c_uchar, c_void};

#[repr(C)]
pub enum FMODE {
    FA_READ = 0x01,
    FA_WRITE = 0x02,
    FA_OPEN_EXISTING = 0x00,
    FA_CREATE_NEW = 0x04,
    FA_CREATE_ALWAYS = 0x08,
    FA_OPEN_ALWAYS = 0x10,
    FA_OPEN_APPEND = 0x30,
}

#[repr(C)]
pub enum FRESULT {
    FR_OK,
    FR_DISK_ERR,
    FR_INT_ERR,
    FR_NOT_READY,
    FR_NO_FILE,
    FR_NO_PATH,
    FR_INVALID_NAME,
    FR_DENIED,
    FR_EXIST,
    FR_INVALID_OBJECT,
    FR_WRITE_PROTECTED,
    FR_INVALID_DRIVE,
    FR_NOT_ENABLED,
    FR_NO_FILESYSTEM,
    FR_MKFS_ABORTED,
    FR_TIMEOUT,
    FR_LOCKED,
    FR_NOT_ENOUGH_CORE,
    FR_TOO_MANY_OPEN_FILES,
    FR_INVALID_PARAMETER,
}

#[repr(C)]
pub struct FIL {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Storage {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn File_new() -> *mut FIL;
    pub fn File_free(file: *mut FIL);

    pub fn File_raw_open(file: *mut FIL, path: *const c_char, mode: c_uchar) -> FRESULT;
    pub fn File_raw_close(file: *mut FIL) -> FRESULT;

    pub fn Storage_new() -> *mut Storage;
    pub fn Storage_free(storage: *mut Storage);

    pub fn Storage_is_ready(storage: *mut Storage) -> bool;
    pub fn Storage_mount(storage: *mut Storage) -> FRESULT;
    pub fn Storage_unmount(storage: *mut Storage) -> FRESULT;
    pub fn Storage_raw_write(storage: *mut Storage, file: *mut FIL, text: *const c_char)
    -> FRESULT;
    pub fn Storage_write(
        storage: *mut Storage,
        path: *const c_char,
        text: *const c_char,
    ) -> FRESULT;
    pub fn Storage_raw_read(
        storage: *mut Storage,
        file: *mut FIL,
        buffer: *mut c_void,
        buffer_size: u64,
    ) -> FRESULT;
    pub fn Storage_read(
        storage: *mut Storage,
        path: *const c_char,
        buffer: *mut c_void,
        buffer_size: u64,
        offset: u64,
    ) -> FRESULT;
}
