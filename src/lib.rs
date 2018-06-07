// pilcrow/src/lib.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate cocoa;
extern crate core_foundation;
extern crate core_graphics;
extern crate core_text;
extern crate euclid;
extern crate indexmap;
extern crate libc;
extern crate minikin_sys;
extern crate pulldown_cmark;
extern crate rayon;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate objc;

#[cfg(feature = "cairo")]
extern crate cairo;

//pub use format::{Color, Font, FontFaceId, FontId, Format, Image};

use core_foundation::attributedstring::{CFAttributedString, CFMutableAttributedString};
use core_foundation::base::{CFIndex, CFRange, CFType, CFTypeRef, TCFType, kCFNotFound};
use core_foundation::dictionary::{CFDictionary, CFMutableDictionary};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::stringtokenizer::{CFStringTokenizer, kCFStringTokenizerUnitWord};
use core_graphics::base::CGFloat;
use core_graphics::font::CGGlyph;
use core_graphics::geometry::{CGPoint, CGRect, CGSize, CG_ZERO_POINT};
use core_graphics::path::CGPath;
use core_text::frame::CTFrame;
use core_text::framesetter::CTFramesetter;
use core_text::line::CTLine;
use core_text::run::CTRun;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Vector2D};
use minikin_sys::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::cmp::{self, Ordering};
use std::ops::Range;
use std::sync::{Mutex, MutexGuard, RwLock};

#[cfg_attr(target_os = "macos", path = "platform/macos.rs")]
pub mod platform;

//pub mod ffi;
pub mod font;
pub mod font_family;
pub mod font_set;
pub mod font_traits;
pub mod framesetter;
pub mod line;
//pub mod markdown;
pub mod run;
pub mod simple_styled_text;
pub mod styled_text;

//mod format;

#[cfg(test)]
pub mod tests;

// Crate-only because the line break suggestion API isn't implemented yet…
pub(crate) mod typesetter;

