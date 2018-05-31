// pilcrow/src/font_collection.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use minikin_sys::{minikin_font_collection_create, minikin_font_collection_destroy};
use minikin_sys::{minikin_font_collection_t};
use std::mem;

use font_family::FontFamily;

#[derive(Debug)]
pub struct FontCollection {
    minikin_font_collection: *mut minikin_font_collection_t,
}

impl Drop for FontCollection {
    #[inline]
    fn drop(&mut self) {
        // FIXME(pcwalton): Drop the font families too?
        unsafe {
            minikin_font_collection_destroy(self.minikin_font_collection)
        }
    }
}

impl FontCollection {
    pub fn from_font_families<I>(font_families: I) -> FontCollection
                                 where I: Iterator<Item = FontFamily> {
        let mut minikin_families = vec![];
        for font_family in font_families {
            minikin_families.push(font_family.as_minikin_font_family());
            mem::forget(font_family);
        }
        let minikin_families_ptr = minikin_families.as_mut_ptr();
        unsafe {
            FontCollection {
                minikin_font_collection: minikin_font_collection_create(minikin_families_ptr,
                                                                        minikin_families.len()),
            }
        }
    }

    #[inline]
    pub(crate) fn as_minikin_font_collection(&self) -> *mut minikin_font_collection_t {
        self.minikin_font_collection
    }
}
