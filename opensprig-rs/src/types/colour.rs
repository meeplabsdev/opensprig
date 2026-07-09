pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn as_16bit(&self) -> u16 {
        rgb(self.r, self.g, self.b)
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u16 {
    let r = ((r as f32 / 255f32) * 31f32) as u8;
    let b = ((b as f32 / 255f32) * 31f32) as u8;
    let g = ((g as f32 / 255f32) * 63f32) as u8;

    ((r as u16 & 0b11111000) << 8) | ((b as u16 & 0b11111100) << 3) | (g as u16 >> 3)
}
