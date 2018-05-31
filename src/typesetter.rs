// pilcrow/src/typesetter.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use libc;
use minikin_sys::{minikin_font_style_create, minikin_font_style_set_italic};
use minikin_sys::{minikin_font_style_set_weight, minikin_font_style_t};
use minikin_sys::{minikin_line_breaker_add_replacement, minikin_line_breaker_add_style_run};
use minikin_sys::{minikin_line_breaker_compute_breaks, minikin_line_breaker_create};
use minikin_sys::{minikin_line_breaker_destroy, minikin_line_breaker_get_buffer};
use minikin_sys::{minikin_line_breaker_get_glyph_ids, minikin_line_breaker_get_size};
use minikin_sys::{minikin_line_breaker_get_widths, minikin_line_breaker_resize};
use minikin_sys::{minikin_line_breaker_t, minikin_paint_clone, minikin_paint_create};
use minikin_sys::{minikin_paint_destroy, minikin_paint_get_font, minikin_paint_get_size};
use minikin_sys::{minikin_paint_set_letter_spacing, minikin_paint_set_size};
use minikin_sys::{minikin_paint_set_word_spacing, minikin_paint_t};
use std::cmp;
use std::i32;
use std::ops::Range;
use std::ptr;
use std::slice;
use std::sync::Arc;

use font::FontLike;
use font_collection::FontCollection;
use line::Line;
use platform::Font;
use run::Run;
use styled_text::{InitialStyle, Style, StyledText, StyledTextNode};

pub struct Typesetter<Text> {
    text: Text,
    index: usize,
    minikin_line_breaker: Option<MinikinLineBreaker>,
}

impl<Text> Typesetter<Text> where Text: StyledText {
    #[inline]
    pub fn new(text: Text) -> Typesetter<Text> {
        Typesetter {
            text,
            index: 0,
            minikin_line_breaker: None,
        }
    }

    #[inline]
    pub fn text(&mut self) -> &mut Text {
        &mut self.text
    }

    pub fn create_line(&mut self, string_range: Range<usize>) -> Line {
        self.ensure_typeset();
        let line_breaker = self.minikin_line_breaker.as_mut().unwrap();

        let mut runs = vec![];
        let (mut utf8_index, mut utf16_index) = (0, 0);

        let mut style_runs = vec![MinikinStyleRun::from_initial_style(&self.text.initial_style())];

        while self.text.move_next() {
            match self.text.get() {
                StyledTextNode::String(ref string) => {
                    let utf8_node_range = utf8_index..(utf8_index + string.len());
                    let utf16_node_range = utf16_index..(utf16_index + string.encode_utf16()
                                                                             .count());

                    let clamped_utf8_range = string_range.intersect(&utf8_node_range);
                    if clamped_utf8_range.start < clamped_utf8_range.end {
                        let utf8_offset = (clamped_utf8_range.start - utf8_node_range.start);
                        let prefix = &string[0..utf8_offset];
                        let utf16_offset = prefix.encode_utf16().count();

                        let clamped_utf16_range =
                            (utf16_node_range.start + utf16_offset)..utf16_node_range.end;
                        let widths = line_breaker.widths()[clamped_utf16_range.clone()].to_vec();
                        let glyph_ids = line_breaker.glyph_ids()[clamped_utf16_range].to_vec();

                        let style = style_runs.last().unwrap();
                        if let MinikinStyleRunAttrs::InlineStyle { paint, .. } = style.attrs {
                            unsafe {
                                let font = minikin_paint_get_font(paint);
                                let size = minikin_paint_get_size(paint);
                                let font = Font::from_minikin_font(font).unwrap();
                                runs.push(Run::new(glyph_ids, widths, (*font).clone(), size));
                            }
                        }
                    }

                    utf8_index = utf8_node_range.end;
                    utf16_index = utf16_node_range.end;
                }
                StyledTextNode::Start(ref style) => {
                    let mut new_style_run = (*style_runs.last().unwrap()).clone();
                    new_style_run.add_style(style);
                    style_runs.push(new_style_run);
                }
                StyledTextNode::End => {
                    style_runs.pop().unwrap();
                }
            }
        }

        self.text.rewind();

        // TODO(pcwalton)
        Line::from_runs_and_range(runs, string_range)
    }

