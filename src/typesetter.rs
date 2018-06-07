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
use minikin_sys::{minikin_line_breaker_get_breaks};
use minikin_sys::{minikin_line_breaker_get_char_widths};
use minikin_sys::{minikin_line_breaker_get_face_count, minikin_line_breaker_get_face_fake_bold};
use minikin_sys::{minikin_line_breaker_get_face_fake_italic};
use minikin_sys::{minikin_line_breaker_get_face_indices};
use minikin_sys::{minikin_line_breaker_get_face_typeface};
use minikin_sys::{minikin_line_breaker_get_glyph_ids};
use minikin_sys::{minikin_line_breaker_get_size};
use minikin_sys::{minikin_line_breaker_resize, minikin_line_breaker_set_line_widths};
use minikin_sys::{minikin_line_breaker_set_locales, minikin_line_breaker_set_text};
use minikin_sys::{minikin_line_breaker_t, minikin_paint_clone};
use minikin_sys::{minikin_paint_create};
use minikin_sys::{minikin_paint_destroy, minikin_paint_get_size, minikin_paint_get_typeface};
use minikin_sys::{minikin_paint_set_letter_spacing, minikin_paint_set_size};
use minikin_sys::{minikin_paint_set_word_spacing, minikin_paint_t, minikin_typeface_t};
use std::cmp;
use std::ffi::CStr;
use std::i32;
use std::ops::Range;
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::u32;

use font::FontLike;
use font_set::FontSet;
use line::Line;
use platform::Font;
use run::Run;
use styled_text::{InitialStyle, Style, StyledText, StyledTextNode};

// TODO(pcwalton): Make this configurable.
static DEFAULT_LOCALE: &'static [u8] = b"en-US\0";

pub struct Typesetter<Text> {
    text: Text,
    width: f32,
    index: usize,
    minikin_line_breaker: Option<MinikinLineBreaker>,
}

impl<Text> Typesetter<Text> where Text: StyledText {
    // TODO(pcwalton): Before exposing publicly, get rid of the `width` parameter in favor of
    // `width` arguments on `suggest_line_break()`.
    #[inline]
    pub fn new(text: Text, width: f32) -> Typesetter<Text> {
        Typesetter {
            text,
            width,
            index: 0,
            minikin_line_breaker: None,
        }
    }

    #[inline]
    pub fn text(&self) -> &Text {
        &self.text
    }

    #[inline]
    pub fn text_mut(&mut self) -> &mut Text {
        &mut self.text
    }

