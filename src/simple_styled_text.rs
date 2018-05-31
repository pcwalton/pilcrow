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

use platform::Font;
use styled_text::{InitialStyle, ReplacedContentMetrics, StyledText};
use styled_text::{StyledTextNode, StyledTextNodeBuf};

#[derive(Clone)]
pub struct SimpleStyledTextBuf {
    nodes: Vec<StyledTextNodeBuf>,
    initial_style: InitialStyle,
}

impl SimpleStyledTextBuf {
    pub fn new(string: String, initial_style: InitialStyle) -> SimpleStyledTextBuf {
        SimpleStyledTextBuf {
            initial_style: initial_style,
            nodes: vec![StyledTextNodeBuf::String(string)],
        }
    }

    pub fn nodes(&self) -> &[StyledTextNodeBuf] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<StyledTextNodeBuf> {
        &mut self.nodes
    }

    pub fn borrow(&self) -> SimpleStyledText {
        SimpleStyledText {
            buffer: self,
            index: 0,
        }
    }

    /*
       TODO(pcwalton): Untested

    pub fn replace_string(&mut self, range: Range<usize>, new_string: &str) {
        // Find and modify the first node.
        let (mut current_byte_index, mut current_node_index) = (0, 0);
        loop {
            let mut node = &mut self.nodes[current_node_index];
            if let SimpleStyledTextBufNode::String(ref mut dest_string) = node {
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
            if let SimpleStyledTextBufNode::String(ref mut dest_string) = *node {
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
    */
}

pub struct SimpleStyledText<'a> {
    buffer: &'a SimpleStyledTextBuf,
    index: isize,
}

impl<'a> StyledText for SimpleStyledText<'a> {
    #[inline]
    fn move_prev(&mut self) -> bool {
        let ok = self.index >= 0;
        if ok {
            self.index -= 1
        }
        ok
    }

    #[inline]
    fn move_next(&mut self) -> bool {
        let ok = self.index < self.buffer.nodes.len() as isize;
        if ok {
            self.index += 1
        }
        ok
    }

    #[inline]
    fn get(&self) -> StyledTextNode {
        assert!(self.index >= 0 && self.index < self.buffer.nodes.len() as isize);
        self.buffer.nodes[self.index as usize].borrow()
    }

    #[inline]
    fn initial_style(&self) -> InitialStyle {
        self.buffer.initial_style.clone()
    }
}
