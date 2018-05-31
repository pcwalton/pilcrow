// pilcrow/src/font.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use euclid::Rect;
use libc::size_t;
use minikin_sys::{minikin_font_callbacks_t, minikin_font_create, minikin_font_destroy};
use minikin_sys::{minikin_font_get_callbacks, minikin_font_get_userdata, minikin_font_t};
use minikin_sys::{minikin_paint_get_size, minikin_paint_t, minikin_rect_t};
use platform::Font;
use std::fmt::Debug;
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

    fn into_minikin_font(this: Box<Self>) -> *mut minikin_font_t {
        unsafe {
            let unique_id = this.get_unique_id();
            let font_ptr = Box::into_raw(this) as *mut c_void;
            minikin_font_create(&MINIKIN_FONT_CALLBACKS, unique_id, font_ptr)
        }
    }

    unsafe fn from_minikin_font(font: *mut minikin_font_t) -> Option<*mut Self> {
        if minikin_font_get_callbacks(font) == &MINIKIN_FONT_CALLBACKS {
            Some(minikin_font_get_userdata(font) as *mut Self)
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

static MINIKIN_FONT_CALLBACKS: minikin_font_callbacks_t = minikin_font_callbacks_t {
    destroy: Some(destroy_callback),
    get_horizontal_advance: Some(get_horizontal_advance_callback),
    get_bounds: Some(get_bounds_callback),
    get_font_data: Some(get_font_data_callback),
    get_font_size: Some(get_font_size_callback),
    get_font_index: Some(get_font_index_callback),
};

unsafe extern fn destroy_callback(font: *mut minikin_font_t) {
    let _: Box<Arc<Font>> = mem::transmute(minikin_font_get_userdata(font));
}

unsafe extern fn get_horizontal_advance_callback(font: *const minikin_font_t,
                                                 glyph_id: u32,
                                                 paint: *const minikin_paint_t)
                                                 -> f32 {
    let font: &Arc<Font> = mem::transmute(minikin_font_get_userdata(font));
    font.get_horizontal_advance(glyph_id, &FontAttributes::from_minikin_paint(paint))
}

unsafe extern fn get_bounds_callback(font: *const minikin_font_t,
                                     out_bounds: *mut minikin_rect_t,
                                     glyph_id: u32,
                                     paint: *const minikin_paint_t) {
    let font: &Arc<Font> = mem::transmute(minikin_font_get_userdata(font));
    let rect = font.get_bounds(glyph_id, &FontAttributes::from_minikin_paint(paint));
    *out_bounds = minikin_rect_t {
        left: rect.origin.x,
        top: rect.origin.y,
        right: rect.max_x(),
        bottom: rect.max_y(),
    }
}

unsafe extern fn get_font_data_callback(font: *const minikin_font_t) -> *const c_void {
    let font: &Arc<Font> = mem::transmute(minikin_font_get_userdata(font));
    font.get_font_data().as_ptr() as *const c_void
}

unsafe extern fn get_font_size_callback(font: *const minikin_font_t) -> size_t {
    let font: &Arc<Font> = mem::transmute(minikin_font_get_userdata(font));
    font.get_font_data().len()
}

unsafe extern fn get_font_index_callback(font: *const minikin_font_t) -> i32 {
    let font: &Arc<Font> = mem::transmute(minikin_font_get_userdata(font));
    font.get_font_index()
}
