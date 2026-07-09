#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::flash::{Blocking, ERASE_SIZE, Flash};
use embassy_rp::pac;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, FLASH, PIO0, SPI0};
use embassy_rp::pio::InterruptHandler;
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use opensprig_rs::hardware::Storage;
use opensprig_rs::init;
use opensprig_rs::types::{Colour, Error};

const PROGRAM_ADDRESS: u32 = 0x1008_0000;
const PROGRAM_LENGTH: u32 = 0x13_E000;
const PROGRAM_OFFSET: u32 = PROGRAM_ADDRESS - 0x1000_0000;
const TOTAL_FLASH_SIZE: usize = 2 * 1024 * 1024;
const BOOT_PROGRAM_MAGIC: u32 = 0x0550_1EAD;
const READ_CHUNK_SIZE: usize = 4096;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
});

fn boot() -> ! {
    cortex_m::interrupt::disable();

    unsafe {
        (*cortex_m::peripheral::NVIC::PTR).icer[0].write(0xFFFF_FFFF);
        (*cortex_m::peripheral::NVIC::PTR).icpr[0].write(0xFFFF_FFFF);

        let scb = &*cortex_m::peripheral::SCB::PTR;
        scb.vtor.write(PROGRAM_ADDRESS);
    }

    unsafe { cortex_m::interrupt::enable() };
    unsafe { cortex_m::asm::bootload(PROGRAM_ADDRESS as *const u32) };
}

async fn load(
    storage: &mut Storage<'static, SPI0>,
    mut flash: Flash<'static, FLASH, Blocking, TOTAL_FLASH_SIZE>,
    path: &str,
) -> Result<u32, Error> {
    let size = storage.file_size(path).await?;
    if size == 0 {
        return Err(Error::new("target is empty"));
    }

    let erase_len = (size + (ERASE_SIZE as u32 - 1)) & !(ERASE_SIZE as u32 - 1);
    if erase_len > PROGRAM_LENGTH {
        return Err(Error::new("target too large for PROGRAM partition"));
    }

    flash.blocking_erase(PROGRAM_OFFSET, PROGRAM_OFFSET + erase_len)?;

    let mut buffer = [0u8; READ_CHUNK_SIZE];
    let written = storage
        .read_into(path, &mut buffer, |offset, data| {
            flash.blocking_write(PROGRAM_OFFSET + offset, data)?;
            Ok(())
        })
        .await?;

    Ok(written)
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if pac::WATCHDOG.scratch0().read() == BOOT_PROGRAM_MAGIC {
        pac::WATCHDOG.scratch0().write(|w| *w = 0);
        boot();
    }

    let p = embassy_rp::init(Default::default());

    let (
        mut w,
        _net_device,
        _control,
        screen,
        _speaker,
        storage,
        _button_l_up,
        _button_l_left,
        _button_l_down,
        _button_l_right,
        _button_r_up,
        _button_r_left,
        _button_r_down,
        _button_r_right,
        _led_left,
        _led_right,
    ) = init!(spawner, p);

    screen.draw_flood(&Colour::new(0, 0, 0)).await;
    screen
        .draw_text(&Colour::new(255, 255, 255), "OpenSprig", 5, 5)
        .await
        .unwrap();
    screen.blit().await.unwrap();

    match load(storage, Flash::new_blocking(p.FLASH), "program.bin").await {
        Ok(size) => {
            info!("Loaded ({} bytes). Rebooting into program.", size);

            screen.draw_flood(&Colour::new(0, 0, 0)).await;
            screen.blit().await.unwrap();

            Timer::after_millis(10).await;
            pac::WATCHDOG.scratch0().write(|w| *w = BOOT_PROGRAM_MAGIC);
            w.trigger_reset();
            loop {}
        }
        Err(e) => {
            error!("Failed to load: {}", Display2Format(&e));
        }
    }
}
