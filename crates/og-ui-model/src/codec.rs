//! Bytes of og-ui's saved state: little endian, counted lists. The
//! primitives; og-ui and og-rive write their own types with them.

use crate::{Affine, Rect};
use og_core::glam::Vec2;
use og_css::Srgba;

pub struct W<'a>(pub &'a mut Vec<u8>);

impl W<'_> {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
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
    pub bytes: &'a [u8],
    pub at: usize,
    /// A UI state before version 19: a placement has no glyph id, and a
    /// shaped form is its text's character U+F0000 + glyph id.
    pub legacy_glyphs: bool,
    /// A UI state before version 22: a map has no projective row.
    pub affine_only: bool,
}
impl R<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let s = self.bytes.get(self.at..self.at + N)?;
        self.at += N;
        s.try_into().ok()
    }
    pub fn u8(&mut self) -> Option<u8> {
        Some(self.take::<1>()?[0])
    }
    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take()?))
    }
    pub fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take()?))
    }
    pub fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take()?))
    }
    pub fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take()?))
    }
    pub fn str(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        let s = self.bytes.get(self.at..self.at + n)?;
        self.at += n;
        String::from_utf8(s.to_vec()).ok()
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
                })
            }
        })
    }
}
