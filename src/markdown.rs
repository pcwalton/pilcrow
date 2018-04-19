// pilcrow/src/markdown.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use pulldown_cmark::{Event, Parser, Tag};

use format::{Font, Format};
use {ParagraphBuf, TextBuf};

pub struct MarkdownParser {
    fonts: [Font; 4],
}

impl MarkdownParser {
    pub fn new() -> MarkdownParser {
        let plain_font = Font::default_serif();
        let monospace_font = Font::default_monospace();
        let heading_1_font = plain_font.to_size(48.0);
        let heading_2_font = plain_font.to_size(36.0);
        MarkdownParser {
            fonts: [
                plain_font,
                monospace_font,
                heading_1_font,
                heading_2_font,
            ],
        }
    }

    pub fn add_to_text_buf(self, text_buf: &mut TextBuf, string: &str) {
        let mut parser = Parser::new(string);
        while let Some(event) = parser.next() {
            match event {
                Event::Start(tag @ Tag::Paragraph) |
                Event::Start(tag @ Tag::CodeBlock(_)) |
                Event::Start(tag @ Tag::Header(_)) => {
                    let mut current_paragraph = ParagraphBuf::new();
                    {
                        let mut current_cursor = current_paragraph.edit_at(0);

                        let body_font_selector = match tag {
                            Tag::CodeBlock(_) => FontSelector::Code,
                            Tag::Header(1) => FontSelector::Heading1,
                            Tag::Header(_) => FontSelector::Heading2,
                            _ => FontSelector::Plain,
                        };

                        let body_font = self.fonts[body_font_selector as usize].clone();
                        current_cursor.push_format(Format::from_font(body_font));

                        while let Some(event) = parser.next() {
                            match event {
                                Event::End(ref end_tag) if *end_tag == tag => break,
                                Event::Start(Tag::Emphasis) => {
                                    // This is a bit of an unfortunate design on Cocoa's part. It
                                    // would be better to have separate "emphasis" and "strong"
                                    // attributes instead of a single `NSFontAttributeName`, so
                                    // that we don't need this ugly song and dance.
                                    let new_font =
                                        current_cursor.format_stack()
                                                      .iter()
                                                      .rev()
                                                      .find(|format| format.font().is_some())
                                                      .unwrap()
                                                      .font()
                                                      .unwrap()
                                                      .to_italic();
                                    current_cursor.push_format(Format::from_font(new_font));
                                }
                                Event::Start(Tag::Strong) => {
                                    let new_font =
                                        current_cursor.format_stack()
                                                      .iter()
                                                      .rev()
                                                      .find(|format| format.font().is_some())
                                                      .unwrap()
                                                      .font()
                                                      .unwrap()
                                                      .to_bold();
                                    current_cursor.push_format(Format::from_font(new_font));
                                }
                                Event::Start(Tag::Code) => {
                                    let font = self.fonts[FontSelector::Code as usize].clone();
                                    current_cursor.push_format(Format::from_font(font));
                                }
                                Event::End(Tag::Emphasis) |
                                Event::End(Tag::Strong) |
                                Event::End(Tag::Code) => {
                                    current_cursor.pop_format()
                                }
                                Event::Text(string) => current_cursor.push_string(&string),
                                Event::SoftBreak => current_cursor.push_string(" "),
                                Event::HardBreak => current_cursor.push_string("\n"),
                                _ => {}
                            }
                        }
                        current_cursor.pop_format();
                        current_cursor.commit();
                    }
                    text_buf.append_paragraph(current_paragraph)
                }
                _ => {}
            }
        }
    }

    pub fn set_font(&mut self, selector: FontSelector, font: Font) {
        self.fonts[selector as usize] = font
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum FontSelector {
    Plain = 0,
    Code,
    Heading1,
    Heading2,
}
