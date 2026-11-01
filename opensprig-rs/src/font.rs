use crate::types::Result;
use embassy_sync::once_lock::OnceLock;

pub use bdf_parser::Font;

static BDF_FOURBYSIX: OnceLock<Font> = OnceLock::new();
static BDF_SIXBYTEN: OnceLock<Font> = OnceLock::new();

pub async fn bdf_fourbysix() -> Result<&'static Font<'static>> {
    if BDF_FOURBYSIX.is_set() {
        return Ok(BDF_FOURBYSIX.get().await);
    }

    let f = Font::parse(include_str!("../../fonts/4x6.bdf"))?;
    Ok(BDF_FOURBYSIX.get_or_init(|| f))
}

pub async fn bdf_sixbyten() -> Result<&'static Font<'static>> {
    if BDF_SIXBYTEN.is_set() {
        return Ok(BDF_SIXBYTEN.get().await);
    }

    let f = Font::parse(include_str!("../../fonts/6x10.bdf"))?;
    Ok(BDF_SIXBYTEN.get_or_init(|| f))
}
