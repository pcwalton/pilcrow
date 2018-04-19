// pilcrow/src/lib.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate core_foundation;
extern crate core_graphics;
extern crate core_text;
extern crate euclid;
extern crate libc;
extern crate pulldown_cmark;
extern crate rayon;

pub use format::{Font, Format};

use core_foundation::attributedstring::{CFAttributedString, CFMutableAttributedString};
use core_foundation::base::{CFIndex, CFRange, CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::{CFDictionary, CFMutableDictionary};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::font::CGGlyph;
use core_graphics::geometry::{CGPoint, CGRect, CGSize, CG_ZERO_POINT};
use core_graphics::path::CGPath;
use core_text::frame::CTFrame;
use core_text::framesetter::CTFramesetter;
use core_text::line::CTLine;
use core_text::run::CTRun;
use euclid::{Point2D, Rect, Size2D};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::{Mutex, MutexGuard};

pub mod ffi;
mod format;
pub mod markdown;

pub type Glyph = CGGlyph;

pub struct TextBuf {
    paragraphs: Vec<ParagraphBuf>,
}

impl TextBuf {
    #[inline]
    pub fn new() -> TextBuf {
        TextBuf {
            paragraphs: vec![],
        }
    }

    #[inline]
    pub fn append_paragraph(&mut self, paragraph: ParagraphBuf) {
        self.paragraphs.push(paragraph)
    }

    #[inline]
    pub fn paragraphs(&self) -> &[ParagraphBuf] {
        &self.paragraphs
    }

    #[inline]
    pub fn paragraphs_mut(&mut self) -> &mut [ParagraphBuf] {
        &mut self.paragraphs
    }
}

pub struct ParagraphBuf {
    attributed_string: Mutex<CFMutableAttributedString>,
}

unsafe impl Sync for ParagraphBuf {}

impl ParagraphBuf {
    #[inline]
    pub fn new() -> ParagraphBuf {
        let attributed_string = CFAttributedString::new(CFString::from(""), CFDictionary::new());
        let mutable_attributed_string =
            CFMutableAttributedString::from_attributed_string(attributed_string);
        ParagraphBuf {
            attributed_string: Mutex::new(mutable_attributed_string),
        }
    }

    pub fn from_string(string: &str) -> ParagraphBuf {
        let attributed_string = CFAttributedString::new(CFString::from(string),
                                                        CFDictionary::new());
        let mutable_attributed_string =
            CFMutableAttributedString::from_attributed_string(attributed_string);
        ParagraphBuf {
            attributed_string: Mutex::new(mutable_attributed_string),
        }
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
        let format_stack = attributes_to_formatting(&attributes);
        ParagraphCursor {
            attributed_string: self.attributed_string.lock().unwrap(),
            position: position,
            buffer: CFMutableAttributedString::new(),
            format_stack: format_stack,
        }
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
            attributes.set(format.key.clone(), format.value.clone())
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
}

impl Framesetter {
    pub fn new(text: &TextBuf) -> Framesetter {
        Framesetter {
            framesetters: text.paragraphs().par_iter().map(|paragraph| {
                let attributed_string = paragraph.attributed_string.lock().unwrap();
                let attributed_string = attributed_string.as_attributed_string();
                let framesetter = CTFramesetter::from_attributed_string(attributed_string.clone());
                Mutex::new(ParagraphFramesetter {
                    framesetter: framesetter,
                    attributed_string: attributed_string,
                })
            }).collect(),
        }
    }

    pub fn layout_in_rect(&self, rect: &Rect<f32>) -> Vec<Frame> {
        let mut frames: Vec<_> = self.framesetters.par_iter().map(|paragraph_framesetter| {
            let paragraph_framesetter = paragraph_framesetter.lock().unwrap();
            let range = CFRange::init(0, paragraph_framesetter.attributed_string
                                                              .string()
                                                              .char_len());
            let size = CGSize::new(rect.size.width as f64, rect.size.height as f64);
            let path = CGPath::from_rect(CGRect::new(&CG_ZERO_POINT, &size), None);
            Frame {
                frame: paragraph_framesetter.framesetter.create_frame(range, path, None),
                virtual_size: rect.size,
                origin: Point2D::zero(),
            }
        }).collect();

        // TODO(pcwalton): Vertical writing direction.
        let mut origin = rect.origin;
        for mut frame in &mut frames {
            frame.origin = origin;
            eprintln!("frame origin={:?}", frame.origin);
            origin.y += frame.height();
        }

        frames
    }
}

struct ParagraphFramesetter {
    framesetter: CTFramesetter,
    attributed_string: CFAttributedString,
}

pub struct Frame {
    frame: CTFrame,
    virtual_size: Size2D<f32>,
    origin: Point2D<f32>,
}

impl Frame {
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

    pub fn height(&self) -> f32 {
        let lines = self.frame.lines();
        let line_count = lines.len();
        if line_count == 0 {
            return 0.0
        }

        let last_line = lines.get(line_count - 1).unwrap();
        let mut line_origins = [CG_ZERO_POINT];
        self.frame.get_line_origins(line_count - 1, &mut line_origins);
        let height = last_line.typographic_bounds().descent as f32 - line_origins[0].y as f32 +
            self.virtual_size.height;
        eprintln!("frame height={:?}", height);
        height
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
    pub fn typographic_bounds(&self) -> TypographicBounds {
        let typographic_bounds = self.line.typographic_bounds();
        TypographicBounds {
            width: typographic_bounds.width as f32,
            ascent: typographic_bounds.ascent as f32,
            descent: typographic_bounds.descent as f32,
            leading: typographic_bounds.leading as f32,
        }
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

    pub fn formatting(&self) -> Vec<Format> {
        attributes_to_formatting(&self.run.attributes())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypographicBounds {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub leading: f32,
}

fn attributes_to_formatting(attributes: &CFDictionary<CFString, CFType>) -> Vec<Format> {
    let (attribute_keys, attribute_values) = attributes.get_keys_and_values();
    attribute_keys.into_iter().zip(attribute_values.into_iter()).map(|(key, value)| {
        unsafe {
            Format {
                key: TCFType::wrap_under_get_rule(key as CFStringRef),
                value: TCFType::wrap_under_get_rule(value as CFTypeRef),
            }
        }
    }).collect()
}
