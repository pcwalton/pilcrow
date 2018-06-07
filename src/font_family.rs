// pilcrow/src/font_family.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use minikin_sys::{minikin_font_family_create, minikin_font_family_destroy, minikin_font_family_t};
use std::iter;
use std::mem;
use std::sync::Arc;

use font::FontLike;
use platform::Font;

pub struct FontFamily {
    minikin_font_family: *mut minikin_font_family_t,
}

impl Drop for FontFamily {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            assert!(!self.minikin_font_family.is_null());
            minikin_font_family_destroy(self.minikin_font_family);
        }
    }
}

impl FontFamily {
    /// Creates a font family from the given fonts.
    pub fn from_fonts<I>(fonts: I) -> FontFamily where I: Iterator<Item = Font> {
        let mut minikin_fonts = vec![];
        for font in fonts {
            minikin_fonts.push(Font::into_minikin_font(Box::new(font)))
        }
        let minikin_fonts_ptr = minikin_fonts.as_mut_ptr();
        unsafe {
            FontFamily {
                minikin_font_family: minikin_font_family_create(minikin_fonts_ptr,
                                                                minikin_fonts.len()),
            }
        }
    }

    /// A convenience method that calls `FontFamily::from_fonts()` with a single font.
    #[inline]
    pub fn from_font(font: Font) -> FontFamily {
        FontFamily::from_fonts(iter::once(font))
    }

    pub(crate) fn as_minikin_font_family(&self) -> *mut minikin_font_family_t {
        self.minikin_font_family
    }
}