    fn ensure_typeset(&mut self) {
        if self.minikin_line_breaker.is_none() {
            self.minikin_line_breaker = Some(MinikinLineBreaker::create_and_break(&mut self.text))
        }
    }
}

struct MinikinLineBreaker {
    line_breaker: *mut minikin_line_breaker_t,
    break_count: libc::size_t,
}

impl Drop for MinikinLineBreaker {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.line_breaker.is_null());
            minikin_line_breaker_destroy(self.line_breaker);
        }
    }
}

impl MinikinLineBreaker {
    fn create_and_break<Text>(text: &mut Text) -> MinikinLineBreaker where Text: StyledText {
        unsafe {
            // Create a Minikin line breaker.
            let line_breaker = minikin_line_breaker_create();
            assert!(!line_breaker.is_null());

            // Build and index the UTF-16 string.
            let (mut utf16_string, mut string_utf16_lengths) = (vec![], vec![]);
            while text.move_next() {
                if let StyledTextNode::String(string) = text.get() {
                    let utf16_start_index = utf16_string.len();
                    utf16_string.extend(string.encode_utf16());
                    let utf16_end_index = utf16_string.len();
                    string_utf16_lengths.push(utf16_end_index - utf16_start_index);
                }
            }
            text.rewind();

            // Feed the UTF-16 string to Minikin.
            let utf16_len = utf16_string.len() as libc::size_t;
            minikin_line_breaker_resize(line_breaker, utf16_len);
            let line_breaker_buffer = minikin_line_breaker_get_buffer(line_breaker);
            ptr::copy_nonoverlapping(utf16_string.as_ptr(), line_breaker_buffer, utf16_len * 2);

            // Set up styles.
            let mut style_runs = vec![MinikinStyleRun::from_initial_style(&text.initial_style())];
            let (mut string_index, mut utf16_index) = (0, 0);
            while text.move_next() {
                match text.get() {
                    StyledTextNode::Start(ref style) => {
                        let mut new_style_run = (*style_runs.last().unwrap()).clone();
                        new_style_run.utf16_start_index = utf16_index;
                        new_style_run.add_style(style);
                        style_runs.push(new_style_run);
                    }
                    StyledTextNode::End => {
                        style_runs.pop().unwrap().add_to_line_breaker(line_breaker, utf16_index)
                    }
                    StyledTextNode::String(_) => {
                        utf16_index += string_utf16_lengths[string_index];
                        string_index += 1;
                    }
                }
            }

            // Perform line breaking.
            let break_count = minikin_line_breaker_compute_breaks(line_breaker);

            // Finish up.
            MinikinLineBreaker {
                line_breaker,
                break_count,
            }
        }
    }

    fn char_len(&self) -> usize {
        unsafe {
            minikin_line_breaker_get_size(self.line_breaker)
        }
    }

    fn widths(&self) -> &[f32] {
        unsafe {
            let length = self.char_len();
            slice::from_raw_parts(minikin_line_breaker_get_widths(self.line_breaker), length)
        }
    }

    fn glyph_ids(&self) -> &[u32] {
        unsafe {
            let length = self.char_len();
            slice::from_raw_parts(minikin_line_breaker_get_glyph_ids(self.line_breaker), length)
        }
    }
}

struct MinikinStyleRun {
    utf16_start_index: usize,
    attrs: MinikinStyleRunAttrs,
}

enum MinikinStyleRunAttrs {
    InlineStyle {
        paint: *mut minikin_paint_t,
        font_collection: Arc<FontCollection>,
        font_style: minikin_font_style_t,
        is_rtl: bool,
    },
    ReplacedContent {
        width: f32,
    },
}

impl Drop for MinikinStyleRun {
    fn drop(&mut self) {
        unsafe {
            if let MinikinStyleRunAttrs::InlineStyle { paint, .. } = self.attrs {
                assert!(!paint.is_null());
                minikin_paint_destroy(paint);
            }
        }
    }
}

