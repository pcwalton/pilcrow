// pilcrow/src/font.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use font_traits::FontTraits;
use platform::Font;

use euclid::Rect;
use libc::size_t;
use minikin_sys::{minikin_font_create, minikin_font_destroy};
use minikin_sys::{minikin_font_get_typeface, minikin_font_t};
use minikin_sys::{minikin_paint_get_size, minikin_paint_t, minikin_rect_t};
use minikin_sys::{minikin_typeface_callbacks_t, minikin_typeface_create, minikin_typeface_destroy};
use minikin_sys::{minikin_typeface_get_callbacks};
use minikin_sys::{minikin_typeface_get_userdata, minikin_typeface_t};
use std::fmt::Debug;
use std::io::{self, Read};
use std::mem;
use std::os::raw::c_void;
use std::sync::Arc;

pub trait FontLike : Clone + Debug {
    type NativeFont;

    fn from_bytes(bytes: Arc<Vec<u8>>) -> Result<Self, ()>;
    fn from_native_font(native_font: Self::NativeFont) -> Self;

    fn get_unique_id(&self) -> u32;
    fn get_horizontal_advance(&self, glyph_id: u32, attributes: &FontAttributes) -> f32;
    fn get_bounds(&self, glyph_id: u32, attributes: &FontAttributes) -> Rect<f32>;
    fn get_font_data(&self) -> &[u8];
    fn get_font_index(&self) -> i32;

    fn get_font_traits(&self) -> FontTraits;

    /// A convenience method to create a font from a `File` or other readable object.
    fn from_reader<R>(mut reader: R) -> Result<Self, ()> where R: Read {
        let mut font_data = vec![];
        try!(reader.read_to_end(&mut font_data).map_err(drop));
        FontLike::from_bytes(Arc::new(font_data))
    }

    #[doc(hidden)]
    fn into_minikin_font(this: Box<Self>) -> *mut minikin_font_t {
        unsafe {
            let (unique_id, font_traits) = (this.get_unique_id(), this.get_font_traits());
            let typeface_ptr = Box::into_raw(this) as *mut c_void;
            let typeface = minikin_typeface_create(&MINIKIN_TYPEFACE_CALLBACKS,
                                                   unique_id,
                                                   typeface_ptr);
            minikin_font_create(typeface, font_traits.minikin_font_style())
        }
    }

    #[doc(hidden)]
    unsafe fn from_minikin_typeface(typeface: *mut minikin_typeface_t) -> Option<*mut Self> {
        if minikin_typeface_get_callbacks(typeface) == &MINIKIN_TYPEFACE_CALLBACKS {
            Some(minikin_typeface_get_userdata(typeface) as *mut Self)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FontAttributes {
    pub size: f32,
}

impl FontAttributes {
    unsafe fn from_minikin_paint(paint: *const minikin_paint_t) -> FontAttributes {
        FontAttributes {
            size: minikin_paint_get_size(paint),
        }
    }
}

static MINIKIN_TYPEFACE_CALLBACKS: minikin_typeface_callbacks_t = minikin_typeface_callbacks_t {
    destroy: Some(destroy_callback),
    get_horizontal_advance: Some(get_horizontal_advance_callback),
    get_bounds: Some(get_bounds_callback),
    get_font_data: Some(get_font_data_callback),
    get_font_data_length: Some(get_font_data_length_callback),
    get_font_index: Some(get_font_index_callback),
};

unsafe extern fn destroy_callback(typeface: *mut minikin_typeface_t) {
    let _: Box<Font> = mem::transmute(minikin_typeface_get_userdata(typeface));
}

unsafe extern fn get_horizontal_advance_callback(typeface: *const minikin_typeface_t,
                                                 glyph_id: u32,
                                                 paint: *const minikin_paint_t)
                                                 -> f32 {
    let typeface: &Font = mem::transmute(minikin_typeface_get_userdata(typeface));
    typeface.get_horizontal_advance(glyph_id, &FontAttributes::from_minikin_paint(paint))
}

unsafe extern fn get_bounds_callback(typeface: *const minikin_typeface_t,
                                     out_bounds: *mut minikin_rect_t,
                                     glyph_id: u32,
                                     paint: *const minikin_paint_t) {
    let typeface: &Font = mem::transmute(minikin_typeface_get_userdata(typeface));
    let rect = typeface.get_bounds(glyph_id, &FontAttributes::from_minikin_paint(paint));
    *out_bounds = minikin_rect_t {
        left: rect.origin.x,
        top: rect.origin.y,
        right: rect.max_x(),
        bottom: rect.max_y(),
    }
}

unsafe extern fn get_font_data_callback(typeface: *const minikin_typeface_t) -> *const c_void {
    let typeface: &Font = mem::transmute(minikin_typeface_get_userdata(typeface));
    typeface.get_font_data().as_ptr() as *const c_void
}

unsafe extern fn get_font_data_length_callback(typeface: *const minikin_typeface_t) -> size_t {
    let typeface: &Font = mem::transmute(minikin_typeface_get_userdata(typeface));
    typeface.get_font_data().len()
}

unsafe extern fn get_font_index_callback(typeface: *const minikin_typeface_t) -> i32 {
    let typeface: &Font = mem::transmute(minikin_typeface_get_userdata(typeface));
    typeface.get_font_index()
}
