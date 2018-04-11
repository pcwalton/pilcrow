// pilcrow/src/ffi.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use euclid::{Point2D, Rect, Size2D};
use libc::c_uchar;
use std::mem;
use std::slice;
use std::str;
use {Frame, Framesetter, Line, Run, TextBuf};

#[repr(C)]
pub struct Lines {
    pub lines: *mut Line,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct Runs {
    pub runs: *mut Run,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct Glyphs {
    pub glyphs: *mut u16,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct GlyphPositions {
    pub positions: *mut GlyphPosition,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct GlyphPosition {
    pub x: f32,
    pub y: f32,
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_text_buf_new_from_string(string: *const c_uchar, len: usize)
                                                          -> *mut TextBuf {
    let string = str::from_utf8(slice::from_raw_parts(string, len)).unwrap();
    Box::into_raw(Box::new(TextBuf::from_string(string)))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_text_buf_destroy(buf: *mut TextBuf) {
    drop(Box::from_raw(buf))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_framesetter_new(text: *const TextBuf) -> *mut Framesetter {
    Box::into_raw(Box::new(Framesetter::new(&*text)))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_framesetter_destroy(framesetter: *mut Framesetter) {
    drop(Box::from_raw(framesetter))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_framesetter_layout_range_in_rect(framesetter: *mut Framesetter,
                                                                  start: usize,
                                                                  end: usize,
                                                                  x: f32,
                                                                  y: f32,
                                                                  width: f32,
                                                                  height: f32)
                                                                  -> *mut Frame {
    let rect = Rect::new(Point2D::new(x, y), Size2D::new(width, height));
    Box::into_raw(Box::new((*framesetter).layout_range_in_rect(start..end, &rect)))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_frame_destroy(frame: *mut Frame) {
    drop(Box::from_raw(frame))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_frame_get_lines(frame: *mut Frame) -> Lines {
    let mut lines = (*frame).lines();
    let out_lines = Lines {
        lines: lines.as_mut_ptr(),
        len: lines.len(),
        cap: lines.capacity(),
    };
    mem::forget(lines);
    out_lines
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_lines_destroy(lines: *mut Lines) {
    drop(Vec::from_raw_parts((*lines).lines, (*lines).len, (*lines).cap))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_line_get_runs(line: *mut Line) -> Runs {
    let mut runs = (*line).runs();
    let out_runs = Runs {
        runs: runs.as_mut_ptr(),
        len: runs.len(),
        cap: runs.capacity(),
    };
    mem::forget(runs);
    out_runs
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_runs_destroy(runs: *mut Runs) {
    drop(Vec::from_raw_parts((*runs).runs, (*runs).len, (*runs).cap))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_run_get_glyphs(run: *mut Run) -> Glyphs {
    let mut glyphs = (*run).glyphs();
    let out_glyphs = Glyphs {
        glyphs: glyphs.as_mut_ptr(),
        len: glyphs.len(),
        cap: glyphs.capacity(),
    };
    mem::forget(glyphs);
    out_glyphs
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_run_get_positions(run: *mut Run) -> GlyphPositions {
    let mut positions = (*run).positions();
    let out_positions = GlyphPositions {
        positions: positions.as_mut_ptr() as *mut GlyphPosition,
        len: positions.len(),
        cap: positions.capacity(),
    };
    mem::forget(positions);
    out_positions
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_glyphs_destroy(glyphs: *mut Glyphs) {
    drop(Vec::from_raw_parts((*glyphs).glyphs, (*glyphs).len, (*glyphs).cap))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_glyph_positions_destroy(positions: *mut GlyphPositions) {
    drop(Vec::from_raw_parts((*positions).positions, (*positions).len, (*positions).cap))
}
