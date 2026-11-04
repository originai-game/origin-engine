//! The 3D rows of docs/art.md §2.3: models, the textures they use, and
//! animation clips and flipbooks, made to meet the project's art contract.
//!
//! [`Conform::of`] reads the contract (`look::contract`'s blocks) once per
//! cook; `None` when it sets nothing this module reads, so a project
//! without a look, or whose look leaves ArtShape, ArtTexels, ArtLine and
//! ArtMotion at zero, cooks exactly as before (no stamp moves). Otherwise
//! each conformed output's stamp carries [`Conform::model_desc`],
//! [`Conform::texture_desc`] or [`Motion::describe`], so a look edit
//! recooks what it changes and nothing else.
//!
//! A model (glTF, `type model`, `.vox`, each a glTF here) is conformed per
//! primitive inside `model::cook_primitive`, before welding, so the model
//! blob and its `#anim` (which cooks the same primitives) agree vertex for
//! vertex. In order:
//!
//! 1. **Voxelise** at `ArtShape.voxel` (metres, the model's units): the
//!    surface, placed by the first node that draws it, sampled into one
//!    grid aligned to the model's origin (so turned parts step), the
//!    inside filled, and the grid meshed by the `.vox` importer's greedy
//!    mesher (`vox::mesh_model`) and brought back under the node; each cell's colour is the surface's
//!    (base colour factor × texture × vertex colour), mapped to
//!    `ArtPalette.colors` (Oklab nearest) or cut to `maxColors`. Past the
//!    face budget the edge doubles (at most four times). Skinned or morphed
//!    primitives are not voxelised (a voxel has no source vertex to skin).
//! 2. **Decimate** to the budget (`propFaces`, or `characterFaces` for a
//!    model with a skin or that says `conform character`), counted over
//!    the model's drawn triangles (each primitive times the nodes that
//!    draw it), never destroying it. A `type model` past it was built
//!    again at fewer segments first (`model_text::to_glb_within`), so
//!    only what its floors leave over is decimated here:
//!    meshoptimizer's edge collapses with the topology kept (no sloppy
//!    simplifier, open borders and so material seams locked, uv seams
//!    held), the surface moved at most `ArtShape.silhouette`, else
//!    [`SHAPE_ERROR`] of the model's bounding radius. Each primitive goes as
//!    far as that bound lets it, and the budget's remainder is shared in
//!    proportion to what each has left, so the model lands at the budget
//!    ([`Plan`]). When even that is past the budget, every primitive stops
//!    at its bound and the cook warns that the model is over budget.
//! 3. **Bevel** at `ArtShape.bevel`: every hard edge (faces meeting past
//!    30°, or `normalAngle` with `normals angle`) chamfered: the faces
//!    inset by the radius from their hard edges, a quad bridging each hard
//!    edge and a fan closing each corner where three or more meet. The
//!    radius is held to a quarter of the shortest hard edge.
//! 4. **Bake vertex colours** where the style is untextured (`normals
//!    flat` and no ArtTexels density, maxSize or filter, or voxels): the base colour
//!    texture times the factor sampled over each face (flat) or at each
//!    vertex, mapped to the palette or cut to `maxColors`; the model's
//!    materials then draw white with no base colour texture.
//! 5. **Normals**: flat (one per face), smooth (area-weighted over the
//!    corners that share a position and a uv, so uv seams stay), or angle
//!    (over the faces at a position within `normalAngle`).
//!
//! With `ArtLine.width` the model blob gains an `outline` section, the
//! hull's data an outline pass extrudes along: a header of eight 4-byte
//! words (`1` the layout's version, the width, the colour's linear r g b,
//! the place 0 outside / 1 inside / 2 both, the distance falloff, the
//! primitive count), then per primitive its vertex count (u32) and per
//! cooked vertex the area-weighted normal over every corner at its
//! position (two oct-encoded i16), so a flat-shaded mesh's hull does not
//! crack at its edges. Readers that do not know the section skip it.
//!
//! Textures a model uses: resampled (an area filter, in linear light, the
//! colour alpha-weighted, normals renormalised) down to `ArtTexels.density`
//! texels per metre over the surface it covers (the span its UVs give a
//! side, [`texel_span`]; rounded to a power of two) and `maxSize`; mapped to the palette where `ArtPalette.strict`;
//! `filter nearest` cooks a colour texture as pixel art (RGBA8, no mips,
//! nearest), `mips none` without mips.
//!
//! Clips (`#anim`): with `ArtMotion.stepFps` resampled at that rate as
//! STEP channels, so each pose holds for one step; past `maxFrames` the
//! rate is lowered until the clip holds that many, with a warning.
//! Flipbooks: each frame's duration held to whole steps (at least one),
//! and past `maxFrames` a clip keeps every k-th frame, the dropped
//! frames' time added to the one kept before them, with a warning.

use crate::cook::conform::texture2d::paint::Paint;
use crate::cook::model::{FVertex, GltfSource};
use crate::cook::texture::Image;
use crate::cook::vox::{VoxModel, mesh_model};
use crate::format::kind;
use crate::look::{LookBlock, LookFile, hex_to_linear};
use glam::Vec3;
use og_core::ogt::Value;
use std::collections::{BTreeMap, BTreeSet};

mod flat;
mod geometry;

pub use geometry::*;

/// How the contract shades models (`ArtShape.normals`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Normals {
    Any,
    Smooth,
    Flat,
    /// Split past this many degrees.
    Angle(f32),
}

/// `ArtTexels.filter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Any,
    Nearest,
    Linear,
}

/// `ArtTexels.mips`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mips {
    Any,
    Full,
    None,
}

/// `ArtLine`, for the outline hull.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub width: f32,
    pub color: [f32; 3],
    /// 0 outside, 1 inside, 2 both.
    pub place: u32,
    pub distance: f32,
}

/// `ArtMotion`, for clips and flipbooks.
#[derive(Clone, Debug, PartialEq)]
pub struct Motion {
    pub step_fps: f32,
    pub max_frames: u32,
}

