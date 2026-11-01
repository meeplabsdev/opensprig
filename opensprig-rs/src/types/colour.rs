pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn as_16bit(&self) -> u16 {
        rgb(self.r, self.g, self.b)
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u16 {
    let r = (r >> 3) as u16;
    let g = (g >> 2) as u16;
    let b = (b >> 3) as u16;

    (b << 11) | (g << 5) | r
}
