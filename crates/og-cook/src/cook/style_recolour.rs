//! A style's painted art recoloured to its palette (`recolour "<from>"
//! <role>`, docs/art.md §2.7): the team-colour mask of strategy games
//! (Warcraft III's, StarCraft's) and the hue replacement of image editors
//! (Photoshop's Replace Color), taken from the art itself instead of a
//! painted mask. The mask is the pixels whose Oklab hue is near the
//! accent's and whose chroma is a good part of it; each is turned by the
//! hue between the accent as painted and the role's colour now, its chroma
//! scaled and its lightness shifted by theirs, so an edge darker than the
//! fill stays darker, and the grey and black of the art stay as they are.
//! Oklab is Björn Ottosson's (public domain), through the cook's
//! conformance colour module.
//!
//! The cook writes the recoloured atlas beside the style, one texture per
//! density the style's `atlas` line names (`<style>#atlas.png`,
//! `<style>#atlas@2x.png` …), points the cooked style at it and measures
//! the recoloured frames' `tone` again, so a label still picks ink that
//! reads on its button. A palette that still has every accent as painted
//! cooks nothing extra: the style draws its own atlas, byte for byte.

use super::Source;
use super::conform::texture2d::colour::{ONE, lab_to_linear, linear_to_lab, to_linear, to_srgb8};
use image::RgbaImage;
use og_ui::style_file::{RecolourRegion, dense_name};
use std::collections::BTreeSet;

/// The full mask within this hue of the accent, none past [`HUE_OUT`] (degrees).
const HUE_IN: f64 = 30.0;
const HUE_OUT: f64 = 50.0;
/// The mask from none to full between these fractions of the accent's chroma.
const CHROMA_IN: f64 = 0.2;
const CHROMA_FULL: f64 = 0.45;
/// An accent with less chroma than this has no hue to key on.
const GREY: f64 = 0.02;

/// The id of a style's recoloured atlas at `density`.
pub(super) fn atlas_id(style: &str, density: u32) -> String {
    let base = format!("{style}#atlas.png");
    if density == 1 {
        base
    } else {
        dense_name(&base, density)
    }
}

/// The style `s` parsed with its references resolved, when it parses.
pub(super) fn parsed(s: &Source, known: &BTreeSet<String>) -> Option<og_ui::StyleFile> {
    let mut f = og_ui::parse_style(std::str::from_utf8(&s.bytes).ok()?).ok()?;
    f.resolve(&s.rel, |id| known.contains(crate::runtime::asset_id(id)));
    Some(f)
}

/// The atlases a style recolours, by density, with each one's source id: none
/// when its palette has every accent as painted, or its atlas is a built-in
/// pack's (which draws as painted).
pub(super) fn atlases(s: &Source, known: &BTreeSet<String>) -> Vec<(u32, String)> {
    let Some(f) = parsed(s, known) else {
        return Vec::new();
    };
    let Some(base) = f.atlas.clone().filter(|a| !a.starts_with('@')) else {
        return Vec::new();
    };
    if f.recolour_regions().is_empty() {
        return Vec::new();
    }
    let mut out = vec![(1, base)];
    for (k, d) in f.atlas_dense.iter().enumerate() {
        if let Some(d) = d {
            out.push((k as u32 + 2, d.clone()));
        }
    }
    out.retain(|(_, id)| known.contains(crate::runtime::asset_id(id)));
    out
}

/// The cooked style's text when it recolours: its `atlas` line naming the
/// recoloured atlas beside it, and a `tone` for each recoloured frame,
/// measured from `atlas` (the 1× copy, recoloured).
pub(super) fn cooked_text(
    text: &str,
    style: &str,
    file: &og_ui::StyleFile,
    atlas: &RgbaImage,
) -> String {
    let name = style.rsplit('/').next().unwrap_or(style);
    let mut out = String::with_capacity(text.len() + 256);
    let mut done = false;
    for line in text.lines() {
        match line
            .strip_prefix("atlas \"")
            .and_then(|r| r.split_once('"'))
        {
            Some((_, rest)) if !done => {
                out.push_str(&format!("atlas \"{name}#atlas.png\"{rest}"));
                done = true;
            }
            _ => out.push_str(line),
        }
        out.push('\n');
    }
    out.push_str("# Recoloured by og cook to the palette (recolour): each frame's tone again.\n");
    for r in file.recolour_regions() {
        let Some(sel) = &r.dressing else { continue };
        let Some(sl) = file
            .dressings
            .iter()
            .find(|d| &d.selector == sel)
            .and_then(|d| d.frame.slice)
        else {
            continue;
        };
        let t = super::execute::slice_tone(atlas, &sl);
        out.push_str(&format!(
            "dress \"{sel}\" {{\n  tone {:.3} {:.3} {:.3} {:.2}\n}}\n",
            t[0], t[1], t[2], t[3]
        ));
    }
    out
}