/// The art contract as the 3D conformance reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct Conform {
    pub prop_faces: u32,
    pub character_faces: u32,
    /// `ArtShape.silhouette`: the thinnest feature a model keeps, in
    /// metres; the most decimation may move its surface (0: a fraction of
    /// the model's bounding radius, [`SHAPE_ERROR`]).
    pub silhouette: f32,
    pub normals: Normals,
    pub bevel: f32,
    pub voxel: f32,
    pub density: f32,
    pub max_size: u32,
    pub filter: Filter,
    pub mips: Mips,
    /// `ArtPalette.colors`, sRGB, in file order (empty: none).
    pub palette: Vec<[u8; 3]>,
    pub strict: bool,
    pub max_colors: u32,
    /// `ArtPalette.rampSteps`: shades per hue family on colour textures.
    pub ramp_steps: u32,
    pub line: Option<Line>,
    pub motion: Option<Motion>,
    /// The look's `Stylise` paint fields read as texel space, applied to
    /// the colour textures models use (painterly 3D: the stroke follows
    /// the surface); `None` when the look sets none (set by the cook from
    /// the flattened look, `texture2d::paint_of`).
    pub(in crate::cook) paint: Option<Paint>,
    /// The assets kept as made, by asset id, with why: their own `conform
    /// off` or project.ogt's `conform { off ... }` (set by the cook).
    pub(in crate::cook) off: BTreeMap<String, String>,
    /// The textures that are a mesh's though no model names them
    /// (`super::TextureRole::Mesh`, set by the cook).
    pub(in crate::cook) textures: BTreeSet<String>,
}

/// What a texture conformed as a mesh's by default says, before its
/// changes: why, and how a HUD or UI image opts out.
pub const DEFAULT_MESH_TEXTURE: &str = "conformed as a mesh texture (no model claims it; a HUD or UI image takes `conform ui` in its import settings)";

/// Moves painted mesh textures' stamps when the paint steps change.
const PAINT_VERSION: u32 = 2;

/// Moves flattened mesh textures' stamps when the palette limit's steps
/// change (`flatten`).
const FLAT_VERSION: u32 = 2;

/// What an output the contract leaves as it was reports.
pub const UNCHANGED: &str = "meets the contract as it is";

fn field<'a>(b: Option<&'a LookBlock>, f: &str) -> Option<&'a [Value]> {
    b.and_then(|b| b.get(f))
}

fn num(b: Option<&LookBlock>, f: &str) -> f64 {
    field(b, f)
        .and_then(|v| v.first())
        .and_then(|v| v.as_number())
        .map_or(0.0, |n| n.as_f64())
}

fn word<'a>(b: Option<&'a LookBlock>, f: &str) -> &'a str {
    field(b, f)
        .and_then(|v| v.first())
        .and_then(|v| v.as_word())
        .unwrap_or("")
}

/// Reads a palette asset by the id a look names: its sRGB colours.
pub type PaletteOf<'a> = &'a dyn Fn(&str) -> Result<Vec<[u8; 3]>, String>;

impl Conform {
    /// The contract of the flattened look `look` (its art blocks), the
    /// palette `ArtPalette.colors` names read through `palette` (a built-in
    /// `@palettes/<name>` or the game's own file). `None` when it constrains
    /// nothing this module conforms.
    pub fn of(look: &LookFile, palette: PaletteOf) -> Result<Option<Conform>, String> {
        let block = |r: &str| look.blocks.iter().find(|b| b.resource == r);
        let (shape, texels, pal, line, motion) = (
            block("ArtShape"),
            block("ArtTexels"),
            block("ArtPalette"),
            block("ArtLine"),
            block("ArtMotion"),
        );
        let normals = match word(shape, "normals") {
            "smooth" => Normals::Smooth,
            "flat" => Normals::Flat,
            "angle" => Normals::Angle(match field(shape, "normalAngle") {
                Some(_) => num(shape, "normalAngle") as f32,
                None => 30.0,
            }),
            _ => Normals::Any,
        };
        let colors = match field(pal, "colors").and_then(|v| v.first()) {
            Some(Value::String(id)) if !id.is_empty() => {
                palette(id).map_err(|e| format!("ArtPalette.colors \"{id}\": {e}"))?
            }
            _ => Vec::new(),
        };
        let width = num(line, "width") as f32;
        let line = (width > 0.0).then(|| Line {
            width,
            color: match field(line, "color") {
                Some([Value::String(h)]) => {
                    hex_to_linear(h).map_or([0.02, 0.02, 0.03], |c| c.map(|x| x as f32))
                }
                Some(v) if v.len() == 3 => {
                    [0, 1, 2].map(|i| v[i].as_number().map_or(0.0, |n| n.as_f64() as f32))
                }
                _ => [0.02, 0.02, 0.03],
            },
            place: match word(line, "place") {
                "inside" => 1,
                "both" => 2,
                _ => 0,
            },
            distance: num(line, "distanceFalloff") as f32,
        });
        let (step_fps, max_frames) = (
            num(motion, "stepFps") as f32,
            num(motion, "maxFrames") as u32,
        );
        let c = Conform {
            prop_faces: num(shape, "propFaces") as u32,
            character_faces: num(shape, "characterFaces") as u32,
            silhouette: num(shape, "silhouette") as f32,
            normals,
            bevel: num(shape, "bevel") as f32,
            voxel: num(shape, "voxel") as f32,
            density: num(texels, "density") as f32,
            max_size: num(texels, "maxSize") as u32,
            filter: match word(texels, "filter") {
                "nearest" => Filter::Nearest,
                "linear" => Filter::Linear,
                _ => Filter::Any,
            },
            mips: match word(texels, "mips") {
                "full" => Mips::Full,
                "none" => Mips::None,
                _ => Mips::Any,
            },
            palette: colors,
            strict: field(pal, "strict")
                .and_then(|v| v.first())
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            max_colors: num(pal, "maxColors") as u32,
            ramp_steps: num(pal, "rampSteps") as u32,
            line,
            motion: (step_fps > 0.0 || max_frames > 0).then_some(Motion {
                step_fps,
                max_frames,
            }),
            paint: None,
            off: BTreeMap::new(),
            textures: BTreeSet::new(),
        };
        Ok((c.geometry() || c.textures() || c.motion.is_some()).then_some(c))
    }

    /// Whether models change at all.
    fn geometry(&self) -> bool {
        self.prop_faces > 0
            || self.character_faces > 0
            || self.normals != Normals::Any
            || self.bevel > 0.0
            || self.voxel > 0.0
            || self.line.is_some()
    }

    /// Whether the textures models use change.
    fn textures(&self) -> bool {
        self.density > 0.0
            || self.max_size > 0
            || self.filter != Filter::Any
            || self.mips != Mips::Any
            || (self.strict && !self.palette.is_empty())
            || self.max_colors > 0
            || self.ramp_steps > 0
            || self.paint.is_some()
    }

    /// The style is untextured: models carry their colour in vertices.
    /// Flat shading with no ArtTexels constraint (a style that asks for a
    /// texel density, a size or a filter keeps its textures, as faceted
    /// low poly's 64-texel nearest ones), or voxels.
    pub fn bakes(&self) -> bool {
        self.voxel > 0.0
            || (self.normals == Normals::Flat
                && self.density <= 0.0
                && self.max_size == 0
                && self.filter == Filter::Any)
    }

