use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{ErrorType, Operation, SpiBus, SpiDevice};
use embedded_hal_bus::spi::DeviceError;

/// `embassy_sync::blocking_mutex` `Mutex`-based shared bus [`SpiDevice`] implementation.
///
/// This allows for sharing an [`SpiBus`], obtaining multiple [`SpiDevice`] instances,
/// each with its own `CS` pin.
pub struct MutexDevice<'a, BUS, CS, D> {
    bus: &'a Mutex<ThreadModeRawMutex, BUS>,
    cs: CS,
    delay: D,
}

impl<'a, BUS, CS, D> MutexDevice<'a, BUS, CS, D> {
    /// Create a new [`MutexDevice`].
    ///
    /// This sets the `cs` pin high, and returns an error if that fails. It is recommended
    /// to set the pin high the moment it's configured as an output, to avoid glitches.
    #[inline]
    pub fn new(
        bus: &'a Mutex<ThreadModeRawMutex, BUS>,
        mut cs: CS,
        delay: D,
    ) -> Result<Self, CS::Error>
    where
        CS: OutputPin,
    {
        cs.set_high()?;
        Ok(Self { bus, cs, delay })
    }
}

impl<BUS, CS, D> ErrorType for MutexDevice<'_, BUS, CS, D>
where
    BUS: ErrorType,
    CS: OutputPin,
{
    type Error = DeviceError<BUS::Error, CS::Error>;
}

impl<Word: Copy + 'static, BUS, CS, D> SpiDevice<Word> for MutexDevice<'_, BUS, CS, D>
where
    BUS: SpiBus<Word>,
    CS: OutputPin,
    D: DelayNs,
{
    #[inline]
    fn transaction(&mut self, operations: &mut [Operation<'_, Word>]) -> Result<(), Self::Error> {
        unsafe {
            self.bus
                .lock_mut(|bus| transaction(operations, bus, &mut self.delay, &mut self.cs))
        }
    }
}

// vvv this is private in the lib so i guess its here now

#[inline]
fn transaction<Word, BUS, CS, D>(
    operations: &mut [Operation<Word>],
    bus: &mut BUS,
    delay: &mut D,
    cs: &mut CS,
) -> Result<(), DeviceError<BUS::Error, CS::Error>>
where
    BUS: SpiBus<Word> + ErrorType,
    CS: OutputPin,
    D: DelayNs,
    Word: Copy,
{
    cs.set_low().map_err(DeviceError::Cs)?;

    let op_res = operations.iter_mut().try_for_each(|op| match op {
        Operation::Read(buf) => bus.read(buf),
        Operation::Write(buf) => bus.write(buf),
        Operation::Transfer(read, write) => bus.transfer(read, write),
        Operation::TransferInPlace(buf) => bus.transfer_in_place(buf),
        Operation::DelayNs(ns) => {
            bus.flush()?;
            delay.delay_ns(*ns);
            Ok(())
        }
    });

    // On failure, it's important to still flush and deassert CS.
    let flush_res = bus.flush();
    let cs_res = cs.set_high();

    op_res.map_err(DeviceError::Spi)?;
    flush_res.map_err(DeviceError::Spi)?;
    cs_res.map_err(DeviceError::Cs)?;

    Ok(())
}
