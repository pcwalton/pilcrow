// pilcrow/src/format.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use cocoa::base::id;
use cocoa::foundation::NSUInteger;
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef, CFMutableDictionary};
use core_foundation::number::{CFNumber, CFNumberRef};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::CFURL;
use core_graphics::base::CGFloat;
use core_graphics::font::CGFont;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_text::font as ct_font;
use core_text::font::{CTFont, CTFontRef};
use core_text::font_descriptor::{kCTFontBoldTrait, kCTFontItalicTrait};
use core_text::run::{CTRunDelegate, ICTRunDelegate};
use euclid::{Point2D, Size2D};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::collections::HashMap;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str::FromStr;
use std::sync::{self, Once, RwLock};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSColor, NSImage};
#[cfg(target_os = "ios")]
use cocoa::uikit::UIColor;
#[cfg(target_word_size = "32")]
use std::f32;
#[cfg(target_word_size = "64")]
use std::f64;


use {LAYOUT_CALLBACKS, LayoutCallbacks};

static IMAGE_ATTACHMENT_CLASSES_REGISTER: Once = sync::ONCE_INIT;

type NativeAttributeDictionary = CFMutableDictionary<CFString, CFType>;

pub enum Format {
    Font(Font),
    Color(Color),
    Link(u32, String),
    Image(u32),
}

impl Format {
    #[inline]
    pub fn from_font(font: Font) -> Format {
        Format::Font(font)
    }

    #[inline]
    pub fn from_color(color: Color) -> Format {
        Format::Color(color)
    }

    #[inline]
    pub fn from_link(link_id: u32, url: String) -> Format {
        Format::Link(link_id, url)
    }

    #[inline]
    pub fn from_image(image_id: u32) -> Format {
        Format::Image(image_id)
    }

    pub fn font(&self) -> Option<Font> {
        if let Format::Font(ref font) = *self {
            Some((*font).clone())
        } else {
            None
        }
    }

    pub fn color(&self) -> Option<Color> {
        if let Format::Color(ref color) = *self {
            Some((*color).clone())
        } else {
            None
        }
    }

    #[inline]
    pub fn link(&self) -> Option<(u32, &str)> {
        if let Format::Link(link_id, ref url) = *self {
            Some((link_id, &**url))
        } else {
            None
        }
    }

    #[inline]
    pub fn image(&self) -> Option<u32> {
        if let Format::Image(image_id) = *self {
            Some(image_id)
        } else {
            None
        }
    }

