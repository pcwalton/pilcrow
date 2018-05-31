// pilcrow/src/styled_text.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use font_collection::FontCollection;
use platform::Font;

const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_FONT_WEIGHT: i32 = 400;

pub trait StyledText {
    fn move_prev(&mut self) -> bool;
    fn move_next(&mut self) -> bool;
    fn get(&self) -> StyledTextNode;
    fn initial_style(&self) -> InitialStyle;

    #[inline]
    fn rewind(&mut self) {
        while self.move_prev() {}
    }

    #[inline]
    fn byte_length(&mut self) -> usize {
        self.rewind();
        self.remaining_byte_length()
    }

    fn remaining_byte_length(&mut self) -> usize {
        let mut length = 0;
        loop {
            if let StyledTextNode::String(string) = self.get() {
                length += string.len()
            }
            if !self.move_next() {
                return length
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct InitialStyle {
    pub font_family: Arc<FontCollection>,
    pub font_size: f32,
    pub font_weight: i32,
    pub font_italic: bool,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

impl InitialStyle {
    pub fn from_font_family(font_family: Arc<FontCollection>) -> InitialStyle {
        InitialStyle {
            font_family: font_family,
            font_size: DEFAULT_FONT_SIZE,
            font_weight: DEFAULT_FONT_WEIGHT,
            font_italic: false,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Style {
    FontFamily(Arc<FontCollection>),
    FontSize(f32),
    FontWeight(i32),
    FontItalic(bool),
    LetterSpacing(f32),
    WordSpacing(f32),
    ReplacedContent(ReplacedContentMetrics),
}

#[derive(Clone, Debug)]
pub struct ReplacedContentMetrics {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

#[derive(Clone, Debug)]
pub enum StyledTextNode<'a> {
    String(&'a str),
    Start(Style),
    End,
}

#[derive(Clone, Debug)]
pub enum StyledTextNodeBuf {
    String(String),
    Start(Style),
    End,
}

impl StyledTextNodeBuf {
    #[inline]
    pub fn borrow(&self) -> StyledTextNode {
        match *self {
            StyledTextNodeBuf::String(ref string) => StyledTextNode::String(string),
            StyledTextNodeBuf::Start(ref style) => StyledTextNode::Start((*style).clone()),
            StyledTextNodeBuf::End => StyledTextNode::End,
        }
    }
}