    /// Whether this model is conformed: not one with authored levels of
    /// detail (`MSFT_lod`), which are kept as made.
    pub fn applies(&self, src: &GltfSource) -> bool {
        self.geometry()
            && !src
                .doc
                .nodes()
                .any(|n| n.extension_value("MSFT_lod").is_some())
    }

    fn palette_desc(&self) -> String {
        let p: Vec<String> = self
            .palette
            .iter()
            .map(|c| format!("{:02x}{:02x}{:02x}", c[0], c[1], c[2]))
            .collect();
        format!("palette [{}] maxColors {}", p.join(" "), self.max_colors)
    }

    /// What a model's blob (and its `#anim`'s vertex streams) depends on;
    /// empty when models do not change.
    pub fn model_desc(&self) -> String {
        if !self.geometry() {
            return String::new();
        }
        format!(
            "conform-mesh-v3 faces {} {} silhouette {} normals {:?} bevel {} voxel {} bakes {} {} line {:?}",
            self.prop_faces,
            self.character_faces,
            self.silhouette,
            self.normals,
            self.bevel,
            self.voxel,
            self.bakes(),
            if self.bakes() {
                self.palette_desc()
            } else {
                String::new()
            },
            self.line
        )
    }

    /// What a model's texture depends on besides its image; empty when
    /// textures do not change.
    pub fn texture_desc(&self) -> String {
        if !self.textures() {
            return String::new();
        }
        let paint = self.paint.as_ref().map_or(String::new(), |p| {
            let mut h = og_core::hash::StableHasher::new();
            if let Some((grain, w, gh)) = &p.canvas {
                h.write_str(&format!("{w}x{gh}"));
                for g in grain {
                    h.write(&g.to_bits().to_le_bytes());
                }
            }
            format!(
                " paint {} radius {} anisotropy {} edge {} bleed {} canvas {} scale {} {:016x}",
                PAINT_VERSION,
                p.radius,
                p.anisotropy,
                p.edge,
                p.bleed,
                p.canvas_strength,
                p.canvas_scale,
                h.finish()
            )
        });
        format!(
            "conform-texture-v3 density {} maxSize {} filter {:?} mips {:?} maxColors {} rampSteps {} flat {} strict {} {}{paint}",
            self.density,
            self.max_size,
            self.filter,
            self.mips,
            self.max_colors,
            self.ramp_steps,
            FLAT_VERSION,
            self.strict,
            if self.strict {
                self.palette_desc()
            } else {
                String::new()
            }
        )
    }

    /// The face budget of a model: a character's when it has a skin or
    /// says it is one (`conform character`).
    pub(in crate::cook) fn budget(&self, src: &GltfSource) -> (u32, &'static str) {
        if src.character || src.doc.skins().next().is_some() {
            (self.character_faces, "characterFaces")
        } else {
            (self.prop_faces, "propFaces")
        }
    }
}

impl Motion {
    pub fn describe(&self) -> String {
        format!(
            "conform-motion-v1 stepFps {} maxFrames {}",
            self.step_fps, self.max_frames
        )
    }

    /// A clip's resampling `rate` over `duration` seconds as the contract
    /// holds it: the rate, whether its channels step, and the notes.
    pub fn clip(&self, rate: f32, duration: f32, name: &str) -> (f32, bool, Vec<String>) {
        let mut notes = Vec::new();
        let (mut rate, step) = if self.step_fps > 0.0 {
            notes.push(format!(
                "conform: clip '{name}' stepped at {} fps (ArtMotion.stepFps), each pose held",
                self.step_fps
            ));
            (self.step_fps, true)
        } else {
            (rate, false)
        };
        let frames = og_anim::ClipAsset::frames_for(duration, rate);
        if self.max_frames > 0 && frames > self.max_frames && duration > 0.0 {
            rate = (self.max_frames.max(2) - 1) as f32 / duration;
            notes.push(format!(
                "conform: warning: clip '{name}' holds {frames} frames, past ArtMotion.maxFrames {}: resampled at {rate:.3} Hz ({} frames)",
                self.max_frames,
                og_anim::ClipAsset::frames_for(duration, rate)
            ));
        }
        (rate, step, notes)
    }

    /// Holds a flipbook's clips to the contract; the notes.
    pub fn flipbook(&self, sheet: &mut og_anim::FlipbookSheet) -> Vec<String> {
        let mut notes = Vec::new();
        for clip in &mut sheet.clips {
            let max = self.max_frames as usize;
            if max > 0 && clip.frames.len() > max {
                let n = clip.frames.len();
                let k = n.div_ceil(max);
                let mut frames = Vec::new();
                let mut durations = Vec::new();
                let mut events = Vec::new();
                for (g, start) in (0..n).step_by(k).enumerate() {
                    frames.push(clip.frames[start]);
                    durations.push(clip.durations[start..(start + k).min(n)].iter().sum());
                    for (f, e) in &clip.events {
                        if (*f as usize) >= start && (*f as usize) < start + k {
                            events.push((g as u32, e.clone()));
                        }
                    }
                }
                notes.push(format!(
                    "conform: warning: flipbook clip '{}' holds {n} frames, past ArtMotion.maxFrames {}: every {k} kept ({} frames)",
                    clip.name,
                    self.max_frames,
                    frames.len()
                ));
                clip.frames = frames;
                clip.durations = durations;
                clip.events = events;
            }
            if self.step_fps > 0.0 {
                let step = 1.0 / self.step_fps;
                let mut moved = 0;
                for d in &mut clip.durations {
                    let held = (*d * self.step_fps).round().max(1.0) * step;
                    if (held - *d).abs() > 1e-6 {
                        moved += 1;
                    }
                    *d = held;
                }
                notes.push(format!(
                    "conform: flipbook clip '{}' held to {} fps steps ({moved} of {} durations moved)",
                    clip.name,
                    self.step_fps,
                    clip.durations.len()
                ));
            }
        }
        notes
    }
}

// ── colour ────────────────────────────────────────────────────────────────

