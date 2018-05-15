// pilcrow/src/markdown.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core_graphics::base::CGFloat;
use indexmap::IndexMap;
use indexmap::map::Entry;
use pulldown_cmark::{Event, Parser, Tag};

use format::{Color, Font, Format, Image};
use {Document, DocumentStyle, Paragraph, ParagraphContent, ParagraphStyle};

pub struct MarkdownParser {
    paragraph_styles: [ParagraphStyle; 5],
    fonts: [Font; 4],
}

impl MarkdownParser {
    pub fn new() -> MarkdownParser {
        let plain_font = Font::default_serif();
        let monospace_font = Font::default_monospace();
        let heading_1_font = plain_font.to_size(48.0);
        let heading_2_font = plain_font.to_size(36.0);
        MarkdownParser {
            paragraph_styles: [
                ParagraphStyle::default(),
                ParagraphStyle::default(),
                ParagraphStyle::default(),
                ParagraphStyle::default(),
                ParagraphStyle::new(ParagraphContent::Rule),
            ],
            fonts: [
                plain_font,
                monospace_font,
                heading_1_font,
                heading_2_font,
            ],
        }
    }

    pub fn add_to_document(self, document: &mut Document, string: &str) -> ParseResults {
        let mut lists = vec![];
        let mut image_urls = IndexMap::new();
        let mut next_link_id = 0;

        let mut parser = Parser::new(string);
        while let Some(event) = parser.next() {
            match event {
                Event::Start(tag @ Tag::Paragraph) |
                Event::Start(tag @ Tag::CodeBlock(_)) |
                Event::Start(tag @ Tag::Header(_)) |
                Event::Start(tag @ Tag::Item) |
                Event::Start(tag @ Tag::BlockQuote) => {
                    let block_selector = match tag {
                        Tag::Header(1) => BlockSelector::Heading1,
                        Tag::Header(_) => BlockSelector::Heading2,
                        Tag::CodeBlock(_) => BlockSelector::Code,
                        _ => BlockSelector::Body,
                    };

                    let mut paragraph_done = false;
                    while !paragraph_done {
                        let paragraph_style = self.paragraph_styles[block_selector as usize]
                                                  .clone();
                        let mut current_paragraph = Paragraph::new(paragraph_style);

                        {
                            let mut current_cursor = current_paragraph.edit_at(0);

                            let body_font_selector = match tag {
                                Tag::CodeBlock(_) => InlineSelector::Code,
                                Tag::Header(1) => InlineSelector::Heading1,
                                Tag::Header(_) => InlineSelector::Heading2,
                                _ => InlineSelector::Body,
                            };

                            let body_font = self.fonts[body_font_selector as usize].clone();
                            current_cursor.push_format(Format::from_font(body_font));

                            if tag == Tag::Item {
                                match lists.last_mut() {
                                    Some(&mut List::Unordered) => current_cursor.push_string("• "),
                                    Some(&mut List::Ordered(ref mut number)) => {
                                        current_cursor.push_string(&format!("{}. ", number));
                                        *number += 1
                                    }
                                    _ => {}
                                }
                            }

                            while let Some(event) = parser.next() {
                                match event {
                                    Event::End(ref end_tag) if *end_tag == tag => {
                                        paragraph_done = true;
                                        break
                                    }
                                    Event::Start(Tag::Emphasis) => {
                                        // This is a bit of an unfortunate design on Cocoa's part.
                                        // It would be better to have separate "emphasis" and
                                        // "strong" attributes instead of a single
                                        // `NSFontAttributeName`, so that we don't need this ugly
                                        // song and dance.
                                        let mut new_font =
                                            current_cursor.format_stack()
                                                        .iter()
                                                        .rev()
                                                        .find(|format| format.font().is_some())
                                                        .unwrap()
                                                        .font()
                                                        .unwrap();
                                        if let Some(italic_font) = new_font.to_italic() {
                                            new_font = italic_font
                                        }
                                        current_cursor.push_format(Format::from_font(new_font));
                                    }
                                    Event::Start(Tag::Strong) => {
                                        let mut new_font =
                                            current_cursor.format_stack()
                                                        .iter()
                                                        .rev()
                                                        .find(|format| format.font().is_some())
                                                        .unwrap()
                                                        .font()
                                                        .unwrap();
                                        if let Some(bold_font) = new_font.to_bold() {
                                            new_font = bold_font
                                        }
                                        current_cursor.push_format(Format::from_font(new_font));
                                    }
                                    Event::Start(Tag::Link(url, _)) => {
                                        let url = url.to_string();
                                        current_cursor.push_format(Format::from_link(next_link_id,
                                                                                     url));
                                        next_link_id += 1;
                                    }
                                    Event::Start(Tag::Code) => {
                                        let font = self.fonts[InlineSelector::Code as usize]
                                                       .clone();
                                        current_cursor.push_format(Format::from_font(font));
                                    }
                                    Event::Start(Tag::Image(url, alt_text)) => {
                                        let url = url.to_string();
                                        let image_count = image_urls.len();
                                        let image_id = match image_urls.entry(url) {
                                            Entry::Vacant(entry) => {
                                                entry.insert(image_count as u32);
                                                image_count as u32
                                            }
                                            Entry::Occupied(entry) => *entry.get(),
                                        };
                                        let image = Image {
                                            id: image_id,
                                            alt_text: alt_text.to_string(),
                                        };
                                        current_cursor.push_format(Format::from_image(image_id));
                                        current_cursor.push_string("\u{fffc}");
                                    }
                                    Event::End(Tag::Emphasis) |
                                    Event::End(Tag::Strong) |
                                    Event::End(Tag::Code) |
                                    Event::End(Tag::Link(_, _)) |
                                    Event::End(Tag::Image(_, _)) => {
                                        current_cursor.pop_format()
                                    }
                                    Event::Text(string) => {
                                        if !current_cursor.format_stack().iter().any(|format| {
                                            format.image().is_some()
                                        }) {
                                            current_cursor.push_string(&string)
                                        }
                                    }
                                    Event::SoftBreak => current_cursor.push_string(" "),
                                    Event::HardBreak => break,
                                    _ => {}
                                }
                            }
                            current_cursor.pop_format();
                            current_cursor.commit();
                        }

                        document.append_paragraph(current_paragraph)
                    }
                }

                Event::Start(Tag::List(None)) => lists.push(List::Unordered),
                Event::Start(Tag::List(Some(number))) => lists.push(List::Ordered(number)),
                Event::End(Tag::List(_)) => drop(lists.pop()),

                Event::End(Tag::Rule) => {
                    let style = self.paragraph_styles[BlockSelector::Rule as usize].clone();
                    document.append_paragraph(Paragraph::new(style));
                }

                _ => {}
            }
        }

        ParseResults {
            image_urls: image_urls.into_iter().map(|(url, _)| url).collect(),
        }
    }

    #[inline]
    pub fn set_font(&mut self, selector: InlineSelector, font: Font) {
        self.fonts[selector as usize] = font
    }

    #[inline]
    pub fn paragraph_style_mut(&mut self, selector: BlockSelector) -> &mut ParagraphStyle {
        &mut self.paragraph_styles[selector as usize]
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum InlineSelector {
    Body = 0,
    Code,
    Heading1,
    Heading2,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum BlockSelector {
    Body = 0,
    Code,
    Heading1,
    Heading2,
    Rule,
}

#[derive(Clone, Copy, PartialEq)]
pub enum List {
    Ordered(usize),
    Unordered,
}

#[derive(Debug)]
pub struct ParseResults {
    image_urls: Vec<String>,
}

impl ParseResults {
    #[inline]
    pub fn image_count(&self) -> usize {
        self.image_urls.len()
    }

    #[inline]
    pub fn image_url(&self, image_index: usize) -> &str {
        &self.image_urls[image_index]
    }
}
