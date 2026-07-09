#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::flash::{Blocking, ERASE_SIZE, Flash};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pac;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH2, DMA_CH3, FLASH, PIO0, SPI0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::{bind_interrupts, dma};
use embassy_sync::{
    blocking_mutex::Mutex as BlockingMutex, blocking_mutex::raw::ThreadModeRawMutex,
};
use embassy_time::{Duration, Timer};
use opensprig_rs::hardware::{PwmLed, Screen, Storage};
use opensprig_rs::types::{Colour, Error};
use opensprig_rs::{clm, fw, nvram, pwm_led_a};
use static_cell::StaticCell;

/// Start address of the PROGRAM partition. Must match memory.x's PROGRAM
/// region, and parts/blink/memory.x's FLASH region.
const PROGRAM_ADDRESS: u32 = 0x1008_0000;
/// Size of the PROGRAM partition. Must match memory.x's PROGRAM region.
const PROGRAM_LENGTH: u32 = 0x13_E000;
/// PROGRAM_ADDRESS expressed as an offset from flash base, since embassy-rp's
/// Flash API is offset-relative rather than absolute-address-based.
const PROGRAM_OFFSET: u32 = PROGRAM_ADDRESS - 0x1000_0000;

const TOTAL_FLASH_SIZE: usize = 2 * 1024 * 1024;

/// Written to watchdog scratch register 0 to mean: "a program was just
/// flashed — on this boot, chain-load it instead of running loader logic."
/// One-shot: cleared the instant it's observed, so any *later* reset (from
/// the program itself, a button, whatever) falls back to the loader.
const BOOT_PROGRAM_MAGIC: u32 = 0x0550_1EAD;

const READ_CHUNK_SIZE: usize = 4096;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

/// Relocates the vector table to the PROGRAM partition and jumps into it.
/// Never returns. Must be called before any peripheral is touched by the
/// loader, since the program runs its own cortex-m-rt startup and re-inits
/// everything itself.
fn boot_into_program() -> ! {
    cortex_m::interrupt::disable();

    unsafe {
        (*cortex_m::peripheral::NVIC::PTR).icer[0].write(0xFFFF_FFFF);
        (*cortex_m::peripheral::NVIC::PTR).icpr[0].write(0xFFFF_FFFF);

        let scb = &*cortex_m::peripheral::SCB::PTR;
        scb.vtor.write(PROGRAM_ADDRESS);
    }

    // Safe to re-enable globally now: every external IRQ line was just
    // disabled above, so nothing can fire before blink's own init
    // explicitly re-enables what it needs via NVIC::unmask.
    unsafe { cortex_m::interrupt::enable() };

    unsafe { cortex_m::asm::bootload(PROGRAM_ADDRESS as *const u32) };
}

/// Shows an error on screen, blinks the status LED, and idles forever.
async fn fail(screen: &Screen<'static, SPI0>, led: &PwmLed<'static>, message: &str) -> ! {
    screen.draw_flood(&Colour::new(80, 0, 0)).await;
    let _ = screen
        .draw_text(&Colour::new(255, 255, 255), message, 5, 5)
        .await;
    let _ = screen.blit().await;
    led.blink(1.0 / 128.0).await;

    loop {
        Timer::after(Duration::from_secs(3600)).await;
    }
}

/// Erases the PROGRAM partition and streams program.bin from storage into it.
/// Returns the number of bytes written.
async fn load_program(
    storage: &mut Storage<'static, SPI0>,
    mut flash: Flash<'static, FLASH, Blocking, TOTAL_FLASH_SIZE>,
) -> Result<u32, Error> {
    let size = storage.file_size("program.bin").await?;
    if size == 0 {
        return Err(Error::new("program.bin is empty"));
    }

    let erase_len = (size + (ERASE_SIZE as u32 - 1)) & !(ERASE_SIZE as u32 - 1);
    if erase_len > PROGRAM_LENGTH {
        return Err(Error::new("program.bin too large for PROGRAM partition"));
    }

    flash.blocking_erase(PROGRAM_OFFSET, PROGRAM_OFFSET + erase_len)?;

    let mut buffer = [0u8; READ_CHUNK_SIZE];
    let written = storage
        .read_into("program.bin", &mut buffer, |offset, data| {
            flash.blocking_write(PROGRAM_OFFSET + offset, data)?;
            Ok(())
        })
        .await?;

    Ok(written)
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Check the one-shot boot flag before touching anything else. If a
    // program was just flashed, jump straight into it and never run the
    // rest of the loader.
    if pac::WATCHDOG.scratch0().read() == BOOT_PROGRAM_MAGIC {
        pac::WATCHDOG.scratch0().write(|w| *w = 0);
        boot_into_program();
    }

    info!("Normal boot: starting firmware loader.");

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let p = embassy_rp::init(Default::default());

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    let mut pio = Pio::new(p.PIO0, Irqs);
    let cyw43_spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        dma::Channel::new(p.DMA_CH0, Irqs),
    );

    let (_net_device, mut control, runner) =
        cyw43::new(state, pwr, cyw43_spi, fw!(), nvram!()).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));

    control.init(clm!()).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = 20_000_000;

    static SPI: StaticCell<BlockingMutex<ThreadModeRawMutex, Spi<'static, SPI0, Async>>> =
        StaticCell::new();
    let spi = SPI.init(BlockingMutex::new(Spi::new(
        p.SPI0, p.PIN_18, p.PIN_19, p.PIN_16, p.DMA_CH2, p.DMA_CH3, Irqs, spi_config,
    )));

    static SCREEN: StaticCell<Screen<'static, SPI0>> = StaticCell::new();
    let screen: &'static Screen<'static, SPI0> = SCREEN.init(
        Screen::new(spi, p.PIN_20, p.PIN_22, p.PIN_26, p.PIN_17)
            .await
            .unwrap(),
    );

    screen.draw_flood(&Colour::new(0, 0, 0)).await;
    let _ = screen
        .draw_text(&Colour::new(255, 255, 255), "OpenSprig Loader", 5, 5)
        .await;
    let _ = screen.blit().await;

    let led = pwm_led_a!(spawner, p.PWM_SLICE6, p.PIN_28);

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    let flash = Flash::<_, Blocking, TOTAL_FLASH_SIZE>::new_blocking(p.FLASH);

    let mut storage = match Storage::new(spi, p.PIN_21).await {
        Ok(s) => s,
        Err(e) => {
            error!("Storage init failed: {}", Display2Format(&e));
            fail(screen, led, "SD card error").await;
        }
    };

    if !storage.exists("program.bin").await {
        info!("No program.bin found on storage.");
        fail(screen, led, "No program.bin").await;
    }

    match load_program(&mut storage, flash).await {
        Ok(size) => {
            info!(
                "Loaded program.bin ({} bytes). Rebooting into program.",
                size
            );

            screen.draw_flood(&Colour::new(0, 0, 0)).await;
            let _ = screen
                .draw_text(&Colour::new(255, 255, 255), "Loading...", 5, 5)
                .await;
            let _ = screen.blit().await;

            Timer::after_millis(10).await;
            pac::WATCHDOG.scratch0().write(|w| *w = BOOT_PROGRAM_MAGIC);
            watchdog.trigger_reset();
            loop {}
        }
        Err(e) => {
            error!("Failed to load program.bin: {}", Display2Format(&e));
            fail(screen, led, "Load failed").await;
        }
    }
}