fn to_linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// Linear RGB to Oklab (Björn Ottosson's matrices).
fn oklab(c: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = c;
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// The palette colour nearest `c` (linear) in Oklab, as linear RGB.
fn nearest(palette: &[([f32; 3], [f32; 3])], c: [f32; 3]) -> [f32; 3] {
    let lab = oklab(c);
    let d = |p: &[f32; 3]| (0..3).map(|i| (p[i] - lab[i]).powi(2)).sum::<f32>();
    palette
        .iter()
        .min_by(|a, b| d(&a.1).total_cmp(&d(&b.1)))
        .map_or(c, |p| p.0)
}

fn palette_lab(p: &[[u8; 3]]) -> Vec<([f32; 3], [f32; 3])> {
    p.iter()
        .map(|c| {
            let l = c.map(to_linear);
            (l, oklab(l))
        })
        .collect()
}

/// Holds linear colours to the palette (Oklab nearest) or, without one, to
/// at most `max` distinct colours by a median cut over their sRGB bytes
/// (weighted by use; ties broken by order, so the cut is deterministic).
/// Alpha is untouched. Answers the distinct colours left.
fn reduce(colors: &mut [[f32; 4]], palette: &[[u8; 3]], max: u32) -> usize {
    let key = |c: &[f32; 4]| [to_srgb(c[0]), to_srgb(c[1]), to_srgb(c[2])];
    if !palette.is_empty() {
        let lab = palette_lab(palette);
        let mut cache: BTreeMap<[u8; 3], [f32; 3]> = BTreeMap::new();
        for c in colors.iter_mut() {
            let k = key(c);
            let m = *cache
                .entry(k)
                .or_insert_with(|| nearest(&lab, k.map(to_linear)));
            c[..3].copy_from_slice(&m);
        }
    } else if max > 0 {
        let mut counts: BTreeMap<[u8; 3], u32> = BTreeMap::new();
        for c in colors.iter() {
            *counts.entry(key(c)).or_default() += 1;
        }
        if counts.len() > max as usize {
            let mut boxes: Vec<Vec<([u8; 3], u32)>> = vec![counts.into_iter().collect()];
            while boxes.len() < max as usize {
                // The box with the widest channel, split at its weighted median.
                let range = |b: &[([u8; 3], u32)], ch: usize| {
                    let (lo, hi) = b
                        .iter()
                        .fold((255u8, 0u8), |(l, h), (k, _)| (l.min(k[ch]), h.max(k[ch])));
                    hi.saturating_sub(lo)
                };
                let Some((bi, ch, _)) = boxes
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.len() > 1)
                    .flat_map(|(i, b)| (0..3).map(move |ch| (i, ch, range(b, ch))))
                    .max_by(|a, b| a.2.cmp(&b.2).then(b.0.cmp(&a.0)).then(b.1.cmp(&a.1)))
                else {
                    break;
                };
                let mut b = boxes.swap_remove(bi);
                b.sort_by(|x, y| x.0[ch].cmp(&y.0[ch]).then(x.0.cmp(&y.0)));
                let total: u32 = b.iter().map(|e| e.1).sum();
                let (mut acc, mut at) = (0u32, 1usize);
                for (i, e) in b.iter().enumerate() {
                    acc += e.1;
                    if acc * 2 >= total {
                        at = (i + 1).clamp(1, b.len() - 1);
                        break;
                    }
                }
                let rest = b.split_off(at);
                boxes.push(b);
                boxes.push(rest);
                boxes.sort_by(|x, y| x[0].0.cmp(&y[0].0));
            }
            let mut to: BTreeMap<[u8; 3], [f32; 3]> = BTreeMap::new();
            for b in &boxes {
                let w: f32 = b.iter().map(|e| e.1 as f32).sum();
                let mut m = [0f32; 3];
                for (k, n) in b {
                    for i in 0..3 {
                        m[i] += to_linear(k[i]) * *n as f32 / w;
                    }
                }
                // The box's colour, on the sRGB byte grid, so it is one colour.
                let m = m.map(|x| to_linear(to_srgb(x)));
                for (k, _) in b {
                    to.insert(*k, m);
                }
            }
            for c in colors.iter_mut() {
                let m = to[&key(c)];
                c[..3].copy_from_slice(&m);
            }
        }
    }
    colors
        .iter()
        .map(key)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// A primitive's surface colour: its material's base colour factor times
/// its base colour texture (sampled nearest, repeating).
struct Surface {
    factor: [f32; 4],
    image: Option<Image>,
}

impl Surface {
    fn of(src: &GltfSource, prim: &gltf::Primitive) -> Surface {
        let pbr = prim.material().pbr_metallic_roughness();
        let image = pbr.base_color_texture().and_then(|t| {
            src.images
                .get(t.texture().source().index())
                .and_then(|b| crate::cook::texture::decode(b).ok())
        });
        Surface {
            // The cook's default material is 0.8 grey (model.rs), glTF's white.
            factor: match prim.material().index() {
                Some(_) => pbr.base_color_factor(),
                None => [0.8, 0.8, 0.8, 1.0],
            },
            image,
        }
    }

    /// Linear RGBA at `uv` times the vertex colour `c`.
    fn at(&self, uv: [f32; 2], c: [f32; 4]) -> [f32; 4] {
        let t = match &self.image {
            Some(img) if img.width > 0 && img.height > 0 => {
                let x = ((uv[0] - uv[0].floor()) * img.width as f32) as u32;
                let y = ((uv[1] - uv[1].floor()) * img.height as f32) as u32;
                let i = 4 * (y.min(img.height - 1) * img.width + x.min(img.width - 1)) as usize;
                let p = &img.rgba[i..i + 4];
                [
                    to_linear(p[0]),
                    to_linear(p[1]),
                    to_linear(p[2]),
                    p[3] as f32 / 255.0,
                ]
            }
            _ => [1.0; 4],
        };
        [0, 1, 2, 3].map(|i| self.factor[i] * t[i] * c[i])
    }

    /// The mean colour over a triangle (seven points: the centroid, the
    /// three points near the corners and the three edge midpoints).
    fn over(&self, v: [&FVertex; 3]) -> [f32; 4] {
        const W: [[f32; 3]; 7] = [
            [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            [4.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0],
            [1.0 / 6.0, 4.0 / 6.0, 1.0 / 6.0],
            [1.0 / 6.0, 1.0 / 6.0, 4.0 / 6.0],
            [0.5, 0.5, 0.0],
            [0.0, 0.5, 0.5],
            [0.5, 0.0, 0.5],
        ];
        let mut sum = [0f32; 4];
        for w in W {
            let uv = [0, 1].map(|a| w[0] * v[0].uv[a] + w[1] * v[1].uv[a] + w[2] * v[2].uv[a]);
            let c = [0, 1, 2, 3].map(|a| w[0] * v[0].c[a] + w[1] * v[1].c[a] + w[2] * v[2].c[a]);
            let s = self.at(uv, c);
            for i in 0..4 {
                sum[i] += s[i] / W.len() as f32;
            }
        }
        sum
    }
}

// ── textures ─────────────────────────────────────────────────────────────

/// The longest side of the model's default scene, in its units.
pub(in crate::cook) fn extent(src: &GltfSource) -> f32 {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for (p, world) in scene_primitives(src) {
        let b = p.bounding_box();
        for k in 0..8 {
            let corner = Vec3::new(
                if k & 1 == 0 { b.min[0] } else { b.max[0] },
                if k & 2 == 0 { b.min[1] } else { b.max[1] },
                if k & 4 == 0 { b.min[2] } else { b.max[2] },
            );
            let w = world.transform_point3(corner);
            lo = lo.min(w);
            hi = hi.max(w);
        }
    }
    if lo.x > hi.x {
        0.0
    } else {
        (hi - lo).max_element()
    }
}

/// The primitives of the model's default scene, each with its node's
/// placement.
fn scene_primitives(src: &GltfSource) -> Vec<(gltf::Primitive<'_>, glam::Mat4)> {
    use glam::{Mat4, Quat};
    let mut out = Vec::new();
    // A skinned mesh is placed by its bind pose (model.rs `skin_binds`).
    let binds = crate::cook::model::skin_binds(src);
    if let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) {
        let mut stack: Vec<(gltf::Node, Mat4)> =
            scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
        while let Some((node, parent)) = stack.pop() {
            let (t, r, s) = node.transform().decomposed();
            let world = parent
                * Mat4::from_scale_rotation_translation(
                    Vec3::from(s),
                    Quat::from_array(r),
                    Vec3::from(t),
                );
            if let Some(mesh) = node.mesh() {
                let at = binds.get(&node.index()).copied().unwrap_or(world);
                out.extend(mesh.primitives().map(|p| (p, at)));
            }
            stack.extend(node.children().map(|c| (c, world)));
        }
    }
    out
}

/// The metres one side of glTF image `image` spans on the model's surface
/// (ArtTexels.density's: a texture of `n` texels a side draws `n / span`
/// texels per metre): over the triangles that sample it, in their UV set,
/// the area-weighted median of `sqrt(world area / uv area)`, the measure
/// `og style check` takes. A texture laid once across the model spans its
/// longest side; an atlas of a crate's six faces less, a ground repeated
/// ten times across 40 m 4 m. `None` with no triangle to measure (no UVs,
/// or none that sample it): [`extent`] stands in.
pub(in crate::cook) fn texel_span(src: &GltfSource, image: usize) -> Option<f32> {
    let mut spans: Vec<(f64, f64)> = Vec::new();
    for (p, world) in scene_primitives(src) {
        if let Some(set) = uv_set(&p.material(), image) {
            primitive_spans(src, &p, world, set, &mut spans);
        }
    }
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let half = spans.iter().map(|x| x.1).sum::<f64>() / 2.0;
    let mut acc = 0.0;
    spans.iter().find_map(|(span, area)| {
        acc += area;
        (acc >= half).then_some(*span as f32)
    })
}

/// The UV set through which `material` samples glTF image `image`, if any
/// of its maps does.
fn uv_set(material: &gltf::Material, image: usize) -> Option<u32> {
    let pbr = material.pbr_metallic_roughness();
    let is = |t: gltf::Texture| t.source().index() == image;
    pbr.base_color_texture()
        .filter(|i| is(i.texture()))
        .map(|i| i.tex_coord())
        .or_else(|| {
            pbr.metallic_roughness_texture()
                .filter(|i| is(i.texture()))
                .map(|i| i.tex_coord())
        })
        .or_else(|| {
            material
                .normal_texture()
                .filter(|i| is(i.texture()))
                .map(|i| i.tex_coord())
        })
        .or_else(|| {
            material
                .occlusion_texture()
                .filter(|i| is(i.texture()))
                .map(|i| i.tex_coord())
        })
        .or_else(|| {
            material
                .emissive_texture()
                .filter(|i| is(i.texture()))
                .map(|i| i.tex_coord())
        })
}

/// Each triangle of `p` (placed by `world`) with its span in metres per UV
/// unit of set `set` and its area in square metres.
fn primitive_spans(
    src: &GltfSource,
    p: &gltf::Primitive,
    world: glam::Mat4,
    set: u32,
    out: &mut Vec<(f64, f64)>,
) {
    let reader = p.reader(|b| Some(&src.buffers[b.index()].0[..]));
    let (Some(pos), Some(uv)) = (reader.read_positions(), reader.read_tex_coords(set)) else {
        return;
    };
    let pos: Vec<Vec3> = pos.map(|q| world.transform_point3(Vec3::from(q))).collect();
    let uv: Vec<glam::Vec2> = uv.into_f32().map(glam::Vec2::from).collect();
    let idx: Vec<u32> = match reader.read_indices() {
        Some(i) => i.into_u32().collect(),
        None => (0..pos.len() as u32).collect(),
    };
    for t in idx.as_chunks::<3>().0 {
        let [a, b, c] = t.map(|i| i as usize);
        if a.max(b).max(c) >= pos.len().min(uv.len()) {
            continue;
        }
        let area = f64::from((pos[b] - pos[a]).cross(pos[c] - pos[a]).length()) / 2.0;
        let (e, f) = (uv[b] - uv[a], uv[c] - uv[a]);
        let uv_area = f64::from((e.x * f.y - e.y * f.x).abs()) / 2.0;
        if area > 1e-12 && uv_area > 1e-12 {
            out.push(((area / uv_area).sqrt(), area));
        }
    }
}

/// An area resample of `img` to `w` × `h`: colour in linear light weighted
/// by alpha, normals renormalised, linear data averaged.
fn resample(img: &Image, w: u32, h: u32, k: u32) -> Image {
    let (sw, sh) = (img.width as usize, img.height as usize);
    // To floats: colour linear and premultiplied, normals -1..1.
    let mut f: Vec<[f32; 4]> = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| match k {
            kind::COLOR | kind::PIXEL => {
                let a = p[3] as f32 / 255.0;
                [
                    to_linear(p[0]) * a,
                    to_linear(p[1]) * a,
                    to_linear(p[2]) * a,
                    a,
                ]
            }
            kind::NORMAL => [0, 1, 2, 3].map(|i| p[i] as f32 / 127.5 - 1.0),
            _ => [0, 1, 2, 3].map(|i| p[i] as f32 / 255.0),
        })
        .collect();
    // One axis at a time: each output texel the coverage-weighted mean of
    // the source texels under it.
    let pass = |data: &[[f32; 4]], n_in: usize, n_out: usize, lines: usize, along: bool| {
        let stride_line = if along { 1 } else { n_in };
        let _ = stride_line;
        let scale = n_in as f64 / n_out as f64;
        let mut out = vec![[0f32; 4]; n_out * lines];
        for line in 0..lines {
            for o in 0..n_out {
                let (a, b) = (o as f64 * scale, (o + 1) as f64 * scale);
                let mut acc = [0f64; 4];
                let mut i = a.floor() as usize;
                while (i as f64) < b && i < n_in {
                    let cover = (b.min(i as f64 + 1.0) - a.max(i as f64)).max(0.0);
                    let s = if along {
                        data[line * n_in + i]
                    } else {
                        data[i * lines + line]
                    };
                    for c in 0..4 {
                        acc[c] += s[c] as f64 * cover;
                    }
                    i += 1;
                }
                let px = acc.map(|x| (x / scale) as f32);
                if along {
                    out[line * n_out + o] = px;
                } else {
                    out[o * lines + line] = px;
                }
            }
        }
        out
    };
    f = pass(&f, sw, w as usize, sh, true);
    f = pass(&f, sh, h as usize, w as usize, false);
    let rgba = f
        .iter()
        .flat_map(|p| match k {
            kind::COLOR | kind::PIXEL => {
                let a = p[3];
                let c = |x: f32| if a > 0.0 { to_srgb(x / a) } else { 0 };
                [c(p[0]), c(p[1]), c(p[2]), (a * 255.0).round() as u8]
            }
            kind::NORMAL => {
                let n = Vec3::new(p[0], p[1], p[2]).normalize_or(Vec3::Z);
                let e = |x: f32| ((x + 1.0) * 127.5).round().clamp(0.0, 255.0) as u8;
                [e(n.x), e(n.y), e(n.z), e(p[3])]
            }
            _ => p.map(|x| (x * 255.0).round().clamp(0.0, 255.0) as u8),
        })
        .collect();
    Image {
        width: w,
        height: h,
        rgba,
    }
}

