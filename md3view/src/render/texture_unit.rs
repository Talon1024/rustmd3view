use std::ops::{Deref, DerefMut};
use crate::render::MAX_TEXTURE_UNITS;

#[derive(Debug, Clone, Copy)]
pub struct TextureUnit(pub u8);

impl Default for TextureUnit {
    fn default() -> Self {
        Self(1)
    }
}

impl Deref for TextureUnit {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for TextureUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TextureUnit {
    #[inline]
    pub fn max() -> u8 {
        MAX_TEXTURE_UNITS.get().copied().unwrap_or(32)
    }
    pub fn slot(self) -> u32 {
        let max_unit = Self::max();
        match self.0 {
            // All OpenGL implementations have at least 16 texture slots
            // available
            0 => 0,
            1 => glow::TEXTURE0,
            2 => glow::TEXTURE1,
            3 => glow::TEXTURE2,
            4 => glow::TEXTURE3,
            5 => glow::TEXTURE4,
            6 => glow::TEXTURE5,
            7 => glow::TEXTURE6,
            8 => glow::TEXTURE7,
            9 => glow::TEXTURE8,
            10 => glow::TEXTURE9,
            11 => glow::TEXTURE10,
            12 => glow::TEXTURE11,
            13 => glow::TEXTURE12,
            14 => glow::TEXTURE13,
            15 => glow::TEXTURE14,
            16 => glow::TEXTURE15,
            x if x == 17 && x < max_unit => glow::TEXTURE16,
            x if x == 18 && x < max_unit => glow::TEXTURE17,
            x if x == 19 && x < max_unit => glow::TEXTURE18,
            x if x == 20 && x < max_unit => glow::TEXTURE19,
            x if x == 21 && x < max_unit => glow::TEXTURE20,
            x if x == 22 && x < max_unit => glow::TEXTURE21,
            x if x == 23 && x < max_unit => glow::TEXTURE22,
            x if x == 24 && x < max_unit => glow::TEXTURE23,
            x if x == 25 && x < max_unit => glow::TEXTURE24,
            x if x == 26 && x < max_unit => glow::TEXTURE25,
            x if x == 27 && x < max_unit => glow::TEXTURE26,
            x if x == 28 && x < max_unit => glow::TEXTURE27,
            x if x == 29 && x < max_unit => glow::TEXTURE28,
            x if x == 30 && x < max_unit => glow::TEXTURE29,
            x if x == 31 && x < max_unit => glow::TEXTURE30,
            x if x == 32 && x < max_unit => glow::TEXTURE31,
            x => glow::TEXTURE0 + x.min(max_unit) as u32,
        }
    }
    pub fn uniform(self) -> i32 {
        let max_unit = Self::max();
        match self.0 {
            x if x >= 1 && x <= max_unit => x - 1,
            _ => 0,
        }
        .into()
    }
    pub fn next(&mut self) -> () {
        self.0 += 1;
    }
}