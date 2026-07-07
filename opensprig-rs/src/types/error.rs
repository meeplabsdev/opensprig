use core::fmt::{Debug, Display};

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

impl From<embassy_rp::spi::Error> for Error {
    fn from(_: embassy_rp::spi::Error) -> Self {
        Self { inner: "SPI Error" }
    }
}

impl From<embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> for Error {
    fn from(_: embedded_sdmmc::Error<embedded_sdmmc::SdCardError>) -> Self {
        Self {
            inner: "SD Card Error",
        }
    }
}

impl From<embedded_sdmmc::SdCardError> for Error {
    fn from(_: embedded_sdmmc::SdCardError) -> Self {
        Self {
            inner: "SD Card Error",
        }
    }
}
