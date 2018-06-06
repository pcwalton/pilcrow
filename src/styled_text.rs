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
use font_traits::DEFAULT_FONT_WEIGHT;
use platform::Font;

const DEFAULT_FONT_SIZE: f32 = 16.0;

pub trait StyledText : Sized {
    fn get(&self, index: usize) -> StyledTextNode;
    fn node_count(&self) -> usize;
    fn initial_style(&self) -> InitialStyle;

    fn byte_length(&self) -> usize {
        self.iter().map(|node| {
            match node {
                StyledTextNode::String(ref string) => string.len(),
                StyledTextNode::Start(_) | StyledTextNode::End => 0,
            }
        }).sum()
    }

    #[inline]
    fn iter(&self) -> StyledTextNodeIter<Self> {
        StyledTextNodeIter {
            styled_text: self,
            index: 0,
        }
    }
}

pub struct StyledTextNodeIter<'a, T> where T: StyledText + 'a {
    styled_text: &'a T,
    index: usize,
}

impl<'a, T> Iterator for StyledTextNodeIter<'a, T> where T: StyledText {
    type Item = StyledTextNode<'a>;

    fn next(&mut self) -> Option<StyledTextNode<'a>> {
        if self.index == self.styled_text.node_count() {
            return None
        }

        let node = self.styled_text.get(self.index);
        self.index += 1;
        Some(node)
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