/// A mesh's texture as the contract holds it: the image, the kind it cooks
/// as, whether it has mips, and the report's line (`None`: the contract
/// leaves textures as they are). `extent` is the size in metres of the
/// meshes that use it (ArtTexels.density's), `None` when unknown: then
/// only ArtTexels.maxSize sizes it.
pub(in crate::cook) fn texture(
    c: &Conform,
    extent: Option<f32>,
    img: Image,
    k: u32,
    mips: bool,
) -> (Image, u32, bool, Option<String>) {
    if c.texture_desc().is_empty() {
        return (img, k, mips, None);
    }
    let mut what = Vec::new();
    let longest = img.width.max(img.height);
    let mut target = longest;
    if c.density > 0.0
        && let Some(size) = extent
    {
        let want = (c.density * size).max(4.0);
        let pow2 = 2f32.powf(want.log2().round()) as u32;
        if pow2 < target {
            target = pow2;
            what.push(format!(
                "ArtTexels.density {} over the {size:.2} m a side spans on the model",
                c.density
            ));
        }
    }
    if c.max_size > 0 && c.max_size < target {
        target = c.max_size;
        what.push(format!("ArtTexels.maxSize {}", c.max_size));
    }
    let mut img = img;
    if target < longest {
        let s = target as f64 / longest as f64;
        let (w, h) = (
            ((img.width as f64 * s).round() as u32).max(1),
            ((img.height as f64 * s).round() as u32).max(1),
        );
        what.insert(0, format!("{}x{} → {w}x{h}", img.width, img.height));
        img = resample(&img, w, h, k);
    }
    let color = k == kind::COLOR || k == kind::PIXEL;
    // Painterly 3D: the frame's paint fields in the texture's own texels,
    // after it is at its density, so a stroke is a few texels of the
    // surface wherever the camera stands.
    if let Some(p) = c.paint.as_ref().filter(|_| color) {
        p.apply(img.rgba.as_chunks_mut::<4>().0, img.width, img.height);
        what.push(format!("painted in texel space ({})", p.describe()));
    }
    if c.strict && !c.palette.is_empty() && color {
        let lab = palette_lab(&c.palette);
        let mut cache: BTreeMap<[u8; 3], [u8; 3]> = BTreeMap::new();
        for p in img.rgba.as_chunks_mut::<4>().0 {
            if p[3] == 0 {
                continue;
            }
            let key = [p[0], p[1], p[2]];
            let m = *cache
                .entry(key)
                .or_insert_with(|| nearest(&lab, key.map(to_linear)).map(to_srgb));
            p[..3].copy_from_slice(&m);
        }
        what.push(format!(
            "every opaque texel on ArtPalette.colors' {}",
            c.palette.len()
        ));
    }
    // A texture that cooks lossless (pixel art: `filter nearest`) shows
    // its colours exactly, so it holds to ArtPalette.maxColors as a baked
    // model does (a median cut, deterministic); a strict palette has
    // already mapped it.
    let lossless = k == kind::PIXEL || (c.filter == Filter::Nearest && k == kind::COLOR);
    if lossless && c.max_colors > 0 && !(c.strict && !c.palette.is_empty()) {
        let opaque: Vec<usize> = (0..img.rgba.len() / 4)
            .filter(|&i| img.rgba[i * 4 + 3] > 0)
            .collect();
        let mut colors: Vec<[f32; 4]> = opaque
            .iter()
            .map(|&i| {
                let p = &img.rgba[i * 4..i * 4 + 3];
                [to_linear(p[0]), to_linear(p[1]), to_linear(p[2]), 1.0]
            })
            .collect();
        let before = colors
            .iter()
            .map(|c| c.map(to_srgb))
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        if before > c.max_colors as usize {
            let after = reduce(&mut colors, &[], c.max_colors);
            for (&i, m) in opaque.iter().zip(&colors) {
                for (px, v) in img.rgba[i * 4..i * 4 + 3].iter_mut().zip(m) {
                    *px = to_srgb(*v);
                }
            }
            what.push(format!(
                "{before} → {after} colours (ArtPalette.maxColors {})",
                c.max_colors
            ));
        }
    }
    // Every other colour texture (linear, mipped: a photographic ground, a
    // crate) holds to the palette limits too, so a flat look's meshes are
    // not photographs under its bands (docs/art.md §2.3).
    if color
        && !lossless
        && (c.max_colors > 0 || c.ramp_steps > 0)
        && !(c.strict && !c.palette.is_empty())
        && let Some(line) = flatten(&mut img, c.max_colors, c.ramp_steps)
    {
        what.push(line);
    }
    let (mut k, mut mips) = (k, mips);
    if c.filter == Filter::Nearest && k == kind::COLOR {
        k = kind::PIXEL;
        what.push("filter nearest (pixel art: RGBA8, no mips)".into());
    } else if c.filter == Filter::Nearest {
        what.push("filter nearest is not cooked for non-colour data (kept linear)".into());
    }
    if c.mips == Mips::None && mips && k != kind::PIXEL {
        mips = false;
        what.push("mips none".into());
    }
    let note = Some(if what.is_empty() {
        format!("conform: {UNCHANGED}")
    } else {
        format!("conform: {}", what.join("; "))
    });
    (img, k, mips, note)
}