/*

pub type Glyph = CGGlyph;

pub trait LayoutCallbacks: Send + Sync {
    fn get_image_size(&self, image_id: u32) -> Option<Size2D<u32>>;
}

lazy_static! {
    static ref LAYOUT_CALLBACKS: RwLock<Option<Box<LayoutCallbacks>>> = {
        RwLock::new(None)
    };
}

pub struct Document {
    paragraphs: Vec<Paragraph>,
    style: DocumentStyle,
}

impl Document {
    #[inline]
    pub fn new() -> Document {
        Document {
            paragraphs: vec![],
            style: DocumentStyle::default(),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.paragraphs.clear()
    }

    #[inline]
    pub fn append_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph)
    }

    #[inline]
    pub fn append_document(&mut self, other_document: Document) {
        self.paragraphs.extend(other_document.paragraphs.into_iter())
    }

    #[inline]
    pub fn paragraphs(&self) -> &[Paragraph] {
        &self.paragraphs
    }

    #[inline]
    pub fn paragraphs_mut(&mut self) -> &mut [Paragraph] {
        &mut self.paragraphs
    }

    #[inline]
    pub fn style_mut(&mut self) -> &mut DocumentStyle {
        &mut self.style
    }

    #[inline]
    pub fn entire_range(&self) -> Range<TextLocation> {
        let start = TextLocation::new(0, 0);
        let end = match self.paragraphs.last() {
            None => start,
            Some(last_paragraph) => {
                TextLocation::new(self.paragraphs.len() - 1, last_paragraph.char_len())
            }
        };
        start..end
    }

    pub fn copy_string_in_range(&self, range: Range<TextLocation>) -> String {
        let mut buffer = String::new();
        let first_paragraph_index = range.start.paragraph_index;
        let last_paragraph_index = cmp::min(range.end.paragraph_index + 1, self.paragraphs.len());
        let paragraph_count = last_paragraph_index - first_paragraph_index;
        let paragraph_range = first_paragraph_index..last_paragraph_index;
        for (paragraph_index, paragraph) in self.paragraphs[paragraph_range].iter().enumerate() {
            let char_start = if paragraph_index == 0 {
                range.start.character_index
            } else {
                0
            };
            let char_end = if paragraph_index == paragraph_count - 1 {
                range.end.character_index
            } else {
                paragraph.char_len()
            };
            if paragraph_index != 0 {
                buffer.push('\n')
            }
            paragraph.copy_string_in_range(&mut buffer, char_start..char_end)
        }
        buffer
    }

    #[inline]
    pub fn copy_string(&self) -> String {
        self.copy_string_in_range(self.entire_range())
    }
}

pub struct Paragraph {
    attributed_string: Mutex<CFMutableAttributedString>,
    style: ParagraphStyle,
}

unsafe impl Sync for Paragraph {}

impl Paragraph {
    #[inline]
    pub fn new(style: ParagraphStyle) -> Paragraph {
        let attributed_string = CFAttributedString::new(CFString::from(""), CFDictionary::new());
        let mutable_attributed_string =
            CFMutableAttributedString::from_attributed_string(attributed_string);
        Paragraph {
            attributed_string: Mutex::new(mutable_attributed_string),
            style,
        }
    }

    pub fn from_string(string: &str, style: ParagraphStyle) -> Paragraph {
        let attributed_string = CFAttributedString::new(CFString::from(string),
                                                        CFDictionary::new());
        let mutable_attributed_string =
            CFMutableAttributedString::from_attributed_string(attributed_string);
        Paragraph {
            attributed_string: Mutex::new(mutable_attributed_string),
            style,
        }
    }

    #[inline]
    pub fn copy_string_in_range(&self, buffer: &mut String, range: Range<usize>) {
        buffer.extend(self.attributed_string
                          .lock()
                          .unwrap()
                          .string()
                          .to_string()
                          .chars()
                          .skip(range.start)
                          .take(range.end - range.start))
    }

    #[inline]
    pub fn char_len(&self) -> usize {
        self.attributed_string.lock().unwrap().string().char_len() as usize
    }

    #[inline]
    pub fn edit_at(&mut self, position: usize) -> ParagraphCursor {
        let attributes = self.attributed_string
                             .lock()
                             .unwrap()
                             .attributes_at(position as CFIndex)
                             .0;
        let format_stack = format::attributes_to_formatting(&attributes);
        ParagraphCursor {
            attributed_string: self.attributed_string.lock().unwrap(),
            position: position,
            buffer: CFMutableAttributedString::new(),
            format_stack: format_stack,
        }
    }

    pub fn word_range_at_char_index(&self, index: usize) -> Range<usize> {
        let attributed_string = self.attributed_string.lock().unwrap();
        let string = attributed_string.string();
        let range = CFRange::init(0, string.char_len());
        let tokenizer = CFStringTokenizer::new(string, range, kCFStringTokenizerUnitWord);
        tokenizer.go_to_token_at_index(index as CFIndex);
        let range = tokenizer.get_current_token_range();
        (range.location as usize)..((range.location + range.length) as usize)
    }
}

pub struct ParagraphCursor<'a> {
    attributed_string: MutexGuard<'a, CFMutableAttributedString>,
    position: usize,
    buffer: CFMutableAttributedString,
    format_stack: Vec<Format>,
}

impl<'a> ParagraphCursor<'a> {
    pub fn commit(self) {
        let range = CFRange::init(self.position as CFIndex, 0);
        let buffer = self.buffer.as_attributed_string();
        self.attributed_string.replace_attributed_string(range, buffer)
    }

    pub fn push_string(&mut self, string: &str) {
        let mut attributes = CFMutableDictionary::new();
        for format in &self.format_stack {
            format.add_to_native_attributes(&mut attributes);
        }
        let attributes = attributes.as_dictionary();
        let attributed_string = CFAttributedString::new(CFString::from(string), attributes);
        let range = CFRange::init(self.buffer.string().char_len() as CFIndex, 0);
        self.buffer.replace_attributed_string(range, attributed_string)
    }

    pub fn push_format(&mut self, format: Format) {
        self.format_stack.push(format)
    }

    pub fn pop_format(&mut self) {
        self.format_stack.pop().expect("ParagraphCursor::pop_format(): Format stack empty!");
    }

    pub fn format_stack(&self) -> &[Format] {
        &self.format_stack
    }
}

pub struct Framesetter {
    framesetters: Vec<Mutex<ParagraphFramesetter>>,
    document_style: DocumentStyle,
}

impl Framesetter {
    pub fn new(document: &Document) -> Framesetter {
        Framesetter {
            framesetters: document.paragraphs().par_iter().map(|paragraph| {
                let attributed_string = paragraph.attributed_string.lock().unwrap();
                let attributed_string = attributed_string.as_attributed_string();
                let framesetter = CTFramesetter::from_attributed_string(attributed_string.clone());
                Mutex::new(ParagraphFramesetter {
                    framesetter: framesetter,
                    attributed_string: attributed_string,
                    style: paragraph.style.clone(),
                })
            }).collect(),
            document_style: document.style.clone(),
        }
    }

    pub fn layout_in_rect(&self, rect: &Rect<f32>, callbacks: Option<Box<LayoutCallbacks>>)
                          -> Section {
        eprintln!("document margins: {:?}", self.document_style.margin);

        *LAYOUT_CALLBACKS.write().unwrap() = callbacks;

        let rect = rect.inner_rect(self.document_style.margin);

        let mut frames: Vec<_> = self.framesetters.par_iter().map(|paragraph_framesetter| {
            let paragraph_framesetter = paragraph_framesetter.lock().unwrap();
            let range = CFRange::init(0, paragraph_framesetter.attributed_string
                                                              .string()
                                                              .char_len());

            let mut rect = rect;
            rect.size.width -= paragraph_framesetter.style.margin.horizontal();

            let origin = CGPoint::new(rect.origin.x as CGFloat, rect.origin.y as CGFloat);
            let size = CGSize::new(rect.size.width as CGFloat, rect.size.height as CGFloat);
            let path = CGPath::from_rect(CGRect::new(&origin, &size), None);
            Frame {
                frame: paragraph_framesetter.framesetter.create_frame(range, path, None),
                style: paragraph_framesetter.style.clone(),
                virtual_size: rect.size,
                origin: Point2D::zero(),
            }
        }).collect();

        // TODO(pcwalton): Vertical writing direction.
        let mut origin = rect.origin;
        for mut frame in &mut frames {
            origin.y += frame.style.margin.top;
            frame.origin = origin + Vector2D::new(frame.style.margin.left, 0.0);
            origin.y += frame.height();
            origin.y += frame.style.margin.bottom;
        }

        Section {
            frames,
        }
    }
}

struct ParagraphFramesetter {
    framesetter: CTFramesetter,
    attributed_string: CFAttributedString,
    style: ParagraphStyle,
}

pub struct Section {
    frames: Vec<Frame>,
}

impl Section {
    #[inline]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub fn frame_index_at_point(&self, point: &Point2D<f32>) -> Option<usize> {
        self.frames.binary_search_by(|frame| {
            compare_bounds_and_point_vertically(&frame.bounds(), &point)
        }).ok()
    }
}

pub struct Frame {
    frame: CTFrame,
    style: ParagraphStyle,
    virtual_size: Size2D<f32>,
    origin: Point2D<f32>,
}

impl Frame {
    pub fn char_len(&self) -> usize {
        self.frame.get_string_range().length as usize
    }

    pub fn lines(&self) -> Vec<Line> {
        let lines = self.frame.lines();
        let mut line_origins = vec![CG_ZERO_POINT; lines.len() as usize];
        self.frame.get_line_origins(0, &mut line_origins);
        let virtual_height = self.virtual_size.height;
        let frame_origin = self.origin;
        lines.into_iter().zip(line_origins.into_iter()).map(|(line, line_origin)| {
            Line {
                line: (*line).clone(),
                origin: Point2D::new(frame_origin.x + line_origin.x as f32,
                                     frame_origin.y + virtual_height - line_origin.y as f32),
            }
        }).collect()
    }

    #[inline]
    pub fn bounds(&self) -> Rect<f32> {
        Rect::new(self.origin, Size2D::new(self.virtual_size.width, self.height()))
    }

    pub fn height(&self) -> f32 {
        let lines = self.frame.lines();
        let line_count = lines.len();
        if line_count == 0 {
            return 0.0
        }

        let last_line = lines.get(line_count - 1).unwrap();
        let mut line_origins = [CG_ZERO_POINT];
        self.frame.get_line_origins(line_count - 1, &mut line_origins);
        last_line.typographic_bounds().descent as f32 - line_origins[0].y as f32 +
            self.virtual_size.height
    }

    pub fn line_index_at_point(&self, point: &Point2D<f32>) -> Option<usize> {
        self.lines().binary_search_by(|line| {
            compare_bounds_and_point_vertically(&line.typographic_bounding_rect(), &point)
        }).ok()
    }

    #[inline]
    pub fn style(&self) -> &ParagraphStyle {
        &self.style
    }
}

#[derive(Clone, PartialEq)]
pub struct ParagraphStyle {
    pub content: ParagraphContent,
    pub margin: SideOffsets2D<f32>,
}

impl ParagraphStyle {
    #[inline]
    pub fn new(content: ParagraphContent) -> ParagraphStyle {
        ParagraphStyle {
            content,
            margin: SideOffsets2D::zero(),
        }
    }
}

impl Default for ParagraphStyle {
    #[inline]
    fn default() -> ParagraphStyle {
        ParagraphStyle {
            content: ParagraphContent::Text,
            margin: SideOffsets2D::zero(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ParagraphContent {
    Text,
    Rule,
}

#[derive(Clone, PartialEq)]
pub struct DocumentStyle {
    pub margin: SideOffsets2D<f32>,
}

impl Default for DocumentStyle {
    #[inline]
    fn default() -> DocumentStyle {
        DocumentStyle {
            margin: SideOffsets2D::zero(),
        }
    }
}

pub struct Line {
    line: CTLine,
    pub origin: Point2D<f32>,
}

impl Line {
    pub fn runs(&self) -> Vec<Run> {
        self.line.glyph_runs().into_iter().map(|run| {
            Run {
                run: (*run).clone(),
            }
        }).collect()
    }

    #[inline]
    pub fn char_range(&self) -> Range<usize> {
        let range = self.line.string_range();
        (range.location as usize)..((range.location + range.length) as usize)
    }

    pub fn typographic_bounding_rect(&self) -> Rect<f32> {
        let typographic_bounds = self.typographic_bounds();
        Rect::new(Point2D::new(self.origin.x, self.origin.y - typographic_bounds.ascent),
                  Size2D::new(typographic_bounds.width,
                              typographic_bounds.ascent + typographic_bounds.descent))
    }

    #[inline]
    pub fn typographic_bounds(&self) -> TypographicBounds {
        let typographic_bounds = self.line.typographic_bounds();
        TypographicBounds {
            width: typographic_bounds.width as f32,
            ascent: typographic_bounds.ascent as f32,
            descent: typographic_bounds.descent as f32,
            leading: typographic_bounds.leading as f32,
        }
    }

    #[inline]
    pub fn char_index_for_position(&self, position: &Point2D<f32>) -> Option<usize> {
        let position = CGPoint::new(position.x as CGFloat, position.y as CGFloat);
        match self.line.get_string_index_for_position(position) {
            kCFNotFound => None,
            index => Some(index as usize),
        }
    }

    #[inline]
    pub fn inline_position_for_char_index(&self, index: usize) -> f32 {
        self.line.get_offset_for_string_index(index as CFIndex).0 as f32
    }
}

pub struct Run {
    run: CTRun,
}

impl Run {
    #[inline]
    pub fn glyph_count(&self) -> usize {
        self.run.glyph_count() as usize
    }

    pub fn glyphs(&self) -> Vec<Glyph> {
        let mut glyphs = vec![0; self.glyph_count()];
        self.run.get_glyphs(0, &mut glyphs);
        glyphs
    }

    pub fn positions(&self) -> Vec<Point2D<f32>> {
        let mut positions = vec![CG_ZERO_POINT; self.glyph_count()];
        self.run.get_positions(0, &mut positions);
        positions.into_iter().map(|p| Point2D::new(p.x as f32, p.y as f32)).collect()
    }

    #[inline]
    pub fn char_range(&self) -> Range<usize> {
        let range = self.run.get_string_range();
        (range.start as usize)..(range.end as usize)
    }

    pub fn formatting(&self) -> Vec<Format> {
        format::attributes_to_formatting(&self.run.attributes())
    }

    #[inline]
    pub fn typographic_bounds(&self) -> TypographicBounds {
        let typographic_bounds = self.run.typographic_bounds(0..(self.glyph_count() as CFIndex));
        TypographicBounds {
            width: typographic_bounds.width as f32,
            ascent: typographic_bounds.ascent as f32,
            descent: typographic_bounds.descent as f32,
            leading: typographic_bounds.leading as f32,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct TextLocation {
    pub paragraph_index: usize,
    pub character_index: usize,
}

impl TextLocation {
    #[inline]
    pub fn new(paragraph_index: usize, character_index: usize) -> TextLocation {
        TextLocation {
            paragraph_index,
            character_index,
        }
    }

    #[inline]
    pub fn beginning() -> TextLocation {
        TextLocation::new(0, 0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypographicBounds {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub leading: f32,
}

fn compare_bounds_and_point_vertically(bounds: &Rect<f32>, point: &Point2D<f32>) -> Ordering {
    match (bounds.origin.y <= point.y, point.y < bounds.max_y()) {
        (true, true) => Ordering::Equal,
        (false, _) => Ordering::Greater,
        (_, false) => Ordering::Less,
    }
}

*/