impl Clone for MinikinStyleRun {
    fn clone(&self) -> MinikinStyleRun {
        unsafe {
            MinikinStyleRun {
                utf16_start_index: 0,
                attrs: match self.attrs {
                    MinikinStyleRunAttrs::InlineStyle {
                        paint,
                        ref font_collection,
                        font_style,
                        is_rtl,
                    } => {
                        MinikinStyleRunAttrs::InlineStyle {
                            paint: minikin_paint_clone(paint),
                            font_collection: (*font_collection).clone(),
                            font_style,
                            is_rtl,
                        }
                    }
                    MinikinStyleRunAttrs::ReplacedContent {
                        width,
                    } => {
                        MinikinStyleRunAttrs::ReplacedContent {
                            width,
                        }
                    }
                },
            }
        }
    }
}

impl MinikinStyleRun {
    fn from_initial_style(initial_style: &InitialStyle) -> MinikinStyleRun {
        unsafe {
            let paint = minikin_paint_create();
            minikin_paint_set_size(paint, initial_style.font_size);
            minikin_paint_set_letter_spacing(paint, initial_style.letter_spacing);
            minikin_paint_set_word_spacing(paint, initial_style.word_spacing);

            // TODO(pcwalton): Set language properly.
            // TODO(pcwalton): Set font variant properly.
            // TODO(pcwalton): Support RTL.
            let font_style = minikin_font_style_create(0,
                                                       0,
                                                       initial_style.font_weight,
                                                       initial_style.font_italic);

            MinikinStyleRun {
                utf16_start_index: 0,
                attrs: MinikinStyleRunAttrs::InlineStyle {
                    paint,
                    font_collection: initial_style.font_family.clone(),
                    font_style,
                    is_rtl: false,
                },
            }
        }
    }

    fn add_style(&mut self, style: &Style) {
        if let Style::ReplacedContent(ref new_metrics) = *style {
            self.attrs = MinikinStyleRunAttrs::ReplacedContent {
                width: new_metrics.width,
            };
            return
        }

        if let MinikinStyleRunAttrs::InlineStyle {
            ref mut font_collection,
            paint,
            ref mut font_style,
            is_rtl: _
        } = self.attrs {
            unsafe {
                match *style {
                    Style::FontFamily(ref new_font_collection) => {
                        *font_collection = (*new_font_collection).clone()
                    }
                    Style::FontSize(new_size) => minikin_paint_set_size(paint, new_size),
                    Style::FontWeight(new_weight) => {
                        *font_style = minikin_font_style_set_weight(*font_style, new_weight)
                    }
                    Style::FontItalic(new_italic) => {
                        *font_style = minikin_font_style_set_italic(*font_style, new_italic)
                    }
                    Style::LetterSpacing(new_spacing) => {
                        minikin_paint_set_letter_spacing(paint, new_spacing)
                    }
                    Style::WordSpacing(new_spacing) => {
                        minikin_paint_set_word_spacing(paint, new_spacing)
                    }
                    Style::ReplacedContent(_) => unreachable!(),
                }
            }
        }
    }

    unsafe fn add_to_line_breaker(&self,
                                  line_breaker: *mut minikin_line_breaker_t,
                                  utf16_end_index: usize) {
        match self.attrs {
            MinikinStyleRunAttrs::InlineStyle {
                ref font_collection,
                paint,
                font_style,
                is_rtl,
            } => {
                assert!(self.utf16_start_index <= i32::MAX as usize);
                assert!(utf16_end_index <= i32::MAX as usize);
                minikin_line_breaker_add_style_run(line_breaker,
                                                   paint,
                                                   font_collection.as_minikin_font_collection(),
                                                   font_style,
                                                   self.utf16_start_index as i32,
                                                   utf16_end_index as i32,
                                                   is_rtl);
            }
            MinikinStyleRunAttrs::ReplacedContent {
                width
            } => {
                minikin_line_breaker_add_replacement(line_breaker,
                                                     self.utf16_start_index,
                                                     utf16_end_index,
                                                     width)
            }
        }
    }
}

trait RangeExt {
    fn intersect(&self, other: &Self) -> Self;
}

impl RangeExt for Range<usize> {
    fn intersect(&self, other: &Range<usize>) -> Range<usize> {
        cmp::max(self.start, other.start)..cmp::min(self.end, other.end)
    }
}
