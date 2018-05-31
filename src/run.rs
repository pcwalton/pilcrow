// pilcrow/src/run.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use platform::Font;

#[derive(Clone, Debug)]
pub struct Run {
    glyphs: Vec<u32>,
    advances: Vec<f32>,
    font: Font,
    size: f32,
}

impl Run {
    #[inline]
    pub(crate) fn new(glyphs: Vec<u32>, advances: Vec<f32>, font: Font, size: f32) -> Run {
        Run {
            glyphs,
            advances,
            font,
            size,
        }
    }

    #[inline]
    pub fn glyphs(&self) -> &[u32] {
        &self.glyphs
    }

    #[inline]
    pub fn advances(&self) -> &[f32] {
        &self.advances
    }

    #[inline]
    pub fn font(&self) -> &Font {
        &self.font
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.size
    }
}
