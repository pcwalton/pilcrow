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
use euclid::SideOffsets2D;
use libc::c_uchar;
use std::cmp;
use std::ptr;
use std::slice;
use std::str;

use format::Font;
use markdown::{BlockSelector, InlineSelector, MarkdownParser, ParseResults};
use {Document, DocumentStyle, Paragraph, ParagraphStyle, TextLocation};

pub type NativeFont = CTFont;

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_new() -> *mut Document {
    Box::into_raw(Box::new(Document::new()))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_clear(text: *mut Document) {
    (*text).clear()
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_append_paragraph(text: *mut Document,
                                                           paragraph: *mut Paragraph) {
    (*text).append_paragraph(*Box::from_raw(paragraph))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_append_document(this_document: *mut Document,
                                                          other_document: *mut Document) {
    (*this_document).append_document(*Box::from_raw(other_document))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_copy_string_in_range(document: *const Document,
                                                               start: *const TextLocation,
                                                               end: *const TextLocation)
                                                               -> *mut String {
    Box::into_raw(Box::new((*document).copy_string_in_range((*start)..(*end))))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_copy_string(document: *const Document) -> *mut String {
    Box::into_raw(Box::new((*document).copy_string()))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_get_style(document: *mut Document)
                                                    -> *mut DocumentStyle {
    (*document).style_mut()
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_style_set_margin(style: *mut DocumentStyle,
                                                           top: f32,
                                                           right: f32,
                                                           bottom: f32,
                                                           left: f32) {
    eprintln!("document style margins={} {} {} {}", top, right, bottom, left);
    (*style).margin = SideOffsets2D::new(top, right, bottom, left)
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_document_style_copy(dest: *mut DocumentStyle,
                                                     src: *const DocumentStyle) {
    *dest = (*src).clone()
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_paragraph_style_set_margin(style: *mut ParagraphStyle,
                                                            top: f32,
                                                            right: f32,
                                                            bottom: f32,
                                                            left: f32) {
    (*style).margin = SideOffsets2D::new(top, right, bottom, left)
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_paragraph_destroy(paragraph: *mut Paragraph) {
    drop(Box::from_raw(paragraph))
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
pub unsafe extern "C" fn pilcrow_markdown_parser_add_to_document(parser: *mut MarkdownParser,
                                                                 string: *const c_uchar,
                                                                 len: usize,
                                                                 document: *mut Document)
                                                                 -> *mut ParseResults {
    let string = str::from_utf8(slice::from_raw_parts(string, len)).unwrap();
    let parse_results = Box::from_raw(parser).add_to_document(&mut *document, string);
    Box::into_raw(Box::new(parse_results))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parser_set_font(parser: *mut MarkdownParser,
                                                          selector: InlineSelector,
                                                          font: *mut Font) {
    (*parser).set_font(selector, *Box::from_raw(font))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parser_get_paragraph_style(parser: *mut MarkdownParser,
                                                                     selector: BlockSelector)
                                                                     -> *mut ParagraphStyle {
    (*parser).paragraph_style_mut(selector)
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parse_results_destroy(parse_results: *mut ParseResults) {
    drop(Box::from_raw(parse_results))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parse_results_get_image_count(parse_results:
                                                                        *mut ParseResults)
                                                                        -> usize {
    (*parse_results).image_count()
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parse_results_get_image_url_len(parse_results:
                                                                          *mut ParseResults,
                                                                          image_index: usize)
                                                                          -> usize {
    (*parse_results).image_url(image_index).len()
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_markdown_parse_results_get_image_url(parse_results:
                                                                      *mut ParseResults,
                                                                      image_index: usize,
                                                                      buffer: *mut c_uchar,
                                                                      buffer_len: usize) {
    let url = (*parse_results).image_url(image_index);
    ptr::copy_nonoverlapping(url.as_ptr(), buffer, cmp::min(url.len(), buffer_len))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_string_destroy(string: *mut String) {
    drop(Box::from_raw(string))
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_string_get_byte_len(string: *const String) -> usize {
    (*string).len()
}

#[no_mangle]
pub unsafe extern "C" fn pilcrow_string_get_chars(string: *const String) -> *const u8 {
    (*string).as_ptr()
}