/// Oklab as floats.
type Lab = [f64; 3];

fn lab_of(c: [u8; 3]) -> Lab {
    let l = linear_to_lab([to_linear(c[0]), to_linear(c[1]), to_linear(c[2])]);
    l.map(|v| v as f64 / ONE)
}

fn srgb8_of(l: Lab) -> Option<[u8; 3]> {
    let lin = lab_to_linear(l.map(|v| (v * ONE).round() as i32));
    lin.iter()
        .all(|v| (-1e-4..=1.0 + 1e-4).contains(v))
        .then(|| lin.map(to_srgb8))
}

/// `l` in gamut: its chroma reduced (hue and lightness kept) until it is.
fn in_gamut(l: Lab) -> [u8; 3] {
    if let Some(c) = srgb8_of(l) {
        return c;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..24 {
        let k = (lo + hi) / 2.0;
        if srgb8_of([l[0], l[1] * k, l[2] * k]).is_some() {
            lo = k;
        } else {
            hi = k;
        }
    }
    let l = [l[0].clamp(0.0, 1.0), l[1] * lo, l[2] * lo];
    srgb8_of(l).unwrap_or_else(|| lab_to_linear(l.map(|v| (v * ONE).round() as i32)).map(to_srgb8))
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// One accent's move: the painted colour's Oklab, chroma and hue, and the
/// turn, chroma scale and lightness shift to the role's colour.
struct Move {
    hue: f64,
    chroma: f64,
    turn: (f64, f64),
    scale: f64,
    shift: f64,
}

impl Move {
    fn new(from: og_ui::Srgba, to: og_ui::Srgba) -> Option<Move> {
        let (f, t) = (lab_of(from.rgb), lab_of(to.rgb));
        let cf = f[1].hypot(f[2]);
        if cf < GREY {
            return None;
        }
        let ct = t[1].hypot(t[2]);
        let (hf, ht) = (f[2].atan2(f[1]), t[2].atan2(t[1]));
        let d = if ct < GREY { 0.0 } else { ht - hf };
        Some(Move {
            hue: hf,
            chroma: cf,
            turn: (d.cos(), d.sin()),
            scale: ct / cf,
            shift: t[0] - f[0],
        })
    }

    /// How much of the accent pixel `l` is (0–1).
    fn weight(&self, l: Lab) -> f64 {
        let c = l[1].hypot(l[2]);
        let mut d = (l[2].atan2(l[1]) - self.hue).abs().to_degrees() % 360.0;
        if d > 180.0 {
            d = 360.0 - d;
        }
        (1.0 - smoothstep(HUE_IN, HUE_OUT, d))
            * smoothstep(CHROMA_IN * self.chroma, CHROMA_FULL * self.chroma, c)
    }

    fn apply(&self, l: Lab) -> Lab {
        let (cos, sin) = self.turn;
        let (a, b) = (l[1] * self.scale, l[2] * self.scale);
        [l[0] + self.shift, a * cos - b * sin, a * sin + b * cos]
    }
}

/// `img` (an atlas at `density` texels per CSS px) with each region's
/// accents moved to the palette's, every region grown by the CSS px the
/// packer extrudes its pieces into, so a filtered edge takes the new colour
/// too. Fully transparent texels are left alone.
pub fn recolour(img: &RgbaImage, regions: &[RecolourRegion], density: u32) -> RgbaImage {
    let mut out = img.clone();
    let (w, h) = (img.width() as f32, img.height() as f32);
    let grow = density as f32;
    let mut done = vec![false; (img.width() * img.height()) as usize];
    for r in regions {
        let moves: Vec<Move> = r
            .map
            .iter()
            .filter_map(|(f, t)| Move::new(*f, *t))
            .collect();
        if moves.is_empty() {
            continue;
        }
        let x0 = (r.uv[0] * w - grow).floor().max(0.0) as u32;
        let y0 = (r.uv[1] * h - grow).floor().max(0.0) as u32;
        let x1 = ((r.uv[2] * w + grow).ceil() as u32).min(img.width());
        let y1 = ((r.uv[3] * h + grow).ceil() as u32).min(img.height());
        for y in y0..y1 {
            for x in x0..x1 {
                let i = (y * img.width() + x) as usize;
                let p = img.get_pixel(x, y).0;
                if done[i] || p[3] == 0 {
                    continue;
                }
                done[i] = true;
                let l = lab_of([p[0], p[1], p[2]]);
                let Some((m, k)) = moves
                    .iter()
                    .map(|m| (m, m.weight(l)))
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .filter(|(_, k)| *k > 0.0)
                else {
                    continue;
                };
                let n = m.apply(l);
                let mixed = [0, 1, 2].map(|j| l[j] + (n[j] - l[j]) * k);
                let c = in_gamut(mixed);
                out.put_pixel(x, y, image::Rgba([c[0], c[1], c[2], p[3]]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_ui::Srgba;

    fn rgb(hex: u32) -> Srgba {
        Srgba::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
    }

    fn region(map: Vec<(Srgba, Srgba)>) -> RecolourRegion {
        RecolourRegion {
            uv: [0.0, 0.0, 1.0, 1.0],
            map,
            dressing: None,
        }
    }

    /// Cyber's yellow (#f5e60c) to a blue primary (#3d7eff): the yellow fill
    /// becomes the blue itself, its darker yellow edge (#c9bc00) a blue
    /// still darker than the fill, and the near-black body, the grey line
    /// and a red (another hue) stay byte for byte.
    ///
    /// Ways it could fail: the hue turned but the lightness kept (a pale
    /// blue fill), the edge's lightness set to the target's (the edge lost),
    /// the mask taking low-chroma pixels (the body tinted blue), or any hue
    /// within reach taken (the red turned).
    #[test]
    fn an_accent_follows_its_role_and_nothing_else_moves() {
        let px = [
            [0xf5, 0xe6, 0x0c, 255],
            [0xc9, 0xbc, 0x00, 255],
            [0x0d, 0x0e, 0x10, 255],
            [0x4a, 0x49, 0x42, 255],
            [0xff, 0x40, 0x40, 255],
        ];
        let mut img = RgbaImage::new(px.len() as u32, 1);
        for (x, p) in px.iter().enumerate() {
            img.put_pixel(x as u32, 0, image::Rgba(*p));
        }
        let out = recolour(&img, &[region(vec![(rgb(0xf5e60c), rgb(0x3d7eff))])], 1);
        let fill = out.get_pixel(0, 0).0;
        let edge = out.get_pixel(1, 0).0;
        let near = |a: [u8; 4], b: [u8; 3]| (0..3).all(|k| (a[k] as i32 - b[k] as i32).abs() <= 2);
        assert!(near(fill, [0x3d, 0x7e, 0xff]), "fill {fill:?}");
        assert!(
            edge[2] > edge[0] + 60 && edge[2] > edge[1],
            "edge not blue {edge:?}"
        );
        let l = |p: [u8; 4]| lab_of([p[0], p[1], p[2]])[0];
        assert!(
            l(edge) < l(fill) - 0.05,
            "edge {edge:?} not darker than fill {fill:?}"
        );
        for x in 2..5 {
            assert_eq!(out.get_pixel(x, 0).0, px[x as usize], "pixel {x} moved");
        }
    }

    /// The region alone moves, grown by the px its pieces were extruded
    /// into at the density, and the alpha is kept.
    #[test]
    fn only_the_region_and_its_extrusion_move() {
        let img = RgbaImage::from_pixel(12, 4, image::Rgba([0xf5, 0xe6, 0x0c, 128]));
        let r = RecolourRegion {
            uv: [4.0 / 12.0, 0.0, 8.0 / 12.0, 1.0],
            ..region(vec![(rgb(0xf5e60c), rgb(0xff3040))])
        };
        let out = recolour(&img, &[r], 2);
        for x in 0..12 {
            let p = out.get_pixel(x, 0).0;
            let moved = p[1] < 0x80;
            assert_eq!(moved, (2..10).contains(&x), "texel {x}: {p:?}");
            assert_eq!(p[3], 128);
        }
    }
}
