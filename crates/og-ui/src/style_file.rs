//! Shared internal paint and type-ramp values. Styles are authored only in CSS.
use crate::motion::TransitionKind;
use crate::style::{Palette, Srgba, TypeRamp, TypeStep};
use crate::tokens::SoundEvent;
use og_core::ogt::{Entry, Value};
use og_render::TextureHandle;

mod art;
pub use art::*;

pub fn palette_color<'a>(p: &'a mut Palette, name: &str) -> Option<&'a mut Srgba> {
    p.get_mut(name)
}

pub const STEP_NAMES: [&str; 22] = [
    "body",
    "xs",
    "meta",
    "caption",
    "sm",
    "reading",
    "base",
    "lg",
    "xl",
    "xl2",
    "xl3",
    "xl4",
    "xl5",
    "xl6",
    "xl7",
    "xl8",
    "displaySm",
    "displayMd",
    "displayLg",
    "displayXl",
    "displayXl2",
    "hero",
];

pub(crate) fn fit_steps(ramp: &TypeRamp, from: TypeStep, min: TypeStep) -> Vec<TypeStep> {
    let mut r = *ramp;
    let mut out: Vec<TypeStep> = STEP_NAMES
        .iter()
        .filter_map(|n| ramp_step(&mut r, n).copied())
        .filter(|s| s.size < from.size && s.size >= min.size.min(from.size))
        .map(|s| TypeStep {
            size: s.size,
            line_height: s.line_height,
            cjk_line_height: s.cjk_line_height,
            ..from
        })
        .collect();
    out.sort_by(|a, b| b.size.total_cmp(&a.size));
    out.dedup_by(|a, b| a.size == b.size);
    out.insert(0, from);
    out
}

pub fn ramp_step<'a>(r: &'a mut TypeRamp, name: &str) -> Option<&'a mut TypeStep> {
    Some(match name {
        "body" => &mut r.body,
        "xs" => &mut r.xs,
        "meta" => &mut r.meta,
        "caption" => &mut r.caption,
        "sm" => &mut r.sm,
        "reading" => &mut r.reading,
        "base" => &mut r.base,
        "lg" => &mut r.lg,
        "xl" => &mut r.xl,
        "xl2" => &mut r.xl2,
        "xl3" => &mut r.xl3,
        "xl4" => &mut r.xl4,
        "xl5" => &mut r.xl5,
        "xl6" => &mut r.xl6,
        "xl7" => &mut r.xl7,
        "xl8" => &mut r.xl8,
        "displaySm" => &mut r.display_sm,
        "displayMd" => &mut r.display_md,
        "displayLg" => &mut r.display_lg,
        "displayXl" => &mut r.display_xl,
        "displayXl2" | "hero" => &mut r.display_xl2,
        _ => return None,
    })
}

/// A CSS hex colour: `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`.
pub fn parse_color(s: &str) -> Option<Srgba> {
    let h = s.strip_prefix('#')?;
    let d: Vec<u8> = h
        .chars()
        .map(|c| c.to_digit(16).map(|v| v as u8))
        .collect::<Option<_>>()?;
    let v: Vec<u8> = match d.len() {
        3 | 4 => d.iter().map(|x| x * 17).collect(),
        6 | 8 => d.chunks(2).map(|p| p[0] * 16 + p[1]).collect(),
        _ => return None,
    };
    Some(Srgba {
        rgb: [v[0], v[1], v[2]],
        alpha: v.get(3).map_or(1.0, |&a| f32::from(a) / 255.0),
    })
}

/// Typed entry access used by the paint-value grammar, not a style-file parser.
pub(crate) struct P<'a> {
    pub(crate) e: &'a Entry,
}

impl<'a> P<'a> {
    pub(crate) fn err(&self, what: &str) -> String {
        format!("line {}: `{}` {what}", self.e.line, self.e.key)
    }
    pub(crate) fn num(&self, i: usize) -> Result<f32, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_number)
            .map(|n| n.as_f32())
            .filter(|v| v.is_finite())
            .ok_or_else(|| self.err(&format!("takes a number at position {}", i + 1)))
    }
    pub(crate) fn word(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_word)
            .ok_or_else(|| self.err(&format!("takes a word at position {}", i + 1)))
    }
    pub(crate) fn string(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_str)
            .ok_or_else(|| self.err(&format!("takes a quoted string at position {}", i + 1)))
    }
}
