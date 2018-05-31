// pilcrow/src/line.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Range;

use run::Run;

#[derive(Clone, Debug)]
pub struct Line {
    runs: Vec<Run>,
    string_range: Range<usize>,
}

impl Line {
    #[inline]
    pub(crate) fn from_runs_and_range(runs: Vec<Run>, string_range: Range<usize>) -> Line {
        Line {
            runs,
            string_range,
        }
    }

    #[inline]
    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    #[inline]
    pub fn string_range(&self) -> Range<usize> {
        self.string_range.clone()
    }
}
