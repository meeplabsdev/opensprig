#![no_std]
extern crate alloc;

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use anyhow::{Context, Result, anyhow};
use core::str::Lines;

struct FontParser<'a> {
    lines: Option<Lines<'a>>,

    within_font: bool,
    within_properties: bool,
    within_glyphs: bool,
    within_glyph: bool,

    name: &'a str,
    size: (usize, usize, usize),
    fontboundingbox: Option<(usize, usize, isize, isize)>,
    metricsset: usize,
    swidth: Option<(usize, usize)>,
    dwidth: Option<(usize, usize)>,
    swidth1: Option<(usize, usize)>,
    dwidth1: Option<(usize, usize)>,
    vvector: Option<(usize, usize)>,

    expected_glyphs: usize,
    glyphs: BTreeMap<isize, Glyph>,

    glyph_encoding: isize,
    glyph_swidth: Option<(usize, usize)>,
    glyph_dwidth: Option<(usize, usize)>,
    glyph_swidth1: Option<(usize, usize)>,
    glyph_dwidth1: Option<(usize, usize)>,
    glyph_vvector: Option<(usize, usize)>,
    glyph_bbx: Option<(usize, usize, isize, isize)>,
    glyph_data: Vec<u8>,
}

impl<'a> FontParser<'a> {
    fn new() -> Self {
        Self {
            lines: None,

            within_font: false,
            within_properties: false,
            within_glyphs: false,
            within_glyph: false,

            name: "Unknown",
            size: (0, 0, 0),
            fontboundingbox: None,
            metricsset: 0,
            swidth: None,
            dwidth: None,
            swidth1: None,
            dwidth1: None,
            vvector: None,

            expected_glyphs: 0,
            glyphs: BTreeMap::new(),

            glyph_encoding: 0,
            glyph_swidth: None,
            glyph_dwidth: None,
            glyph_swidth1: None,
            glyph_dwidth1: None,
            glyph_vvector: None,
            glyph_bbx: None,
            glyph_data: Vec::new(),
        }
    }

    fn parse(&mut self, content: &'a str) -> Result<Font<'a>> {
        self.lines = Some(content.lines());
        while let Some(line) = self.lines.as_mut().and_then(|l| l.next()) {
            let keys = line
                .split_once(|c: char| c.is_whitespace())
                .unwrap_or((line, ""));

            self.parse_directive(keys.0, keys.1)?;
        }

        let font = Font {
            name: self.name,
            point: self.size,
            ascent: self
                .fontboundingbox
                .context("FONTBOUNDINGBOX expected but not found")?
                .1 as isize,
            glyphs: self.glyphs.clone(),
        };

        Ok(font)
    }

    fn parse_directive(&mut self, key: &str, value: &'a str) -> Result<()> {
        match key {
            "COMMENT" | "CONTENTVERSION" => Ok(()),

            "STARTPROPERTIES" => self.startproperties(value),
            "ENDPROPERTIES" => self.endproperties(),
            _ if self.within_properties => Ok(()), // TODO: parse properties?

            "STARTFONT" => self.startfont(),
            "ENDFONT" => self.endfont(),
            "FONT" => self.font(value),
            "SIZE" => self.size(value),
            "FONTBOUNDINGBOX" => self.fontboundingbox(value),
            "METRICSSET" => self.metricsset(value),
            "SWIDTH" => self.swidth(value),
            "DWIDTH" => self.dwidth(value),
            "SWIDTH1" => self.swidth1(value),
            "DWIDTH1" => self.dwidth1(value),
            "VVECTOR" => self.vvector(value),

            "CHARS" => self.chars(value),
            "STARTCHAR" => self.startchar(),
            "ENDCHAR" => self.endchar(),
            "ENCODING" => self.encoding(value),
            "BBX" => self.bbx(value),
            "BITMAP" => self.bitmap(),

            _ => Err(anyhow!("unknown key {key}")),
        }
    }

    fn startfont(&mut self) -> Result<()> {
        if self.within_font {
            return Err(anyhow!(
                "Saw STARTFONT directive when font was already started"
            ));
        }

        self.within_font = true;

        Ok(())
    }

    fn font(&mut self, value: &'a str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw FONT directive before font started"));
        }

        self.name = value;

        Ok(())
    }

