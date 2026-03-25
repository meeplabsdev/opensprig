use core::fmt::{Debug, Display};
use embassy_rp::spi;

pub struct Error {
    inner: &'static str,
}

impl Error {
    pub fn new(reason: &'static str) -> Self {
        Self { inner: reason }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.inner)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.inner)
    }
}

impl From<spi::Error> for Error {
    fn from(_: spi::Error) -> Self {
        // No errors for now
        Self { inner: "SPI Error" }
    }
}
