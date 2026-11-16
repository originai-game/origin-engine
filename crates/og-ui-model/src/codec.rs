//! Bytes of og-ui's saved state: og-core's little-endian bytes
//! ([`og_core::bytes`]) and the UI's types over them. og-ui and og-rive
//! write their own types with these.

use crate::{Affine, Rect};
use og_core::bytes::{ByteReader, ByteWriter};
use og_core::glam::Vec2;
use og_css::Srgba;

pub struct W<'a>(pub &'a mut Vec<u8>);

impl W<'_> {
    fn w(&mut self) -> ByteWriter<'_> {
        ByteWriter::new(self.0)
    }
    pub fn u8(&mut self, v: u8) {
        self.w().u8(v);
    }
    pub fn u32(&mut self, v: u32) {
        self.w().u32(v);
    }
    pub fn i32(&mut self, v: i32) {
        self.w().i32(v);
    }
    pub fn u64(&mut self, v: u64) {
        self.w().u64(v);
    }
    pub fn f32(&mut self, v: f32) {
        self.w().f32(v);
    }
    pub fn str(&mut self, s: &str) {
        self.w().str(s);
    }
    pub fn rect(&mut self, r: &Rect) {
        for v in [r.x, r.y, r.width, r.height] {
            self.f32(v);
        }
    }
    pub fn affine(&mut self, m: &Affine) {
        for v in [m.a, m.b, m.c, m.d, m.e, m.f, m.g, m.h, m.i] {
            self.f32(v);
        }
    }
    pub fn vec2(&mut self, v: Vec2) {
        self.f32(v.x);
        self.f32(v.y);
    }
    pub fn color4(&mut self, c: &[f32; 4]) {
        for v in c {
            self.f32(*v);
        }
    }
    pub fn srgba(&mut self, c: Option<Srgba>) {
        match c {
            None => self.u8(0),
            Some(c) => {
                self.u8(1);
                for v in c.rgb {
                    self.u8(v);
                }
                self.f32(c.alpha);
            }
        }
    }
}

pub struct R<'a> {
    /// The bytes, read in order.
    pub b: ByteReader<'a>,
    /// A UI state before version 19: a placement has no glyph id, and a
    /// shaped form is its text's character U+F0000 + glyph id.
    pub legacy_glyphs: bool,
    /// A UI state before version 22: a map has no projective row.
    pub affine_only: bool,
}
impl<'a> R<'a> {
    /// A reader of `bytes` as this version writes them.
    pub fn new(bytes: &'a [u8]) -> R<'a> {
        R {
            b: ByteReader::new(bytes, "og-ui bytes"),
            legacy_glyphs: false,
            affine_only: false,
        }
    }
    pub fn u8(&mut self) -> Option<u8> {
        self.b.u8().ok()
    }
    pub fn u32(&mut self) -> Option<u32> {
        self.b.u32().ok()
    }
    pub fn i32(&mut self) -> Option<i32> {
        self.b.i32().ok()
    }
    pub fn u64(&mut self) -> Option<u64> {
        self.b.u64().ok()
    }
    pub fn f32(&mut self) -> Option<f32> {
        self.b.f32().ok()
    }
    pub fn str(&mut self) -> Option<String> {
        self.b.string().ok()
    }
    /// A count of items at least `min_item` bytes each, `None` when the
    /// bytes left could not hold them.
    pub fn count(&mut self, min_item: usize) -> Option<usize> {
        self.b.count(min_item).ok()
    }
    pub fn rect(&mut self) -> Option<Rect> {
        Some(Rect::new(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        ))
    }
    pub fn affine(&mut self) -> Option<Affine> {
        let m = Affine::new(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        );
        if self.affine_only {
            return Some(m);
        }
        Some(Affine {
            g: self.f32()?,
            h: self.f32()?,
            i: self.f32()?,
            ..m
        })
    }
    pub fn vec2(&mut self) -> Option<Vec2> {
        Some(Vec2::new(self.f32()?, self.f32()?))
    }
    pub fn color4(&mut self) -> Option<[f32; 4]> {
        Some([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }
    pub fn srgba(&mut self) -> Option<Option<Srgba>> {
        Some(match self.u8()? {
            0 => None,
            _ => {
                let rgb = [self.u8()?, self.u8()?, self.u8()?];
                Some(Srgba {
                    rgb,
                    alpha: self.f32()?,
                    p3: None,
                })
            }
        })
    }
}
