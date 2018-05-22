// pilcrow/src/font.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use euclid::Rect;

pub trait Font {
    fn get_horizontal_advance(&self, glyph_id: u32, attributes: &FontAttributes) -> f32;
    fn get_bounds(&self, glyph_id: u32, attributes: &FontAttributes) -> Rect<f32>;
    fn get_font_data(&self) -> &[u8];
    fn get_font_index(&self) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub struct FontAttributes {
    pub size: f32,
}