    pub(crate) fn add_to_native_attributes(&self, dictionary: &mut NativeAttributeDictionary) {
        unsafe {
            match *self {
                Format::Font(ref font) => {
                    dictionary.set(CFString::from_static_string("NSFont"),
                                font.native_font.as_CFType())
                }
                Format::Color(ref color) => {
                    self.add_color_to_native_attributes(dictionary, color)
                }
                Format::Link(link_id, ref url) => {
                    let url = CFURL::from_cf_string(CFString::from_str(url).unwrap(), None);
                    let mut link_dictionary = CFDictionary::from_CFType_pairs(&[
                        (CFString::from_static_string("PCLinkID"),
                        CFNumber::from(link_id as i64).as_CFType()),
                        (CFString::from_static_string("PCLinkURL"), url.as_CFType()),
                    ]);
                    dictionary.set(CFString::from_static_string("NSLink"), url.as_CFType());
                    dictionary.set(CFString::from_static_string("PCLink"),
                                link_dictionary.as_CFType());
                }
                Format::Image(image_id) => {
                    let run_delegate = CTRunDelegate::new(Box::new(ImageAttachment {
                        id: image_id,
                    }));
                    let id_object = CFNumber::from(image_id as i64);
                    dictionary.set(CFString::from_static_string("CTRunDelegate"),
                                run_delegate.as_CFType());
                    dictionary.set(CFString::from_static_string("PCImage"), id_object.as_CFType());
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn add_color_to_native_attributes(&self,
                                      dictionary: &mut NativeAttributeDictionary,
                                      color: &Color) {
        unsafe {
            let color = NSColor::colorWithRedGreenBlueAlpha(ptr::null_mut(),
                                                            color.r_f32() as CGFloat,
                                                            color.g_f32() as CGFloat,
                                                            color.b_f32() as CGFloat,
                                                            color.a_f32() as CGFloat);
            dictionary.set(CFString::from_static_string("NSForegroundColor"),
                           mem::transmute::<id, CFType>(color))
        }
    }

    #[cfg(target_os = "ios")]
    fn add_color_to_native_attributes(&self,
                                      dictionary: &mut NativeAttributeDictionary,
                                      color: &Color) {
        unsafe {
            let color = UIColor::colorWithRedGreenBlueAlpha(ptr::null_mut(),
                                                            color.r_f32() as CGFloat,
                                                            color.g_f32() as CGFloat,
                                                            color.b_f32() as CGFloat,
                                                            color.a_f32() as CGFloat);
            dictionary.set(CFString::from_static_string("NSForegroundColor"),
                           mem::transmute::<id, CFType>(color))
        }
    }
}

#[derive(Clone)]
pub struct Font {
    native_font: CTFont,
}

impl Font {
    #[inline]
    pub fn from_native_font(native_font: CTFont) -> Font {
        Font {
            native_font
        }
    }

    pub fn default_serif() -> Font {
        Font::from_native_font(ct_font::new_from_name("Times", 16.0).unwrap())
    }

    pub fn default_monospace() -> Font {
        Font::from_native_font(ct_font::new_from_name("Menlo", 12.0).unwrap())
    }

    #[inline]
    pub fn id(&self) -> FontId {
        FontId::from_native_font(self.native_font.clone())
    }

    #[inline]
    pub fn face_id(&self) -> FontFaceId {
        FontFaceId::from_native_font(self.native_font.clone())
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.native_font.pt_size() as f32
    }

    #[inline]
    pub fn native_font(&self) -> CTFont {
        self.native_font.clone()
    }

    pub fn to_size(&self, new_size: f32) -> Font {
        Font::from_native_font(self.native_font.clone_with_font_size(new_size as f64))
    }

    pub fn to_bold(&self) -> Option<Font> {
        self.native_font
            .clone_with_symbolic_traits(kCTFontBoldTrait, kCTFontBoldTrait)
            .map(Font::from_native_font)
    }

    pub fn to_italic(&self) -> Option<Font> {
        self.native_font
            .clone_with_symbolic_traits(kCTFontItalicTrait, kCTFontItalicTrait)
            .map(Font::from_native_font)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FontFaceId(usize);

impl FontFaceId {
    fn from_native_font(font: CTFont) -> FontFaceId {
        unsafe {
            FontFaceId(mem::transmute::<CGFont, usize>(font.copy_to_CGFont()))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FontId(usize);

impl FontId {
    fn from_native_font(font: CTFont) -> FontId {
        unsafe {
            FontId(mem::transmute::<CTFont, usize>(font))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r,
            g,
            b,
            a,
        }
    }

    pub fn from_native_color(color: id) -> Color {
        let (mut r, mut g, mut b, mut a) = (0.0, 0.0, 0.0, 0.0);
        unsafe {
            color.getRedGreenBlueAlpha(&mut r, &mut g, &mut b, &mut a);
        }
        Color {
            r: round_CGFloat(r * 255.0) as u8,
            g: round_CGFloat(g * 255.0) as u8,
            b: round_CGFloat(b * 255.0) as u8,
            a: round_CGFloat(a * 255.0) as u8,
        }
    }

    #[inline]
    pub fn r_f32(&self) -> f32 {
        (self.r as f32) / 255.0
    }

    #[inline]
    pub fn g_f32(&self) -> f32 {
        (self.g as f32) / 255.0
    }

    #[inline]
    pub fn b_f32(&self) -> f32 {
        (self.b as f32) / 255.0
    }

    #[inline]
    pub fn a_f32(&self) -> f32 {
        (self.a as f32) / 255.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Image {
    pub id: u32,
    pub alt_text: String,
}

#[derive(Clone, PartialEq, Debug)]
struct ImageAttachment {
    id: u32,
}

impl ICTRunDelegate for ImageAttachment {
    fn width(&mut self) -> CGFloat {
        match *LAYOUT_CALLBACKS.read().unwrap() {
            Some(ref callbacks) => {
                callbacks.get_image_size(self.id).map(|size| size.width as CGFloat).unwrap_or(0.0)
            }
            None => 0.0,
        }
    }

    fn ascent(&mut self) -> CGFloat {
        match *LAYOUT_CALLBACKS.read().unwrap() {
            Some(ref callbacks) => {
                callbacks.get_image_size(self.id).map(|size| size.height as CGFloat).unwrap_or(0.0)
            }
            None => 0.0,
        }
    }

    fn descent(&mut self) -> CGFloat {
        0.0
    }
}

pub(crate) fn attributes_to_formatting(attributes: &CFDictionary<CFString, CFType>)
                                       -> Vec<Format> {
    let mut formatting = vec![];
    let (attribute_keys, attribute_values) = attributes.get_keys_and_values();
    for (key, value) in attribute_keys.into_iter().zip(attribute_values.into_iter()) {
        unsafe {
            let key = mem::transmute::<*const c_void, CFStringRef>(key);
            let key = CFString::wrap_under_get_rule(key);
            if key == CFString::from_static_string("NSFont") {
                let font = mem::transmute::<*const c_void, CTFontRef>(value);
                let font = Font::from_native_font(CTFont::wrap_under_get_rule(font));
                formatting.push(Format::Font(font))
            } else if key == CFString::from_static_string("NSForegroundColor") {
                let color = mem::transmute::<*const c_void, id>(value);
                formatting.push(Format::Color(Color::from_native_color(color.clone())));
                mem::forget(color);
            } else if key == CFString::from_static_string("PCLink") {
                let info = mem::transmute::<*const c_void, CFDictionaryRef>(value);
                let info: CFDictionary<CFString, CFType> = CFDictionary::wrap_under_get_rule(info);
                let id_key = CFString::from_static_string("PCLinkID");
                let url_key = CFString::from_static_string("PCLinkURL");
                let id = mem::transmute::<CFType, CFNumber>((*info.get(id_key)).clone());
                let url = mem::transmute::<CFType, CFURL>((*info.get(url_key)).clone());
                let url = url.get_string().to_string();
                formatting.push(Format::Link(id.to_i64().unwrap() as u32, url))
            } else if key == CFString::from_static_string("PCImage") {
                let id = mem::transmute::<*const c_void, CFNumberRef>(value);
                let id = CFNumber::wrap_under_get_rule(id);
                formatting.push(Format::Image(id.to_i64().unwrap() as u32))
            }
        }
    }
    formatting
}

#[cfg(target_pointer_width = "32")]
fn round_CGFloat(n: CGFloat) -> CGFloat {
    f32::round(n)
}

#[cfg(target_pointer_width = "64")]
fn round_CGFloat(n: CGFloat) -> CGFloat {
    f64::round(n)
}
