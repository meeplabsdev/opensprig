#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_rp as _;
use panic_probe as _;

// #[macro_export]
// macro_rules! aligned_flash {
//     ($addr:expr, $size:expr) => {{
//         {
//             static BYTES: &[u8; $size] = include_flash!($addr, $size);
//             static ALIGNED_BYTES: &cyw43::Aligned<cyw43::A4, [u8; $size]> = &cyw43::Aligned(*BYTES);
//             ALIGNED_BYTES
//         }
//     }};
// }

// #[macro_export]
// macro_rules! include_flash {
//     ($addr:expr, $size:expr) => {{
//         {
//             static BYTES: &[u8] = unsafe { core::slice::from_raw_parts($addr as *const u8, $size) };
//             static SIZED_BYTES: &[u8; $size] = unsafe { &*(BYTES.as_ptr() as *const [u8; $size]) };
//             SIZED_BYTES
//         }
//     }};
// }

#[macro_export]
macro_rules! aligned_flash {
    ($addr:expr, $size:expr) => {{
        {
            fn fw() -> &'static cyw43::Aligned<cyw43::A4, [u8; $size]> {
                unsafe { &*($addr as *const _) }
            }

            fw()
        }
    }};
}

#[macro_export]
macro_rules! include_flash {
    ($addr:expr, $size:expr) => {{
        {
            fn fw() -> &'static [u8; $size] {
                unsafe { &*($addr as *const _) }
            }

            fw()
        }
    }};
}

#[macro_export]
macro_rules! fw {
    () => {{ opensprig_rs::aligned_flash!(0x101be000, 0x386A5) }};
}

#[macro_export]
macro_rules! clm {
    () => {{ opensprig_rs::include_flash!(0x101fe000, 0x003D8) }};
}

#[macro_export]
macro_rules! nvram {
    () => {{ opensprig_rs::aligned_flash!(0x101ff000, 0x002E6) }};
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt::{assert, *};

    #[test]
    fn assert_true() {
        info!("This test passes.");
        assert!(true);
    }
}