    pub fn create_line(&mut self, utf16_string_range: Range<usize>) -> Line {
        self.ensure_typeset();
        let line_breaker = self.minikin_line_breaker.as_mut().unwrap();

        let mut runs = vec![];
        let (mut utf8_index, mut utf16_index) = (0, 0);
        let (mut utf8_start_index, mut utf8_end_index) = (None, 0);

        let mut style_runs = vec![MinikinStyleRun::from_initial_style(&self.text.initial_style())];

        for node in self.text.iter() {
            match node {
                StyledTextNode::String(ref string) => {
                    let utf8_node_range = utf8_index..(utf8_index + string.len());
                    let utf16_node_range = utf16_index..(utf16_index + string.encode_utf16()
                                                                             .count());

                    let utf16_style_run_range = utf16_string_range.intersect(&utf16_node_range);
                    if utf16_style_run_range.start < utf16_style_run_range.end {
                        // Move to the first byte in the string in range.
                        let utf16_style_run_start_offset = utf16_style_run_range.start -
                            utf16_node_range.start;
                        let utf16_style_run_end_offset = utf16_style_run_range.end -
                            utf16_node_range.start;

                        /*let style_run_prefix = &string[0..style_run_utf8_offset];
                        let style_run_utf16_offset = style_run_prefix.encode_utf16().count();
                        utf8_index += style_run_utf8_offset;*/
                        if utf8_start_index.is_none() {
                            utf8_start_index = Some(utf8_index +
                                string.utf16_len_to_utf8_len(utf16_style_run_start_offset));
                        }
                        utf8_end_index = utf8_index +
                            string.utf16_len_to_utf8_len(utf16_style_run_end_offset);

                        let utf16_end_index = utf16_index + utf16_style_run_end_offset;
                        utf16_index += utf16_style_run_start_offset;
                        debug_assert!(utf16_node_range.start < utf16_node_range.end);

                        let style = style_runs.last().unwrap();
                        if let MinikinStyleRun::InlineStyle { paint, .. } = *style {
                            // FIXME(pcwalton): Could be too much FFI traffic calling
                            // `face_indices()` over and over.
                            let mut current_face_index = i32::MAX;
                            let mut utf16_start_index = utf16_index;
                            let mut utf16_current_index = utf16_start_index;
                            loop {
                                let next_face_index =
                                    line_breaker.face_indices()[utf16_current_index];
                                if current_face_index != next_face_index {
                                    // Flush.
                                    flush_minikin_font_run(&mut runs,
                                                           &line_breaker,
                                                           current_face_index,
                                                           paint,
                                                           utf16_start_index..utf16_current_index);

                                    current_face_index = next_face_index;
                                    utf16_start_index = utf16_current_index;
                                }

                                utf16_current_index += 1;
                                if utf16_current_index >= utf16_end_index {
                                    break
                                }
                            }

                            // Flush the final run.
                            flush_minikin_font_run(&mut runs,
                                                   &line_breaker,
                                                   current_face_index,
                                                   paint,
                                                   utf16_start_index..utf16_current_index);
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

        let utf8_string_range = utf8_start_index.unwrap_or(0)..utf8_end_index;
        return Line::from_runs_and_range(runs, utf8_string_range);

        fn flush_minikin_font_run(runs: &mut Vec<Run>,
                                  line_breaker: &MinikinLineBreaker,
                                  face_index: i32,
                                  paint: *mut minikin_paint_t,
                                  utf16_font_run_range: Range<usize>) {
            if face_index < 0 || utf16_font_run_range.start >= utf16_font_run_range.end {
                return
            }

            unsafe {
                let advances = line_breaker.char_widths()[utf16_font_run_range.clone()].to_vec();
                let glyph_ids = line_breaker.glyph_ids()[utf16_font_run_range.clone()].to_vec();
                let typeface = line_breaker.face(face_index as usize).typeface();
                let size = minikin_paint_get_size(paint);
                let font = (*Font::from_minikin_typeface(typeface).unwrap()).clone();
                runs.push(Run::new(glyph_ids, advances, font, size));
            }
        }
    }

    /// A convenience method that calls `create_line()` with the entire string range.
    #[inline]
    pub fn create_single_line(&mut self) -> Line {
        let range = 0..self.text.byte_length();
        self.create_line(range)
    }

    // TODO(pcwalton): Remove this method before stabilization.
    pub fn line_breaks(&mut self) -> &[i32] {
        self.ensure_typeset();
        self.minikin_line_breaker.as_mut().unwrap().breaks()
    }

    fn ensure_typeset(&mut self) {
        if self.minikin_line_breaker.is_none() {
            self.minikin_line_breaker = Some(MinikinLineBreaker::create_and_break(&self.text,
                                                                                  self.width))
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
    fn create_and_break<Text>(text: &Text, width: f32) -> MinikinLineBreaker
                              where Text: StyledText {
        unsafe {
            // Create a Minikin line breaker.
            let line_breaker = minikin_line_breaker_create();
            assert!(!line_breaker.is_null());

            // Set the locale. (A segfault will occur if we don't do this.)
            let locale = CStr::from_bytes_with_nul(DEFAULT_LOCALE).unwrap();
            minikin_line_breaker_set_locales(line_breaker, locale.as_ptr());

            // Build and index the UTF-16 string.
            let (mut utf16_string, mut string_utf16_lengths) = (vec![], vec![]);
            for node in text.iter() {
                if let StyledTextNode::String(string) = node {
                    let utf16_start_index = utf16_string.len();
                    utf16_string.extend(string.encode_utf16());
                    let utf16_end_index = utf16_string.len();
                    string_utf16_lengths.push(utf16_end_index - utf16_start_index);
                }
            }

            // Feed the UTF-16 string to Minikin.
            let utf16_len = utf16_string.len() as libc::size_t;
            minikin_line_breaker_resize(line_breaker, utf16_len);
            let line_breaker_buffer = minikin_line_breaker_get_buffer(line_breaker);
            ptr::copy_nonoverlapping(utf16_string.as_ptr(), line_breaker_buffer, utf16_len);

            // Set line widths.
            minikin_line_breaker_set_line_widths(line_breaker, width, 1, width);

            // Commit the text. This must be done before setting style runs.
            minikin_line_breaker_set_text(line_breaker);

            // Set up styles.
            let mut style_runs = vec![MinikinStyleRun::from_initial_style(&text.initial_style())];
            let (mut string_index, mut last_utf16_index, mut current_utf16_index) = (0, 0, 0);

            for node in text.iter() {
                match node {
                    StyledTextNode::Start(_) | StyledTextNode::End => {
                        let utf16_range = last_utf16_index..current_utf16_index;
                        style_runs.last().unwrap().add_to_line_breaker(line_breaker, utf16_range);
                        last_utf16_index = current_utf16_index;
                    }
                    _ => {}
                }

                match node {
                    StyledTextNode::Start(ref style) => {
                        let mut new_style_run = (*style_runs.last().unwrap()).clone();
                        new_style_run.add_style(style);
                        style_runs.push(new_style_run);
                    }
                    StyledTextNode::End => {
                        style_runs.pop();
                    }
                    StyledTextNode::String(_) => {
                        current_utf16_index += string_utf16_lengths[string_index];
                        string_index += 1;
                    }
                }
            }

            let utf16_range = last_utf16_index..current_utf16_index;
            style_runs.last().unwrap().add_to_line_breaker(line_breaker, utf16_range);
            debug_assert_eq!(style_runs.len(), 1);

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

    fn char_widths(&self) -> &[f32] {
        unsafe {
            let length = self.char_len();
            slice::from_raw_parts(minikin_line_breaker_get_char_widths(self.line_breaker), length)
        }
    }

    fn glyph_ids(&self) -> &[u32] {
        unsafe {
            let length = self.char_len();
            slice::from_raw_parts(minikin_line_breaker_get_glyph_ids(self.line_breaker), length)
        }
    }

    fn face_indices(&self) -> &[i32] {
        unsafe {
            let length = self.char_len();
            slice::from_raw_parts(minikin_line_breaker_get_face_indices(self.line_breaker), length)
        }
    }

    fn face_count(&self) -> usize {
        unsafe {
            minikin_line_breaker_get_face_count(self.line_breaker)
        }
    }

    fn face(&self, index: usize) -> MinikinFace {
        unsafe {
            assert!(index < self.face_count());
            MinikinFace {
                line_breaker: self,
                index,
            }
        }
    }

    fn breaks(&self) -> &[i32] {
        unsafe {
            slice::from_raw_parts(minikin_line_breaker_get_breaks(self.line_breaker),
                                  self.break_count)
        }
    }
}

struct MinikinFace<'a> {
    line_breaker: &'a MinikinLineBreaker,
    index: usize,
}

impl<'a> MinikinFace<'a> {
    fn typeface(&self) -> *mut minikin_typeface_t {
        unsafe {
            minikin_line_breaker_get_face_typeface(self.line_breaker.line_breaker, self.index)
        }
    }

    fn fake_bold(&self) -> bool {
        unsafe {
            minikin_line_breaker_get_face_fake_bold(self.line_breaker.line_breaker, self.index)
        }
    }

    fn fake_italic(&self) -> bool {
        unsafe {
            minikin_line_breaker_get_face_fake_italic(self.line_breaker.line_breaker, self.index)
        }
    }
}

enum MinikinStyleRun {
    InlineStyle {
        paint: *mut minikin_paint_t,
        font_set: Arc<FontSet>,
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
            if let MinikinStyleRun::InlineStyle { paint, .. } = *self {
                assert!(!paint.is_null());
                minikin_paint_destroy(paint);
            }
        }
    }
}

impl Clone for MinikinStyleRun {
    fn clone(&self) -> MinikinStyleRun {
        unsafe {
            match *self {
                MinikinStyleRun::InlineStyle {
                    paint,
                    ref font_set,
                    font_style,
                    is_rtl,
                } => {
                    MinikinStyleRun::InlineStyle {
                        paint: minikin_paint_clone(paint),
                        font_set: (*font_set).clone(),
                        font_style,
                        is_rtl,
                    }
                }
                MinikinStyleRun::ReplacedContent { width } => {
                    MinikinStyleRun::ReplacedContent {
                        width,
                    }
                }
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

            MinikinStyleRun::InlineStyle {
                paint,
                font_set: initial_style.font_set.clone(),
                font_style,
                is_rtl: false,
            }
        }
    }

    fn add_style(&mut self, style: &Style) {
        if let Style::ReplacedContent(ref new_metrics) = *style {
            *self = MinikinStyleRun::ReplacedContent {
                width: new_metrics.width,
            };
            return
        }

        if let MinikinStyleRun::InlineStyle {
            ref mut font_set,
            paint,
            ref mut font_style,
            is_rtl: _
        } = *self {
            unsafe {
                match *style {
                    Style::FontSet(ref new_font_set) => *font_set = (*new_font_set).clone(),
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
                                  utf16_range: Range<usize>) {
        if utf16_range.start == utf16_range.end {
            return
        }

        match *self {
            MinikinStyleRun::InlineStyle { ref font_set, paint, font_style, is_rtl } => {
                assert!(utf16_range.start <= i32::MAX as usize);
                assert!(utf16_range.end <= i32::MAX as usize);
                minikin_line_breaker_add_style_run(line_breaker,
                                                   paint,
                                                   font_set.as_minikin_font_collection(),
                                                   font_style,
                                                   utf16_range.start as i32,
                                                   utf16_range.end as i32,
                                                   is_rtl);
            }
            MinikinStyleRun::ReplacedContent { width } => {
                minikin_line_breaker_add_replacement(line_breaker,
                                                     utf16_range.start,
                                                     utf16_range.end,
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

trait StringExt {
    fn utf16_len_to_utf8_len(&self, utf16_len: usize) -> usize;
}

impl StringExt for str {
    fn utf16_len_to_utf8_len(&self, utf16_len: usize) -> usize {
        let prefix: Vec<u16> = self.encode_utf16().take(utf16_len).collect();
        String::from_utf16_lossy(&prefix).len()
    }
}
