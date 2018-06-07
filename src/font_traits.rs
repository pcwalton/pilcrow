// pilcrow/src/font_traits.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use minikin_sys::{minikin_font_style_create, minikin_font_style_t};

pub const DEFAULT_FONT_WEIGHT: i32 = 400;

#[derive(Clone)]
pub struct FontTraits {
    minikin_font_style: minikin_font_style_t,
}

impl Default for FontTraits {
    #[inline]
    fn default() -> FontTraits {
        FontTraits::new(0, 0, DEFAULT_FONT_WEIGHT, false)
    }
}

impl FontTraits {
    #[inline]
    pub fn new(lang_list_id: u32, variant: i32, weight: i32, italic: bool) -> FontTraits {
        unsafe {
            let minikin_style = minikin_font_style_create(lang_list_id, variant, weight, italic);
            FontTraits {
                minikin_font_style: minikin_style,
            }
        }
    }

    #[inline]
    pub(crate) fn minikin_font_style(&self) -> minikin_font_style_t {
        self.minikin_font_style
    }
}