/// A linear colour texture held to `max` colours and `ramp` shades per hue
/// family (0: no limit), abstracted into flat regions ([`flat`]): the
/// report's line, or `None` when the texture already holds.
fn flatten(img: &mut Image, max: u32, ramp: u32) -> Option<String> {
    let (before, after, area) = flat::flatten(img, max, ramp)?;
    let limits = [(max, "maxColors"), (ramp, "rampSteps")]
        .iter()
        .filter(|(v, _)| *v > 0)
        .map(|(v, n)| format!("ArtPalette.{n} {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "{before} → {after} colours in flat regions of {area} texels or more ({limits})"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(text: &str) -> LookFile {
        LookFile::parse(&format!("type look\nversion 1\n---\n{text}")).unwrap()
    }

    fn of(text: &str) -> Option<Conform> {
        Conform::of(&look(text), &|_| Ok(vec![[255, 0, 0], [0, 0, 255]])).unwrap()
    }

    /// Painterly 3D: a look's Stylise paint fields paint a model's colour
    /// texture in its texels (a speck in a flat region is smoothed into it),
    /// leave its normal map alone, and move its stamp; without them the
    /// texture keeps its texels.
    #[test]
    fn the_paint_fields_paint_a_mesh_colour_texture_in_texel_space() {
        let speckled = || {
            let mut rgba = vec![120u8; 8 * 8 * 4];
            rgba.chunks_mut(4).for_each(|p| p[3] = 255);
            rgba[(3 * 8 + 3) * 4..(3 * 8 + 3) * 4 + 3].copy_from_slice(&[250, 20, 20]);
            Image {
                width: 8,
                height: 8,
                rgba,
            }
        };
        let plain = of("ArtTexels {\n  density 64\n}\n").unwrap();
        let (img, _, _, _) = texture(&plain, None, speckled(), kind::COLOR, true);
        assert_eq!(img.rgba, speckled().rgba, "no paint fields: untouched");
        let mut painted = plain.clone();
        painted.paint = Some(Paint {
            radius: 2.0,
            ..Paint::default()
        });
        assert_ne!(painted.texture_desc(), plain.texture_desc());
        let (img, _, _, note) = texture(&painted, None, speckled(), kind::COLOR, true);
        let at = (3 * 8 + 3) * 4;
        // Every sector holds the texel itself, so the speck is diluted into
        // its flat surround (250 → about 150), not kept.
        assert!(
            img.rgba[at] < 170 && img.rgba[at + 1] > 90,
            "the speck is painted over: {:?}",
            &img.rgba[at..at + 3]
        );
        assert!(note.unwrap().contains("painted in texel space"));
        let (img, _, _, _) = texture(&painted, None, speckled(), kind::NORMAL, true);
        assert_eq!(img.rgba, speckled().rgba, "a normal map is data");
    }

    /// A 64 x 64 linear colour texture: the left half every shade of one
    /// red, the right half every shade of one blue, a different shade on
    /// every texel row and column.
    fn two_ramps() -> Image {
        let mut rgba = Vec::new();
        for y in 0..64u32 {
            for x in 0..64u32 {
                let v = (40 + (y * 64 + x) % 32 * 6 + y / 8) as u8;
                rgba.extend_from_slice(&if x < 32 {
                    [v, v / 6, v / 7, 255]
                } else {
                    [v / 7, v / 5, v, 255]
                });
            }
        }
        Image {
            width: 64,
            height: 64,
            rgba,
        }
    }

    fn distinct(img: &Image) -> BTreeSet<[u8; 3]> {
        img.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2]])
            .collect()
    }

    /// The flat looks' palette limits (anime-cel's `maxColors 48 rampSteps
    /// 2`) hold a linear, mipped mesh texture: two ramps of hundreds of
    /// shades become at most two shades of each hue, each still red or
    /// blue. Old code applied the limits only to nearest-filtered textures,
    /// so a photographic texture kept every colour.
    #[test]
    fn the_palette_limits_flatten_a_linear_mesh_texture() {
        let src = two_ramps();
        assert!(distinct(&src).len() > 300);
        let c = of("ArtPalette {\n  maxColors 48\n  rampSteps 2\n}\n").unwrap();
        let (img, k, mips, note) = texture(&c, None, two_ramps(), kind::COLOR, true);
        assert_eq!(
            (k, mips),
            (kind::COLOR, true),
            "still a linear, mipped colour texture"
        );
        let colours = distinct(&img);
        let reds = colours.iter().filter(|c| c[0] > c[2]).count();
        let blues = colours.iter().filter(|c| c[2] > c[0]).count();
        assert!(
            reds <= 2 && blues <= 2 && reds + blues == colours.len(),
            "{colours:?}"
        );
        assert!(reds >= 1 && blues >= 1, "both hues survive: {colours:?}");
        // Every texel keeps its own hue.
        for (i, p) in img.rgba.as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(p[0] > p[2], i % 64 < 32, "texel {i}: {p:?}");
            assert_eq!(p[3], 255);
        }
        assert!(note.unwrap().contains("ArtPalette.rampSteps 2"));
        // A normal map is data: untouched.
        let (n, _, _, _) = texture(&c, None, two_ramps(), kind::NORMAL, true);
        assert_eq!(n.rgba, src.rgba);
    }

    /// A photograph of a lawn with a patch of dirt (every texel its own
    /// shade, noise of ±16 on each channel) cooks to flat fields: the lawn
    /// one green and the dirt one brown, edge to edge where the patch was,
    /// with no region smaller than the texture's smallest shape (36 texels
    /// at 256²). Old code (a cut to 48 colours and a 3x3 majority) left
    /// dozens of shades in hundreds of specks.
    #[test]
    fn the_palette_limits_flatten_a_photograph_into_fields() {
        let (w, h) = (256u32, 256u32);
        let green = [70i32, 130, 60];
        let brown = [130i32, 95, 60];
        // The dirt: a square from (64, 64) to (160, 160).
        let dirt = |x: u32, y: u32| (64..160).contains(&x) && (64..160).contains(&y);
        let mut seed = 0x2545_f491u32;
        let mut noise = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed % 33) as i32 - 16
        };
        let mut rgba = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let c = if dirt(x, y) { brown } else { green };
                for k in c {
                    rgba.push((k + noise()).clamp(0, 255) as u8);
                }
                rgba.push(255);
            }
        }
        let img = Image {
            width: w,
            height: h,
            rgba,
        };
        let c = of("ArtPalette {\n  maxColors 48\n}\n").unwrap();
        let (out, _, _, note) = texture(&c, None, img, kind::COLOR, true);
        let px: Vec<[u8; 3]> = out
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2]])
            .collect();
        let colours: BTreeSet<[u8; 3]> = px.iter().copied().collect();
        assert_eq!(colours.len(), 2, "one green, one brown: {colours:?}");
        // Away from the patch's edge (the filter's reach), each field is
        // its own colour: green where the lawn was, brown on the dirt.
        let at = |x: u32, y: u32| px[(y * w + x) as usize];
        let (lawn, soil) = (at(8, 8), at(112, 112));
        assert!(lawn[1] > lawn[0] && soil[0] > soil[1], "{lawn:?} {soil:?}");
        for y in 0..h {
            for x in 0..w {
                let inside = (68..156).contains(&x) && (68..156).contains(&y);
                let outside = !(60..164).contains(&x) || !(60..164).contains(&y);
                if inside {
                    assert_eq!(at(x, y), soil, "({x}, {y}) on the dirt");
                } else if outside {
                    assert_eq!(at(x, y), lawn, "({x}, {y}) on the lawn");
                }
            }
        }
        assert!(note.unwrap().contains("flat regions of 36 texels"));
    }

    #[test]
    fn a_contract_without_3d_fields_conforms_nothing() {
        assert_eq!(of(""), None);
        // Zero is no constraint; 2D-only and camera fields are not this module's.
        assert_eq!(
            of(
                "ArtShape {\n  propFaces 0\n  voxel 0\n}\nArtPixels {\n  unit 16\n}\nArtCamera {\n  projection orthographic\n}\n"
            ),
            None
        );
        // The palette limits hold every colour texture, not the models.
        let c = of("ArtPalette {\n  maxColors 16\n}\n").unwrap();
        assert!(!c.texture_desc().is_empty() && c.model_desc().is_empty());
        let c = of("ArtShape {\n  normals angle\n}\n").unwrap();
        assert_eq!(c.normals, Normals::Angle(30.0));
        assert!(c.texture_desc().is_empty() && !c.model_desc().is_empty());
    }

    /// A unit cube of 12 triangles, each face its own four vertices.
    fn cube() -> (Vec<FVertex>, Vec<u32>) {
        let mut v = Vec::new();
        let mut idx = Vec::new();
        for axis in 0..3 {
            for side in [0.0f32, 1.0] {
                let base = v.len() as u32;
                let (u, w) = ((axis + 1) % 3, (axis + 2) % 3);
                for (a, b) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                    let mut p = [0f32; 3];
                    p[axis] = side;
                    p[u] = a;
                    p[w] = b;
                    v.push(FVertex {
                        p,
                        c: [1.0; 4],
                        uv: [a, b],
                        ..Default::default()
                    });
                }
                let quad = if side > 0.0 {
                    [0, 1, 2, 0, 2, 3]
                } else {
                    [0, 2, 1, 0, 3, 2]
                };
                idx.extend(quad.iter().map(|q| base + q));
            }
        }
        (v, idx)
    }

    #[test]
    fn a_bevel_chamfers_every_edge_of_a_cube_and_closes_its_corners() {
        let (mut v, mut idx) = cube();
        let mut src: Vec<u32> = (0..v.len() as u32).collect();
        let (edges, added) = bevel(&mut v, &mut idx, &mut src, 0.1, 30.0).unwrap();
        // 12 edges (the faces' diagonals are soft), two triangles a
        // chamfer and one a corner.
        assert_eq!(edges, 12);
        assert_eq!(added, 12 * 2 + 8);
        // Every face corner is 0.1 in from both hard edges: no point but
        // the chamfers' lies within 0.1 of two of the cube's faces.
        for t in idx.as_chunks::<3>().0 {
            let n = cross(&v, t);
            assert!(n.length() > 0.0);
            // Facing outwards: away from the centre.
            let c =
                (pos(&v[t[0] as usize]) + pos(&v[t[1] as usize]) + pos(&v[t[2] as usize])) / 3.0;
            assert!(n.dot(c - Vec3::splat(0.5)) > 0.0, "inward face at {c}");
        }
        for x in &v {
            let near =
                x.p.iter()
                    .filter(|&&q| q.abs() < 1e-5 || (q - 1.0).abs() < 1e-5)
                    .count();
            let inset =
                x.p.iter()
                    .filter(|&&q| (q - 0.1).abs() < 1e-5 || (q - 0.9).abs() < 1e-5)
                    .count();
            assert!(near == 1 && inset == 2, "corner at {:?}", x.p);
        }
    }

    #[test]
    fn the_median_cut_leaves_at_most_max_colours_and_the_palette_its_own() {
        let mut c: Vec<[f32; 4]> = (0..64)
            .map(|i| {
                [
                    i as f32 / 63.0,
                    1.0 - i as f32 / 63.0,
                    (i % 7) as f32 / 6.0,
                    1.0,
                ]
            })
            .collect();
        let mut p = c.clone();
        assert!(reduce(&mut c, &[], 5) <= 5);
        assert!(reduce(&mut p, &[[255, 0, 0], [0, 0, 255]], 0) <= 2);
        for x in &p {
            assert!(
                x[..3] == [1.0, 0.0, 0.0] || x[..3] == [0.0, 0.0, 1.0],
                "{x:?}"
            );
        }
        // Red stays red, blue stays blue.
        let mut rb = vec![[0.9, 0.05, 0.0, 1.0], [0.0, 0.1, 0.8, 1.0]];
        reduce(&mut rb, &[[255, 0, 0], [0, 0, 255]], 0);
        assert_eq!(rb[0][..3], [1.0, 0.0, 0.0]);
        assert_eq!(rb[1][..3], [0.0, 0.0, 1.0]);
    }

    #[test]
    fn a_flipbook_steps_and_drops_frames_keeping_its_length() {
        let mut sheet = og_anim::FlipbookSheet::parse(
            "type flipbook\nversion 1\n---\ntexture \"t.png\"\nsize 64 8\ngrid 8 8\nclip \"run\" {\n  frames 0 1 2 3 4 5 6 7\n  fps 30\n}\n",
        )
        .unwrap();
        let m = Motion {
            step_fps: 12.0,
            max_frames: 4,
        };
        m.flipbook(&mut sheet);
        let clip = &sheet.clips[0];
        assert_eq!(clip.frames, [0, 2, 4, 6]);
        // Each kept frame held for two thirtieths, rounded to one twelfth.
        for d in &clip.durations {
            assert!((d - 1.0 / 12.0).abs() < 1e-6, "{d}");
        }
    }
}
