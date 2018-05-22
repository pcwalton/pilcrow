// pilcrow/src/styled_text.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::ops::Range;
use std::sync::Arc;

use font::Font;

// FIXME(pcwalton): This is probably a bad API. Consider an iterator.

bitflags! {
    pub struct StyleChanges: u8 {
        const FONT = 0x01;
        const SIZE = 0x02;
        const LETTER_SPACING = 0x04;
        const WORD_SPACING = 0x08;
    }
}

#[derive(Clone, Debug)]
pub struct StyleRange<T> {
    pub computed_value: T,
    pub longest_effective_range: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct ReplacedContentMetrics {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

pub trait StyledText {
    fn string(&self) -> Cow<str>;

    fn get_style_changes(&self, index: usize) -> StyleChanges;

    fn get_font(&self, index: usize) -> StyleRange<Arc<Font>>;
    fn get_size(&self, index: usize) -> StyleRange<f32>;
    fn get_letter_spacing(&self, index: usize) -> StyleRange<f32>;
    fn get_word_spacing(&self, index: usize) -> StyleRange<f32>;
    fn get_replaced_content(&self, index: usize) -> StyleRange<Option<ReplacedContentMetrics>>;

    fn len(&self) -> usize {
        self.string().len()
    }
}
