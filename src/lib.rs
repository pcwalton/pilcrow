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

use core_foundation::attributedstring::CFAttributedString;
use core_foundation::base::CFRange;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_graphics::font::CGGlyph;
use core_graphics::geometry::{CGPoint, CGRect, CGSize, CG_ZERO_POINT};
use core_graphics::path::CGPath;
use core_text::frame::CTFrame;
use core_text::framesetter::CTFramesetter;
use core_text::line::CTLine;
use core_text::run::CTRun;
use euclid::{Point2D, Rect};
use std::ops::Range;

pub struct TextBuf {
    attributed_string: CFAttributedString,
}

impl TextBuf {
    #[inline]
    pub fn from_string(string: &str) -> TextBuf {
        TextBuf {
            attributed_string: CFAttributedString::new(CFString::from(string), CFDictionary::new()),
        }
    }
}

pub struct Framesetter {
    framesetter: CTFramesetter,
}

impl Framesetter {
    pub fn new(text: TextBuf) -> Framesetter {
        Framesetter {
            framesetter: CTFramesetter::from_attributed_string(text.attributed_string.clone())
        }
    }

    pub fn layout_range_in_rect(&self, range: Range<usize>, rect: &Rect<f32>) -> Frame {
        let range = CFRange::init(range.start as i64, range.len() as i64);
        let origin = CGPoint::new(rect.origin.x as f64, rect.origin.y as f64);
        let size = CGSize::new(rect.size.width as f64, rect.size.height as f64);
        let path = CGPath::from_rect(CGRect::new(&origin, &size), None);
        Frame {
            frame: self.framesetter.create_frame(range, path, None),
        }
    }
}

pub struct Frame {
    frame: CTFrame,
}

impl Frame {
    pub fn lines(&self) -> Vec<Line> {
        let lines = self.frame.lines();
        let mut line_origins = vec![CG_ZERO_POINT; lines.len() as usize];
        self.frame.get_line_origins(0, &mut line_origins);
        lines.into_iter().zip(line_origins.into_iter()).map(|(line, origin)| {
            Line {
                line: (*line).clone(),
                origin: Point2D::new(origin.x as f32, origin.y as f32),
            }
        }).collect()
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
}

pub struct Run {
    run: CTRun,
}

impl Run {
    #[inline]
    pub fn glyph_count(&self) -> usize {
        self.run.glyph_count() as usize
    }

    pub fn glyphs(&self) -> Vec<CGGlyph> {
        let mut glyphs = vec![0; self.glyph_count()];
        self.run.get_glyphs(0, &mut glyphs);
        glyphs
    }

    pub fn positions(&self) -> Vec<Point2D<f32>> {
        let mut positions = vec![CG_ZERO_POINT; self.glyph_count()];
        self.run.get_positions(0, &mut positions);
        positions.into_iter().map(|p| Point2D::new(p.x as f32, p.y as f32)).collect()
    }
}
