// pilcrow/src/tests/typesetter.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::f32;
use std::fs::File;
use std::io::Read;
use std::iter;
use std::sync::Arc;

use font::FontLike;
use font_family::FontFamily;
use font_set::FontSet;
use platform::Font;
use run::Run;
use simple_styled_text::SimpleStyledTextBuf;
use styled_text::{InitialStyle, StyledText};
use typesetter::Typesetter;

static TEST_FONT_PATH_SERIF: &'static str = "resources/tests/EBGaramond12-Regular.ttf";

static TEST_MULTILINE_TEXT: &'static str = "\
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Mauris convallis vel lorem eu dictum. \
Vestibulum id odio sit amet tellus efficitur tincidunt in vitae urna. Phasellus elit neque, \
molestie non venenatis non, consectetur nec elit.\
";

#[test]
pub fn test_single_line() {
    let font = Font::from_reader(File::open(TEST_FONT_PATH_SERIF).unwrap()).unwrap();
    let text = SimpleStyledTextBuf::new("Hello world!".to_owned(), InitialStyle::from_font(font));
    let line = Typesetter::new(text, f32::INFINITY).create_single_line();

    assert_eq!(line.runs().len(), 1);

    // TODO(pcwalton): Test that the correct font was selected. Needs a `PartialEq` implementation
    // on `Font`.
    check_run(&line.runs()[0],
              &[43, 72, 79, 79, 82, 3, 90, 82, 85, 79, 71, 4],
              &[13.0, 6.0, 4.0, 4.0, 8.0, 3.0, 11.0, 8.0, 5.0, 4.0, 8.0, 5.0],
              16.0);
}