    fn size(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw SIZE directive before font started"));
        }

        let mut vals = value.split_whitespace();
        self.size = (
            usize::from_str_radix(vals.next().context("SIZE directive missing PointSize")?, 10)?,
            usize::from_str_radix(vals.next().context("SIZE directive missing Xres")?, 10)?,
            usize::from_str_radix(vals.next().context("SIZE directive missing Yres")?, 10)?,
        );

        Ok(())
    }

    fn fontboundingbox(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw FONT directive before font started"));
        }

        let mut vals = value.split_whitespace();
        self.fontboundingbox = Some((
            usize::from_str_radix(
                vals.next()
                    .context("FONTBOUNDINGBOX directive missing FBBx")?,
                10,
            )?,
            usize::from_str_radix(
                vals.next()
                    .context("FONTBOUNDINGBOX directive missing FBBy")?,
                10,
            )?,
            isize::from_str_radix(
                vals.next()
                    .context("FONTBOUNDINGBOX directive missing Xoff")?,
                10,
            )?,
            isize::from_str_radix(
                vals.next()
                    .context("FONTBOUNDINGBOX directive missing Yoff")?,
                10,
            )?,
        ));

        Ok(())
    }

    fn metricsset(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw FONT directive before font started"));
        }

        self.metricsset = usize::from_str_radix(value, 10)?;

        Ok(())
    }

    fn swidth(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw SWIDTH directive before font started"));
        }

        let mut vals = value.split_whitespace();
        let res = Some((
            usize::from_str_radix(vals.next().context("SWIDTH directive missing swx0")?, 10)?,
            usize::from_str_radix(vals.next().context("SWIDTH directive missing swy0")?, 10)?,
        ));

        if self.within_glyph {
            self.glyph_swidth = res;
        } else if !self.within_glyphs {
            self.swidth = res;
        } else {
            return Err(anyhow!("SWIDTH within CHARS but not STARTCHAR"));
        }

        Ok(())
    }

    fn dwidth(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw DWIDTH directive before font started"));
        }

        let mut vals = value.split_whitespace();
        let res = Some((
            usize::from_str_radix(vals.next().context("DWIDTH directive missing dwx0")?, 10)?,
            usize::from_str_radix(vals.next().context("DWIDTH directive missing dwy0")?, 10)?,
        ));

        if self.within_glyph {
            self.glyph_dwidth = res;
        } else if !self.within_glyphs {
            self.dwidth = res;
        } else {
            return Err(anyhow!("DWIDTH within CHARS but not STARTCHAR"));
        }

        Ok(())
    }

    fn swidth1(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw SWIDTH1 directive before font started"));
        }

        let mut vals = value.split_whitespace();
        let res = Some((
            usize::from_str_radix(vals.next().context("SWIDTH1 directive missing swx1")?, 10)?,
            usize::from_str_radix(vals.next().context("SWIDTH1 directive missing swy1")?, 10)?,
        ));

        if self.within_glyph {
            self.glyph_swidth1 = res;
        } else if !self.within_glyphs {
            self.swidth1 = res;
        } else {
            return Err(anyhow!("SWIDTH1 within CHARS but not STARTCHAR"));
        }

        Ok(())
    }

    fn dwidth1(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw DWIDTH1 directive before font started"));
        }

        let mut vals = value.split_whitespace();
        let res = Some((
            usize::from_str_radix(vals.next().context("DWIDTH1 directive missing dwx1")?, 10)?,
            usize::from_str_radix(vals.next().context("DWIDTH1 directive missing dwy1")?, 10)?,
        ));

        if self.within_glyph {
            self.glyph_dwidth1 = res;
        } else if !self.within_glyphs {
            self.dwidth1 = res;
        } else {
            return Err(anyhow!("DWIDTH1 within CHARS but not STARTCHAR"));
        }

        Ok(())
    }

    fn vvector(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw VVECTOR directive before font started"));
        }

        let mut vals = value.split_whitespace();
        let res = Some((
            usize::from_str_radix(vals.next().context("VVECTOR directive missing xoff")?, 10)?,
            usize::from_str_radix(vals.next().context("VVECTOR directive missing yoff")?, 10)?,
        ));

        if self.within_glyph {
            self.glyph_vvector = res;
        } else if !self.within_glyphs {
            self.vvector = res;
        } else {
            return Err(anyhow!("VVECTOR within CHARS but not STARTCHAR"));
        }

        Ok(())
    }

    fn startproperties(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw STARTPROPERTIES directive before font started"));
        } else if self.within_properties {
            return Err(anyhow!(
                "Saw STARTPROPERTIES directive when properties were already started"
            ));
        }

        self.within_properties = true;
        if let Some(lines) = self.lines.as_mut() {
            let _ = lines.take(usize::from_str_radix(value, 10)?);
        }

        Ok(())
    }

    fn endproperties(&mut self) -> Result<()> {
        if !self.within_properties {
            return Err(anyhow!(
                "Saw ENDPROPERTIES directive before properties started"
            ));
        }

        self.within_properties = false;

        Ok(())
    }

    fn chars(&mut self, value: &str) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw CHARS directive before font started"));
        } else if self.within_glyphs {
            return Err(anyhow!(
                "Saw CHARS directive when glyphs were already started"
            ));
        }

        self.within_glyphs = true;
        self.expected_glyphs = usize::from_str_radix(value, 10)?;

        Ok(())
    }

    fn startchar(&mut self) -> Result<()> {
        if !self.within_glyphs {
            return Err(anyhow!("Saw STARTCHAR directive before glyphs started"));
        } else if self.within_glyph {
            return Err(anyhow!(
                "Saw STARTCHAR directive when a glyph was already started"
            ));
        }

        self.within_glyph = true;

        Ok(())
    }

    fn encoding(&mut self, value: &str) -> Result<()> {
        if !self.within_glyph {
            return Err(anyhow!("Saw ENCODING directive outside a glyph"));
        }

        let mut vals = value.split_whitespace();
        self.glyph_encoding = isize::from_str_radix(
            vals.next().context("ENCODING directive missing integer")?,
            10,
        )?;

        // if self.glyph_encoding == -1 {
        //     self.glyph_encoding *= isize::from_str_radix(
        //         vals.next().context("ENCODING directive missing integer")?,
        //         10,
        //     )?
        //     .abs();
        // }

        Ok(())
    }

    fn bbx(&mut self, value: &str) -> Result<()> {
        if !self.within_glyph {
            return Err(anyhow!("Saw BBX directive outside a glyph"));
        }

        let mut vals = value.split_whitespace();
        self.glyph_bbx = Some((
            usize::from_str_radix(vals.next().context("BBX directive missing BBw")?, 10)?,
            usize::from_str_radix(vals.next().context("BBX directive missing BBh")?, 10)?,
            isize::from_str_radix(vals.next().context("BBX directive missing BBxoff0x")?, 10)?,
            isize::from_str_radix(vals.next().context("BBX directive missing BByoff0y")?, 10)?,
        ));

        Ok(())
    }

    fn bitmap(&mut self) -> Result<()> {
        if !self.within_glyph {
            return Err(anyhow!("Saw BITMAP directive outside a glyph"));
        }

        let Some(bbx) = self.glyph_bbx else {
            return Err(anyhow!("Saw BITMAP directive before BBX"));
        };

        let height = bbx.1;
        let Some(vals) = self.lines.as_mut().map(|l| l.take(height)) else {
            return Err(anyhow!(
                "something has gone horribly wrong - lines was None while processing BITMAP directive line"
            ));
        };

        for val in vals {
            let v = match hex::decode(val) {
                Err(e) => return Err(anyhow!("{e:?}")),
                Ok(v) => v,
            };

            self.glyph_data.extend(v);
        }

        Ok(())
    }

    fn endchar(&mut self) -> Result<()> {
        if !self.within_glyph {
            return Err(anyhow!("Saw ENDCHAR directive without one started"));
        }

        if let Some(swidth) = self.glyph_swidth.or(self.swidth)
            && let Some(dwidth) = self.glyph_dwidth.or(self.dwidth)
            && let Some(bbx) = self.glyph_bbx.or(self.fontboundingbox)
        {
            let (swidth1, dwidth1) = if self.metricsset > 0 {
                (
                    self.glyph_swidth1.or(self.swidth1).ok_or(anyhow!(
                        "Expected SWIDTH1 for writing direction > 0 but did not find it"
                    ))?,
                    self.glyph_dwidth1.or(self.dwidth1).ok_or(anyhow!(
                        "Expected DWIDTH1 for writing direction > 0 but did not find it"
                    ))?,
                )
            } else {
                ((0, 0), (0, 0))
            };

            // TODO: THIS IS WRONG I DID METRICSET > 0 ALL WRONG
            let bbxoff1 = if self.metricsset == 2 {
                let vvector = self.glyph_vvector.or(self.vvector).ok_or(anyhow!(
                    "Expected VVECTOR for writing direction == 0 but did not find it"
                ))?;
                (bbx.2 - vvector.0 as isize, bbx.3 - vvector.1 as isize)
            } else {
                (0, 0)
            };

            let offset = if self.metricsset == 0 {
                (bbx.2, bbx.3)
            // TODO: THIS IS WRONG I DID METRICSET > 0 ALL WRONG
            } else if self.metricsset == 1 {
                bbxoff1
            } else {
                (bbx.2 + bbxoff1.0, bbx.3 + bbxoff1.1)
            };

            self.glyphs.insert(
                self.glyph_encoding,
                Glyph {
                    swidth,
                    dwidth,
                    swidth1,
                    dwidth1,
                    size: (bbx.0, bbx.1),
                    offset,
                    data: self.glyph_data.clone(),
                },
            );
        }

        self.within_glyph = false;
        self.glyph_encoding = 0;
        self.glyph_swidth = None;
        self.glyph_dwidth = None;
        self.glyph_swidth1 = None;
        self.glyph_dwidth1 = None;
        self.glyph_vvector = None;
        self.glyph_bbx = None;
        self.glyph_data.clear();

        Ok(())
    }

    fn endfont(&mut self) -> Result<()> {
        if !self.within_font {
            return Err(anyhow!("Saw ENDFONT directive without starting the font"));
        }

        // if self.glyphs.len() != self.expected_glyphs {
        //     return Err(anyhow!(
        //         "The number of glyphs found was not equal to the number passed to CHARS"
        //     ));
        // }

        self.within_glyphs = false;
        self.within_font = false;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Font<'a> {
    pub name: &'a str,
    pub point: (usize, usize, usize),
    pub ascent: isize,
    glyphs: BTreeMap<isize, Glyph>,
}

impl<'a> Font<'a> {
    pub fn parse(content: &'a str) -> Result<Self> {
        FontParser::new().parse(content)
    }

    pub fn glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(&(c as usize as isize))
    }

    pub fn special_glyph(&self, i: usize) -> Option<&Glyph> {
        self.glyphs.get(&-(i as isize))
    }

    pub fn pixel(&self, glyph: &Glyph, x: usize, y: usize) -> Result<bool> {
        let row = y * ((glyph.size.0.min(1) - 1) / 8 + 1);
        let byte_idx = row + x / 8;
        let byte_rem = x % 8;

        let Some(byte) = glyph.data.get(byte_idx) else {
            return Ok(false);
        };
        let bit = (byte << byte_rem) & 0b10000000;

        Ok(bit == 0b10000000)
    }
}

#[derive(Clone)]
pub struct Glyph {
    pub swidth: (usize, usize),
    pub dwidth: (usize, usize),
    pub swidth1: (usize, usize),
    pub dwidth1: (usize, usize),
    pub size: (usize, usize),
    pub offset: (isize, isize),
    data: Vec<u8>,
}
