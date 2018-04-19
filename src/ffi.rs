// pilcrow/src/ffi.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core_text::font::CTFont;
use libc::c_uchar;
use std::slice;
use std::str;

use format::Font;
use markdown::{FontSelector, MarkdownParser};
use {ParagraphBuf, TextBuf};

pub type NativeFont = CTFont;

#[no_mangle]
pub unsafe extern "C" fn pilcrow_text_buf_new() -> *mut TextBuf {
    Box::into_raw(Box::new(TextBuf::new()))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_text_buf_append_paragraph(text: *mut TextBuf,
                                                           paragraph: *mut ParagraphBuf) {
    (*text).append_paragraph(*Box::from_raw(paragraph))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_paragraph_buf_new_from_string(string: *const c_uchar, len: usize)
                                                               -> *mut ParagraphBuf {
    let string = str::from_utf8(slice::from_raw_parts(string, len)).unwrap();
    Box::into_raw(Box::new(ParagraphBuf::from_string(string)))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_paragraph_buf_destroy(buf: *mut ParagraphBuf) {
    drop(Box::from_raw(buf))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_font_new_from_native(native_font: NativeFont) -> *mut Font {
    Box::into_raw(Box::new(Font::from_native_font(native_font.clone())))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_font_destroy(font: *mut Font) {
    drop(Box::from_raw(font))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parser_new() -> *mut MarkdownParser {
    Box::into_raw(Box::new(MarkdownParser::new()))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parser_destroy(parser: *mut MarkdownParser) {
    drop(Box::from_raw(parser))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parser_add_to_text_buf(parser: *mut MarkdownParser,
                                                                 string: *const c_uchar,
                                                                 len: usize,
                                                                 text_buf: *mut TextBuf) {
    let string = str::from_utf8(slice::from_raw_parts(string, len)).unwrap();
    Box::from_raw(parser).add_to_text_buf(&mut *text_buf, string)
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parser_set_font(parser: *mut MarkdownParser,
                                                          selector: FontSelector,
                                                          font: *mut Font) {
    (*parser).set_font(selector, *Box::from_raw(font))
}
