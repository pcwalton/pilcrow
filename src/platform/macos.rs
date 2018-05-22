// pilcrow/src/platform/macos.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core_text::font::CTFont;

use font::{Font, FontAttributes};

pub struct PlatformFont {
    core_text_font: CTFont,
}

impl PlatformFont {
    #[inline]
    pub fn new(core_text_font: CTFont) -> PlatformFont {
        PlatformFont {
            core_text_font,
        }
    }
}

impl Font for PlatformFont {
    fn get_horizontal_advance(&self, glyph_id: u32, attributes: &FontAttributes) -> f32 {
        self.core_text_font.get_advances_for_glyphs(&[glyph_id])
    }
}