#[test]
pub fn test_multiline() {
    let font = Font::from_reader(File::open(TEST_FONT_PATH_SERIF).unwrap()).unwrap();
    let string = TEST_MULTILINE_TEXT.to_owned();
    let text = SimpleStyledTextBuf::new(string, InitialStyle::from_font(font));

    let mut typesetter = Typesetter::new(text, 640.0);
    let mut line_breaks = typesetter.line_breaks().to_vec();
    assert_eq!(line_breaks.len(), 3);

    let line = typesetter.create_line(0..(line_breaks[0] as usize));
    assert_eq!(line.runs().len(), 1);
    check_run(&line.runs()[0],
              &[47, 82, 85, 72, 80, 3, 76, 83, 86, 88, 80, 3, 71, 82, 79, 82, 85, 3, 86, 76, 87,
              3, 68, 80, 72, 87, 15, 3, 70, 82, 81, 86, 72, 70, 87, 72, 87, 88, 85, 3, 68, 71,
              76, 83, 76, 86, 70, 76, 81, 74, 3, 72, 79, 76, 87, 17, 3, 48, 68, 88, 85, 76, 86,
              3, 70, 82, 81, 89, 68, 79, 79, 76, 86, 3, 89, 72, 79, 3, 79, 82, 85, 72, 80, 3, 72,
              88, 3, 71, 76, 70, 87, 88, 80, 17, 3, 57, 72, 86, 87, 76, 69, 88, 79, 88, 80, 3],
              &[9.0, 8.0, 5.0, 6.0, 12.0, 3.0, 4.0, 8.0, 5.0, 8.0, 12.0, 3.0, 8.0, 8.0, 4.0, 8.0,
              5.0, 3.0, 5.0, 4.0, 5.0, 3.0, 6.0, 12.0, 6.0, 5.0, 4.0, 3.0, 6.0, 8.0, 8.0, 5.0,
              6.0, 6.0, 5.0, 6.0, 5.0, 8.0, 5.0, 3.0, 6.0, 8.0, 4.0, 8.0, 4.0, 5.0, 6.0, 4.0,
              8.0, 7.0, 3.0, 6.0, 4.0, 4.0, 5.0, 4.0, 3.0, 14.0, 6.0, 8.0, 5.0, 4.0, 5.0, 3.0,
              6.0, 8.0, 8.0, 7.0, 6.0, 4.0, 4.0, 4.0, 5.0, 3.0, 7.0, 6.0, 4.0, 3.0, 4.0, 8.0,
              5.0, 6.0, 12.0, 3.0, 6.0, 8.0, 3.0, 8.0, 4.0, 6.0, 5.0, 8.0, 12.0, 4.0, 3.0, 11.0,
              6.0, 5.0, 5.0, 4.0, 8.0, 8.0, 4.0, 8.0, 12.0, 3.0],
              16.0);

    let line = typesetter.create_line((line_breaks[0] as usize)..(line_breaks[1] as usize));
    assert_eq!(line.runs().len(), 1);
    check_run(&line.runs()[0],
              &[76, 71, 3, 82, 71, 76, 82, 3, 86, 76, 87, 3, 68, 80, 72, 87, 3, 87, 72, 79, 79,
              88, 86, 3, 72, 3000, 3001, 2989, 70, 76, 87, 88, 85, 3, 87, 76, 81, 70, 76, 71, 88,
              81, 87, 3, 76, 81, 3, 89, 76, 87, 68, 72, 3, 88, 85, 81, 68, 17, 3, 51, 75, 68, 86,
              72, 79, 79, 88, 86, 3, 72, 79, 76, 87, 3, 81, 72, 84, 88, 72, 15, 3, 80, 82, 79,
              72, 86, 87, 76, 72, 3, 81, 82, 81, 3, 89, 72, 81, 72, 81, 68, 87, 76, 86, 3, 81,
              82, 81, 15, 3],
              &[4.0, 8.0, 3.0, 8.0, 8.0, 4.0, 8.0, 3.0, 5.0, 4.0, 5.0, 3.0, 6.0, 12.0, 6.0, 5.0,
              3.0, 5.0, 6.0, 4.0, 4.0, 8.0, 5.0, 3.0, 6.0, 4.0, 4.0, 4.0, 6.0, 4.0, 5.0, 8.0,
              5.0, 3.0, 5.0, 4.0, 8.0, 6.0, 4.0, 8.0, 8.0, 8.0, 5.0, 3.0, 4.0, 8.0, 3.0, 7.0,
              4.0, 5.0, 6.0, 6.0, 3.0, 8.0, 5.0, 8.0, 6.0, 4.0, 3.0, 9.0, 8.0, 6.0, 5.0, 6.0,
              4.0, 4.0, 8.0, 5.0, 3.0, 6.0, 4.0, 4.0, 5.0, 3.0, 8.0, 6.0, 8.0, 8.0, 6.0, 4.0,
              3.0, 12.0, 8.0, 4.0, 6.0, 5.0, 5.0, 4.0, 6.0, 3.0, 8.0, 8.0, 8.0, 3.0, 7.0, 6.0,
              8.0, 6.0, 8.0, 6.0, 5.0, 4.0, 5.0, 3.0, 8.0, 8.0, 8.0, 4.0, 3.0],
              16.0);

    let line = typesetter.create_line((line_breaks[1] as usize)..(line_breaks[2] as usize));
    assert_eq!(line.runs().len(), 1);
    check_run(&line.runs()[0],
              &[70, 82, 81, 86, 72, 70, 87, 72, 87, 88, 85, 3, 81, 72, 70, 3, 72, 79, 76, 87, 17],
              &[6.0, 8.0, 8.0, 5.0, 6.0, 6.0, 5.0, 6.0, 5.0, 8.0, 5.0, 3.0, 8.0, 6.0, 6.0, 3.0,
              6.0, 4.0, 4.0, 5.0, 4.0],
              16.0);
}

fn check_run(run: &Run, glyphs: &[u32], advances: &[f32], size: f32) {
    assert_eq!(run.glyphs(), glyphs);
    assert_eq!(run.advances(), advances);
    assert_eq!(run.size(), size);
}
