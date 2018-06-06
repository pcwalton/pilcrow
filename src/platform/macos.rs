// pilcrow/src/platform/macos.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::CGFont;
use core_graphics::geometry::{CG_ZERO_SIZE, CGRect};
use core_text::font::CTFont;
use core_text::font_descriptor::{SymbolicTraitAccessors, kCTFontDefaultOrientation};
use euclid::{Point2D, Rect, Size2D};
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use core_text;
use font::{FontAttributes, FontLike};
use font_traits::FontTraits;

#[derive(Clone)]
pub struct Font {
    core_text_font: CTFont,
    font_data: Option<Arc<Vec<u8>>>,
}

impl FontLike for Font {
    type NativeFont = CTFont;

    fn from_bytes(font_data: Arc<Vec<u8>>) -> Result<Font, ()> {
        let data_provider = CGDataProvider::from_buffer(font_data.clone());
        let core_graphics_font = try!(CGFont::from_data_provider(data_provider).map_err(drop));
        let core_text_font = core_text::font::new_from_CGFont(&core_graphics_font, 16.0);
        Ok(Font {
            core_text_font,
            font_data: Some(font_data),
        })
    }

    fn from_native_font(core_text_font: CTFont) -> Font {
        let mut font_data = None;
        match core_text_font.url() {
            None => warn!("No URL found for Core Text font!"),
            Some(url) => {
                match url.to_path() {
                    Some(path) => {
                        match File::open(path) {
                            Ok(mut file) => {
                                let mut buffer = vec![];
                                match file.read_to_end(&mut buffer) {
                                    Err(_) => warn!("Could not read Core Text font from disk!"),
                                    Ok(_) => font_data = Some(Arc::new(buffer)),
                                }
                            }
                            Err(_) => warn!("Could not open file for Core Text font!"),
                        }
                    }
                    None => warn!("Could not convert URL from Core Text font to path!"),
                }
            }
        }

        Font {
            core_text_font,
            font_data,
        }
    }

    fn get_unique_id(&self) -> u32 {
        let mut hasher = DefaultHasher::new();
        (self as *const Font as usize).hash(&mut hasher);
        hasher.finish() as u32
    }

    fn get_horizontal_advance(&self, glyph_id: u32, attributes: &FontAttributes) -> f32 {
        let (glyph_id, mut advance) = (glyph_id as u16, CG_ZERO_SIZE);
        self.core_text_font_for_attributes(attributes)
            .get_advances_for_glyphs(kCTFontDefaultOrientation, &glyph_id, &mut advance, 1);
        advance.width as f32
    }

    fn get_bounds(&self, glyph_id: u32, attributes: &FontAttributes) -> Rect<f32> {
        let glyph_id = glyph_id as u16;
        self.core_text_font_for_attributes(attributes)
            .get_bounding_rects_for_glyphs(kCTFontDefaultOrientation, &[glyph_id])
            .to_euclid_rect()
    }

    #[inline]
    fn get_font_data(&self) -> &[u8] {
        match self.font_data {
            Some(ref font_data) => &**font_data,
            None => &[],
        }
    }

    #[inline]
    fn get_font_index(&self) -> i32 {
        0
    }

    #[inline]
    fn get_font_traits(&self) -> FontTraits {
        // TODO(pcwalton): Figure out the lang list ID.
        // TODO(pcwalton): Figure out the font variant.
        // TODO(pcwalton): Figure out which weights to use.
        let symbolic_traits = self.core_text_font.symbolic_traits();
        FontTraits::new(0, 0, 400, symbolic_traits.is_italic())
    }
}

impl Font {
    fn core_text_font_for_attributes(&self, attributes: &FontAttributes) -> CTFont {
        let size = attributes.size as f64;
        if self.core_text_font.pt_size() == size {
            self.core_text_font.clone()
        } else {
            self.core_text_font.clone_with_font_size(size)
        }
    }
}

impl Debug for Font {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(&self.core_text_font.postscript_name())
    }
}

trait ToEuclidRect {
    fn to_euclid_rect(&self) -> Rect<f32>;
}

impl ToEuclidRect for CGRect {
    #[inline]
    fn to_euclid_rect(&self) -> Rect<f32> {
        Rect::new(Point2D::new(self.origin.x as f32, self.origin.y as f32),
                  Size2D::new(self.size.width as f32, self.size.height as f32))
    }
}
