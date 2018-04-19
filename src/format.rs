// pilcrow/src/format.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core_foundation::base::{CFType, TCFType};
use core_foundation::string::CFString;
use core_graphics::font::CGFont;
use core_text::font as ct_font;
use core_text::font::CTFont;
use core_text::font_descriptor::{kCTFontBoldTrait, kCTFontItalicTrait};
use std::mem;
use std::str::FromStr;

pub struct Format {
    pub(crate) key: CFString,
    pub(crate) value: CFType,
}

impl Format {
    pub fn from_font(font: Font) -> Format {
        Format {
            key: CFString::from_str("NSFont").unwrap(),
            value: font.native_font.as_CFType(),
        }
    }

    pub fn font(&self) -> Option<Font> {
        let key = self.key.to_string();
        unsafe {
            if key == "NSFont" {
                Some(Font {
                    native_font: mem::transmute::<CFType, CTFont>(self.value.clone()),
                })
            } else {
                None
            }
        }
    }
}

#[derive(Clone)]
pub struct Font {
    native_font: CTFont,
}

impl Font {
    #[inline]
    pub fn from_native_font(native_font: CTFont) -> Font {
        Font {
            native_font
        }
    }

    pub fn default_serif() -> Font {
        Font::from_native_font(ct_font::new_from_name("Times", 16.0).unwrap())
    }

    pub fn default_monospace() -> Font {
        Font::from_native_font(ct_font::new_from_name("Menlo", 12.0).unwrap())
    }

    #[inline]
    pub fn id(&self) -> FontId {
        FontId::from_native_font(self.native_font.clone())
    }

    #[inline]
    pub fn face_id(&self) -> FontFaceId {
        FontFaceId::from_native_font(self.native_font.clone())
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.native_font.pt_size() as f32
    }

    #[inline]
    pub fn native_font(&self) -> CTFont {
        self.native_font.clone()
    }

    pub fn to_size(&self, new_size: f32) -> Font {
        Font::from_native_font(self.native_font.clone_with_font_size(new_size as f64))
    }

    pub fn to_bold(&self) -> Font {
        Font::from_native_font(self.native_font.clone_with_symbolic_traits(kCTFontBoldTrait,
                                                                           kCTFontBoldTrait))
    }

    pub fn to_italic(&self) -> Font {
        Font::from_native_font(self.native_font.clone_with_symbolic_traits(kCTFontItalicTrait,
                                                                           kCTFontItalicTrait))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FontFaceId(usize);

impl FontFaceId {
    fn from_native_font(font: CTFont) -> FontFaceId {
        unsafe {
            FontFaceId(mem::transmute_copy::<CGFont, usize>(&font.copy_to_CGFont()))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FontId(usize);

impl FontId {
    fn from_native_font(font: CTFont) -> FontId {
        unsafe {
            FontId(mem::transmute_copy::<CTFont, usize>(&font))
        }
    }
}
