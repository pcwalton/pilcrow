// pilcrow/src/simple_styled_text.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Range;
use std::sync::Arc;

use font::Font;
use styled_text::ReplacedContentMetrics;

#[derive(Clone)]
pub struct SimpleStyledText {
    nodes: Vec<SimpleStyledTextNode>,
}

impl SimpleStyledText {
    pub fn new(string: String, styles: SimpleStyles) -> SimpleStyledText {
        SimpleStyledText {
            nodes: vec![
                SimpleStyledTextNode::Start(SimpleStyle::Font(styles.font)),
                SimpleStyledTextNode::Start(SimpleStyle::Size(styles.size)),
                SimpleStyledTextNode::Start(SimpleStyle::LetterSpacing(styles.letter_spacing)),
                SimpleStyledTextNode::Start(SimpleStyle::WordSpacing(styles.word_spacing)),
                SimpleStyledTextNode::String(string),
                SimpleStyledTextNode::End,
                SimpleStyledTextNode::End,
                SimpleStyledTextNode::End,
                SimpleStyledTextNode::End,
            ],
        }
    }

    pub fn replace_string(&mut self, range: Range<usize>, new_string: &str) {
        // Find and modify the first node.
        let (mut current_byte_index, mut current_node_index) = (0, 0);
        loop {
            let mut node = &mut self.nodes[current_node_index];
            if let SimpleStyledTextNode::String(ref mut dest_string) = node {
                if range.start < current_byte_index + dest_string.len() {
                    if range.end <= current_byte_index + dest_string.len() {
                        let mut rest = dest_string[(range.end - current_byte_index)..].to_owned();
                        dest_string.truncate(range.start - current_byte_index);
                        dest_string.push_str(new_string);
                        dest_string.push_str(rest);
                        return
                    }
                    let new_byte_index = current_byte_index + dest_string.len();
                    dest_string.truncate(range.start - current_byte_index);
                    dest_string.push_str(new_string);
                    current_byte_index = new_byte_index;
                    current_node_index += 1;
                    break
                }
                current_byte_index += dest_string.len()
            }
            current_node_index += 1;
            if current_node_index >= self.nodes.len() {
                return
            }
        }

        // Find and modify the last node.
        self.nodes[current_node_index..].retain(|node| {
            if let SimpleStyledTextNode::String(ref mut dest_string) = *node {
                let new_byte_index = current_byte_index + dest_string.len();
                let past_end = current_byte_index >= range.end;
                if !past_end {
                    let reached_end = range.end <= current_byte_index + dest_string.len();
                    if reached_end {
                        *dest_string = dest_string[0..(range.end - current_byte_index)].to_owned();
                    }
                    current_byte_index = new_byte_index;
                    return reached_end
                }
                current_byte_index = new_byte_index;
                past_end
            } else {
                true
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct SimpleStyles {
    pub font: Arc<Font>,
    pub size: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

#[derive(Clone, Debug)]
pub enum SimpleStyle {
    Font(Arc<Font>),
    Size(f32),
    LetterSpacing(f32),
    WordSpacing(f32),
    ReplacedContent(ReplacedContentMetrics),
}

pub enum SimpleStyledTextNode {
    String(String),
    Start(SimpleStyle),
    End,
}
