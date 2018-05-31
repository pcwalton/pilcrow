// pilcrow/src/tests/typesetter.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::File;
use std::io::Read;
use std::iter;
use std::sync::Arc;

use font::FontLike;
use font_collection::FontCollection;
use font_family::FontFamily;
use platform::Font;
use simple_styled_text::SimpleStyledTextBuf;
use styled_text::{InitialStyle, StyledText};
use typesetter::Typesetter;

#[test]
pub fn test() {
    let mut font_data = vec![];
    File::open("resources/tests/EBGaramond12-Regular.ttf").unwrap()
                                                          .read_to_end(&mut font_data)
                                                          .unwrap();
    let font = Font::from_bytes(Arc::new(font_data)).unwrap();
    let font_family = FontFamily::from_fonts(iter::once(font));
    let font_collection = FontCollection::from_font_families(iter::once(font_family));
    let initial_style = InitialStyle::from_font_family(Arc::new(font_collection));
    let text = SimpleStyledTextBuf::new("Hello world!".to_owned(), initial_style);
    let mut typesetter = Typesetter::new(text.borrow());
    let range = 0..typesetter.text().byte_length();
    let line = typesetter.create_line(range);
    assert_eq!(line.runs().len(), 1);
    let run = &line.runs()[0];
    eprintln!("{:?}", run.glyphs());
    eprintln!("{:?}", run.advances());
}
