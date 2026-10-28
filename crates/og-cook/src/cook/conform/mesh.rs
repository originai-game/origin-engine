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
//!    model with a skin): the model's drawn triangles (each primitive
//!    times the nodes that draw it) are scaled by one fraction, so their
//!    sum is within the budget. meshoptimizer's simplify over the vertices,
//!    then over the positions alone (flat-shaded and seamed meshes), then
//!    its sloppy simplifier, and as a last resort the largest triangles.
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
//! texels per metre over the model's longest side (rounded to a power of
//! two) and `maxSize`; mapped to the palette where `ArtPalette.strict`;
//! `filter nearest` cooks a colour texture as pixel art (RGBA8, no mips,
//! nearest), `mips none` without mips.
//!
//! Clips (`#anim`): with `ArtMotion.stepFps` resampled at that rate as
//! STEP channels, so each pose holds for one step; past `maxFrames` the
//! rate is lowered until the clip holds that many, with a warning.
//! Flipbooks: each frame's duration held to whole steps (at least one),
//! and past `maxFrames` a clip keeps every k-th frame, the dropped
//! frames' time added to the one kept before them, with a warning.

use crate::cook::model::{FVertex, GltfSource};
use crate::cook::texture::Image;
use crate::cook::vox::{VoxModel, mesh_model};
use crate::format::kind;
use crate::look::{LookBlock, LookFile, hex_to_linear};
use glam::Vec3;
use og_core::ogt::Value;
use std::collections::BTreeMap;

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
    pub line: Option<Line>,
    pub motion: Option<Motion>,
}

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
            line,
            motion: (step_fps > 0.0 || max_frames > 0).then_some(Motion {
                step_fps,
                max_frames,
            }),
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
            "conform-mesh-v1 faces {} {} normals {:?} bevel {} voxel {} bakes {} {} line {:?}",
            self.prop_faces,
            self.character_faces,
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
        format!(
            "conform-texture-v1 density {} maxSize {} filter {:?} mips {:?} strict {} {}",
            self.density,
            self.max_size,
            self.filter,
            self.mips,
            self.strict,
            if self.strict {
                self.palette_desc()
            } else {
                String::new()
            }
        )
    }

    /// The face budget of a model: a character's when it has a skin.
    fn budget(&self, src: &GltfSource) -> (u32, &'static str) {
        if src.doc.skins().next().is_some() {
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

// ── geometry ─────────────────────────────────────────────────────────────

/// What conforming one primitive did, for the model's report.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Changes {
    /// Triangles before and after.
    pub faces: (usize, usize),
    /// Set by the conformance: normals (so the cook keeps them), vertex
    /// colours (so the primitive stores them).
    pub normals: bool,
    pub colors: bool,
    /// Voxelised: the edge used and the voxels.
    pub voxel: Option<(f32, usize)>,
    /// Bevelled: hard edges and triangles added.
    pub bevel: Option<(usize, usize)>,
    /// Distinct vertex colours after baking.
    pub distinct: usize,
    /// Why a step was left out.
    pub skipped: Vec<String>,
}

fn pos(v: &FVertex) -> Vec3 {
    Vec3::from(v.p)
}

fn cross(v: &[FVertex], t: &[u32]) -> Vec3 {
    let [a, b, c] = [0, 1, 2].map(|k| pos(&v[t[k] as usize]));
    (b - a).cross(c - a)
}

fn pkey(p: [f32; 3]) -> [u32; 3] {
    // -0.0 and 0.0 are one position.
    p.map(|x| (x + 0.0).to_bits())
}

/// Every corner its own vertex (sources follow).
fn unweld(v: &mut Vec<FVertex>, idx: &mut Vec<u32>, src: &mut Vec<u32>) {
    *v = idx.iter().map(|&i| v[i as usize]).collect();
    *src = idx.iter().map(|&i| src[i as usize]).collect();
    *idx = (0..v.len() as u32).collect();
}

/// Only the vertices `idx` uses, in first-use order.
fn compact(v: &mut Vec<FVertex>, idx: &mut [u32], src: &mut Vec<u32>) {
    let mut at = vec![u32::MAX; v.len()];
    let (mut nv, mut ns) = (Vec::new(), Vec::new());
    for i in idx.iter_mut() {
        if at[*i as usize] == u32::MAX {
            at[*i as usize] = nv.len() as u32;
            nv.push(v[*i as usize]);
            ns.push(src[*i as usize]);
        }
        *i = at[*i as usize];
    }
    *v = nv;
    *src = ns;
}

fn adapter(v: &[FVertex]) -> meshopt::VertexDataAdapter<'_> {
    meshopt::VertexDataAdapter::new(bytemuck::cast_slice(v), std::mem::size_of::<FVertex>(), 0)
        .expect("positions lead the vertex")
}

/// `idx` over the first vertex at each position.
fn by_position(v: &[FVertex], idx: &[u32]) -> Vec<u32> {
    let mut first: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let rep: Vec<u32> = (0..v.len() as u32)
        .map(|i| *first.entry(pkey(v[i as usize].p)).or_insert(i))
        .collect();
    idx.iter().map(|&i| rep[i as usize]).collect()
}

/// Simplifies to at most `target` triangles.
fn decimate(v: &mut Vec<FVertex>, idx: &mut Vec<u32>, src: &mut Vec<u32>, target: usize) {
    let target = target.max(1);
    if idx.len() / 3 <= target {
        return;
    }
    let want = target * 3;
    let a = adapter(v);
    let none = meshopt::SimplifyOptions::None;
    let mut out = meshopt::simplify(idx, &a, want, 1.0, none, None);
    if out.len() > want {
        let shadow = by_position(v, idx);
        out = meshopt::simplify(&shadow, &a, want, 1.0, none, None);
        if out.len() > want {
            out = meshopt::simplify_sloppy(&shadow, &a, want, 1.0, None);
        }
    }
    if out.len() > want || out.len() < 3 {
        // The largest triangles, in their order.
        let mut tris: Vec<(usize, f32)> = idx
            .as_chunks::<3>()
            .0
            .iter()
            .enumerate()
            .map(|(i, t)| (i, cross(v, t).length()))
            .collect();
        tris.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
        let mut keep: Vec<usize> = tris.iter().take(target).map(|t| t.0).collect();
        keep.sort_unstable();
        out = keep
            .iter()
            .flat_map(|&t| idx[t * 3..t * 3 + 3].to_vec())
            .collect();
    }
    compact(v, &mut out, src);
    *idx = out;
}

/// Chamfers the hard edges by `radius`; answers (hard edges, triangles
/// added), or `None` without a hard edge.
fn bevel(
    v: &mut Vec<FVertex>,
    idx: &mut Vec<u32>,
    src: &mut Vec<u32>,
    radius: f32,
    hard_angle: f32,
) -> Option<(usize, usize)> {
    let tris = idx.len() / 3;
    let shadow = by_position(v, idx);
    let id = |f: usize, k: usize| shadow[f * 3 + k % 3];
    let normal: Vec<Vec3> = (0..tris)
        .map(|f| cross(v, &idx[f * 3..f * 3 + 3]).normalize_or_zero())
        .collect();
    // Each undirected edge: its faces and the corner it starts at.
    let mut edges: BTreeMap<(u32, u32), Vec<(usize, usize)>> = BTreeMap::new();
    for f in 0..tris {
        for k in 0..3 {
            let (a, b) = (id(f, k), id(f, k + 1));
            if a != b {
                edges.entry((a.min(b), a.max(b))).or_default().push((f, k));
            }
        }
    }
    let cos = hard_angle.to_radians().cos();
    let hard_key = |a: u32, b: u32| -> bool {
        edges.get(&(a.min(b), a.max(b))).is_some_and(|e| {
            e.len() == 2
                && normal[e[0].0] != Vec3::ZERO
                && normal[e[1].0] != Vec3::ZERO
                && normal[e[0].0].dot(normal[e[1].0]) < cos
        })
    };
    let hard = |f: usize, k: usize| hard_key(id(f, k), id(f, k + 1));
    let hard_edges: Vec<(u32, u32)> = edges
        .keys()
        .copied()
        .filter(|&(a, b)| hard_key(a, b))
        .collect();
    if hard_edges.is_empty() {
        return None;
    }
    let p = |i: u32| pos(&v[i as usize]);
    let shortest = hard_edges
        .iter()
        .map(|&(a, b)| p(a).distance(p(b)))
        .fold(f32::MAX, f32::min);
    let r = radius.min(shortest * 0.25);
    // Each corner's inset point: away from its face's hard edges by r.
    let mut inset = vec![Vec3::ZERO; tris * 3];
    for f in 0..tris {
        for k in 0..3 {
            let c = |j: usize| pos(&v[idx[f * 3 + j % 3] as usize]);
            let (at, next, prev) = (c(k), c(k + 1), c(k + 2));
            let (un, up) = (next - at, prev - at);
            let (ln, lp) = (un.length(), up.length());
            let mut q = at;
            if ln > 0.0 && lp > 0.0 {
                let (un, up) = (un / ln, up / lp);
                let sin = un.cross(up).length();
                if sin > 1e-4 {
                    let s = r / sin;
                    if hard(f, k) {
                        q += up * s.min(lp * 0.45);
                    }
                    if hard(f, k + 2) {
                        q += un * s.min(ln * 0.45);
                    }
                }
            }
            inset[f * 3 + k] = q;
        }
    }
    // Faces meeting at a position across soft edges share one point (the
    // mean of their insets), so no crack opens between them.
    let mut at_pos: BTreeMap<u32, Vec<(usize, usize)>> = BTreeMap::new();
    for f in 0..tris {
        for k in 0..3 {
            at_pos.entry(id(f, k)).or_default().push((f, k));
        }
    }
    let mut group = vec![usize::MAX; tris * 3];
    let mut groups: Vec<(u32, Vec<usize>)> = Vec::new();
    for (&vid, corners) in &at_pos {
        let mut parent: Vec<usize> = (0..corners.len()).collect();
        fn root(p: &mut [usize], i: usize) -> usize {
            let mut i = i;
            while p[i] != i {
                p[i] = p[p[i]];
                i = p[i];
            }
            i
        }
        for (x, &(fx, kx)) in corners.iter().enumerate() {
            for (y, &(fy, ky)) in corners.iter().enumerate().skip(x + 1) {
                // Two faces at vid share a soft edge out of vid.
                let others = |f: usize, k: usize| [id(f, k + 1), id(f, k + 2)];
                let shared = others(fx, kx)
                    .into_iter()
                    .find(|o| others(fy, ky).contains(o));
                if let Some(o) = shared
                    && !hard_key(vid, o)
                {
                    let (rx, ry) = (root(&mut parent, x), root(&mut parent, y));
                    parent[rx.max(ry)] = rx.min(ry);
                }
            }
        }
        let mut seen: BTreeMap<usize, usize> = BTreeMap::new();
        for (x, &(f, k)) in corners.iter().enumerate() {
            let rt = root(&mut parent, x);
            let g = *seen.entry(rt).or_insert_with(|| {
                groups.push((vid, Vec::new()));
                groups.len() - 1
            });
            group[f * 3 + k] = g;
            groups[g].1.push(f * 3 + k);
        }
    }
    let point: Vec<Vec3> = groups
        .iter()
        .map(|(_, cs)| cs.iter().map(|&c| inset[c]).sum::<Vec3>() / cs.len() as f32)
        .collect();
    // The inset faces, every corner its own vertex.
    let mut nv: Vec<FVertex> = Vec::with_capacity(tris * 3 + hard_edges.len() * 4);
    let mut ns: Vec<u32> = Vec::new();
    let mut ni: Vec<u32> = Vec::new();
    let corner = |c: usize| -> (FVertex, u32) {
        let mut x = v[idx[c] as usize];
        x.p = point[group[c]].to_array();
        (x, src[idx[c] as usize])
    };
    for c in 0..tris * 3 {
        let (x, s) = corner(c);
        ni.push(nv.len() as u32);
        nv.push(x);
        ns.push(s);
    }
    let before = ni.len() / 3;
    let push_tri = |nv: &mut Vec<FVertex>,
                    ns: &mut Vec<u32>,
                    ni: &mut Vec<u32>,
                    t: [(FVertex, u32); 3],
                    up: Vec3| {
        let [a, b, c] = t.map(|x| Vec3::from(x.0.p));
        let n = (b - a).cross(c - a);
        if n.length_squared() < 1e-14 {
            return;
        }
        let order = if n.dot(up) < 0.0 {
            [0, 2, 1]
        } else {
            [0, 1, 2]
        };
        for o in order {
            ni.push(nv.len() as u32);
            nv.push(t[o].0);
            ns.push(t[o].1);
        }
    };
    // A quad across each hard edge.
    for &(a, b) in &hard_edges {
        let e = &edges[&(a, b)];
        let ((f1, k1), (f2, k2)) = (e[0], e[1]);
        let at = |f: usize, k: usize, of: u32| {
            if id(f, k) == of {
                f * 3 + k
            } else {
                f * 3 + (k + 1) % 3
            }
        };
        let (a1, b1, a2, b2) = (at(f1, k1, a), at(f1, k1, b), at(f2, k2, a), at(f2, k2, b));
        let up = normal[f1] + normal[f2];
        push_tri(
            &mut nv,
            &mut ns,
            &mut ni,
            [corner(a1), corner(b1), corner(b2)],
            up,
        );
        push_tri(
            &mut nv,
            &mut ns,
            &mut ni,
            [corner(a1), corner(b2), corner(a2)],
            up,
        );
    }
    // A fan closing each corner where three or more groups meet.
    for (&vid, corners) in &at_pos {
        let gs: Vec<usize> = {
            let mut g: Vec<usize> = corners.iter().map(|&(f, k)| group[f * 3 + k]).collect();
            g.sort_unstable();
            g.dedup();
            g
        };
        if gs.len() < 3 {
            continue;
        }
        let up = corners
            .iter()
            .map(|&(f, _)| normal[f])
            .sum::<Vec3>()
            .normalize_or_zero();
        if up == Vec3::ZERO {
            continue;
        }
        let centre = p(vid);
        let x = up.any_orthonormal_vector();
        let y = up.cross(x);
        let mut ring: Vec<(f32, usize)> = gs
            .iter()
            .map(|&g| {
                let d = point[g] - centre;
                (d.dot(y).atan2(d.dot(x)), g)
            })
            .collect();
        ring.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let first = |g: usize| corner(groups[g].1[0]);
        for w in 1..ring.len() - 1 {
            push_tri(
                &mut nv,
                &mut ns,
                &mut ni,
                [first(ring[0].1), first(ring[w].1), first(ring[w + 1].1)],
                up,
            );
        }
    }
    let added = ni.len() / 3 - before;
    *v = nv;
    *src = ns;
    *idx = ni;
    Some((hard_edges.len(), added))
}

/// Sets every corner's normal as the contract shades: corners unwelded,
/// the cook welds what came out the same.
fn shade(v: &mut Vec<FVertex>, idx: &mut Vec<u32>, src: &mut Vec<u32>, normals: Normals) {
    unweld(v, idx, src);
    let tris = v.len() / 3;
    let face: Vec<Vec3> = (0..tris)
        .map(|f| cross(v, &idx[f * 3..f * 3 + 3]))
        .collect();
    let set = |v: &mut [FVertex], i: usize, n: Vec3| {
        v[i].n = n.normalize_or(Vec3::Y).to_array();
    };
    match normals {
        Normals::Any => {}
        Normals::Flat => {
            for i in 0..v.len() {
                set(v, i, face[i / 3]);
            }
        }
        Normals::Smooth => {
            let mut sum: BTreeMap<([u32; 3], [u32; 2]), Vec3> = BTreeMap::new();
            for i in 0..v.len() {
                *sum.entry((pkey(v[i].p), v[i].uv.map(f32::to_bits)))
                    .or_insert(Vec3::ZERO) += face[i / 3];
            }
            for i in 0..v.len() {
                let n = sum[&(pkey(v[i].p), v[i].uv.map(f32::to_bits))];
                set(v, i, n);
            }
        }
        Normals::Angle(deg) => {
            let cos = deg.to_radians().cos();
            let mut faces_at: BTreeMap<[u32; 3], Vec<usize>> = BTreeMap::new();
            for (i, x) in v.iter().enumerate() {
                faces_at.entry(pkey(x.p)).or_default().push(i / 3);
            }
            for i in 0..v.len() {
                let own = face[i / 3].normalize_or_zero();
                let n: Vec3 = faces_at[&pkey(v[i].p)]
                    .iter()
                    .map(|&f| face[f])
                    .filter(|n| n.normalize_or_zero().dot(own) >= cos - 1e-6)
                    .sum();
                set(v, i, n);
            }
        }
    }
}

/// The primitive as voxels of `edge` (its own units, the grid aligned to
/// the origin); `None` when it has no surface.
fn voxelise(
    v: &[FVertex],
    idx: &[u32],
    surface: &Surface,
    edge: f32,
    palette: &[[u8; 3]],
    max_colors: u32,
) -> Option<(Vec<FVertex>, Vec<u32>, usize)> {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for &i in idx {
        lo = lo.min(pos(&v[i as usize]));
        hi = hi.max(pos(&v[i as usize]));
    }
    if lo.x > hi.x {
        return None;
    }
    let cell = |p: Vec3| (p / edge).floor().as_ivec3();
    let (c0, c1) = (cell(lo), cell(hi));
    let dims = (c1 - c0 + 1).max(glam::IVec3::ONE);
    if dims.max_element() > 256 {
        return None;
    }
    let (sx, sy, sz) = (dims.x as usize, dims.y as usize, dims.z as usize);
    let at = |c: glam::IVec3| {
        let d = c - c0;
        d.x as usize + sx * (d.y as usize + sy * d.z as usize)
    };
    // The surface: points over every triangle a third of an edge apart.
    let mut sum: Vec<[f32; 5]> = vec![[0.0; 5]; sx * sy * sz];
    for t in idx.as_chunks::<3>().0 {
        let [a, b, c] = [0, 1, 2].map(|k| &v[t[k] as usize]);
        let longest = pos(a)
            .distance(pos(b))
            .max(pos(b).distance(pos(c)))
            .max(pos(c).distance(pos(a)));
        let n = ((longest / (edge / 3.0)).ceil() as usize).clamp(1, 4096);
        for i in 0..=n {
            for j in 0..=n - i {
                let (wa, wb) = (i as f32 / n as f32, j as f32 / n as f32);
                let wc = 1.0 - wa - wb;
                let p = pos(a) * wa + pos(b) * wb + pos(c) * wc;
                let uv = [0, 1].map(|k| a.uv[k] * wa + b.uv[k] * wb + c.uv[k] * wc);
                let col = [0, 1, 2, 3].map(|k| a.c[k] * wa + b.c[k] * wb + c.c[k] * wc);
                let s = surface.at(uv, col);
                let e = &mut sum[at(cell(p).clamp(c0, c1))];
                for k in 0..4 {
                    e[k] += s[k];
                }
                e[4] += 1.0;
            }
        }
    }
    // The outside: flood-filled from a border one cell wide round the grid.
    let (px, py, pz) = (sx + 2, sy + 2, sz + 2);
    let pidx = |x: usize, y: usize, z: usize| x + px * (y + py * z);
    let mut outside = vec![false; px * py * pz];
    let solid_surface = |x: usize, y: usize, z: usize| {
        (1..=sx).contains(&x)
            && (1..=sy).contains(&y)
            && (1..=sz).contains(&z)
            && sum[(x - 1) + sx * ((y - 1) + sy * (z - 1))][4] > 0.0
    };
    let mut stack = vec![(0usize, 0usize, 0usize)];
    outside[0] = true;
    while let Some((x, y, z)) = stack.pop() {
        let mut go = |x: usize, y: usize, z: usize| {
            if !outside[pidx(x, y, z)] && !solid_surface(x, y, z) {
                outside[pidx(x, y, z)] = true;
                stack.push((x, y, z));
            }
        };
        if x > 0 {
            go(x - 1, y, z);
        }
        if x + 1 < px {
            go(x + 1, y, z);
        }
        if y > 0 {
            go(x, y - 1, z);
        }
        if y + 1 < py {
            go(x, y + 1, z);
        }
        if z > 0 {
            go(x, y, z - 1);
        }
        if z + 1 < pz {
            go(x, y, z + 1);
        }
    }
    // The surface cells' colours, held to the palette or cut, then indexed.
    let surface_cells: Vec<usize> = (0..sum.len()).filter(|&i| sum[i][4] > 0.0).collect();
    let mut colors: Vec<[f32; 4]> = surface_cells
        .iter()
        .map(|&i| {
            let s = sum[i];
            [s[0] / s[4], s[1] / s[4], s[2] / s[4], 1.0]
        })
        .collect();
    let max = if max_colors > 0 {
        max_colors.min(255)
    } else {
        255
    };
    reduce(&mut colors, palette, max);
    if !palette.is_empty() && palette.len() > 255 {
        reduce(&mut colors, &[], 255);
    }
    let key = |c: &[f32; 4]| [to_srgb(c[0]), to_srgb(c[1]), to_srgb(c[2])];
    let mut index: BTreeMap<[u8; 3], u8> = BTreeMap::new();
    for c in &colors {
        let n = index.len() as u8 + 1;
        index.entry(key(c)).or_insert(n);
    }
    let mut table = vec![[0f32; 3]; 256];
    for (k, &i) in &index {
        table[i as usize] = k.map(to_linear);
    }
    let mut grid = vec![0u8; sum.len()];
    for (n, &i) in surface_cells.iter().enumerate() {
        grid[i] = index[&key(&colors[n])];
    }
    let mut model = VoxModel {
        size: [sx as u32, sy as u32, sz as u32],
        voxels: Vec::new(),
    };
    for z in 0..sz {
        for y in 0..sy {
            for x in 0..sx {
                let i = x + sx * (y + sy * z);
                let inside = !outside[pidx(x + 1, y + 1, z + 1)];
                let c = if grid[i] != 0 {
                    grid[i]
                } else if inside {
                    1
                } else {
                    continue;
                };
                model.voxels.push([x as u8, y as u8, z as u8, c]);
            }
        }
    }
    let count = model.voxels.len();
    let mut out_v = Vec::new();
    let mut out_i = Vec::new();
    for q in mesh_model(&model) {
        let base = out_v.len() as u32;
        let col = table[q.color as usize];
        for c in q.corners {
            out_v.push(FVertex {
                p: [0, 1, 2].map(|a| (c0[a] as f32 + c[a]) * edge),
                n: q.normal,
                uv: [0.0; 2],
                c: [col[0], col[1], col[2], 1.0],
                t: [0.0; 4],
                key: [0, 0],
            });
        }
        out_i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Some((out_v, out_i, count))
}

/// The model-space transform of the first node of the default scene that
/// draws `prim`'s mesh (identity when none does): voxels sit on one grid
/// across the model's parts, not one per part.
fn placement_of(src: &GltfSource, prim: &gltf::Primitive) -> glam::Mat4 {
    use glam::{Mat4, Quat};
    let key = |p: &gltf::Primitive| {
        (
            p.get(&gltf::Semantic::Positions).map(|a| a.index()),
            p.indices().map(|a| a.index()),
        )
    };
    let Some(mesh) = src
        .doc
        .meshes()
        .find(|m| m.primitives().any(|p| key(&p) == key(prim)))
    else {
        return Mat4::IDENTITY;
    };
    let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) else {
        return Mat4::IDENTITY;
    };
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    stack.reverse();
    while let Some((node, parent)) = stack.pop() {
        let (t, r, s) = node.transform().decomposed();
        let world = parent
            * Mat4::from_scale_rotation_translation(
                Vec3::from(s),
                Quat::from_array(r),
                Vec3::from(t),
            );
        if node.mesh().is_some_and(|m| m.index() == mesh.index()) {
            return world;
        }
        let mut children: Vec<_> = node.children().map(|c| (c, world)).collect();
        children.reverse();
        stack.extend(children);
    }
    Mat4::IDENTITY
}

/// The triangles a model draws (each primitive times the default scene's
/// nodes that draw its mesh, at least once): the face budget's measure.
fn drawn_triangles(src: &GltfSource) -> usize {
    let mut uses = vec![0usize; src.doc.meshes().count()];
    if let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) {
        let mut stack: Vec<gltf::Node> = scene.nodes().collect();
        while let Some(n) = stack.pop() {
            if let Some(m) = n.mesh() {
                uses[m.index()] += 1;
            }
            stack.extend(n.children());
        }
    }
    src.doc
        .meshes()
        .map(|m| {
            uses[m.index()].max(1)
                * m.primitives()
                    .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
                    .map(|p| triangles_of(p))
                    .sum::<usize>()
        })
        .sum()
}

fn triangles_of(p: gltf::Primitive) -> usize {
    p.indices()
        .map(|a| a.count())
        .or_else(|| p.get(&gltf::Semantic::Positions).map(|a| a.count()))
        .unwrap_or(0)
        / 3
}

/// Conforms one primitive's vertices (`v`, before welding), triangles and
/// source vertices to the contract. The cook keeps normals and colours
/// the answer says were set.
pub(in crate::cook) fn primitive(
    c: &Conform,
    src: &GltfSource,
    prim: &gltf::Primitive,
    v: &mut Vec<FVertex>,
    idx: &mut Vec<u32>,
    sources: &mut Vec<u32>,
) -> Changes {
    let mut ch = Changes {
        faces: (idx.len() / 3, idx.len() / 3),
        ..Changes::default()
    };
    let skinned = v.iter().any(|x| x.key != [0, 0]);
    let surface = Surface::of(src, prim);
    let (budget, _) = c.budget(src);
    let target = (budget > 0).then(|| {
        let total = drawn_triangles(src).max(1);
        if total <= budget as usize {
            usize::MAX
        } else {
            ((idx.len() / 3) as u64 * budget as u64 / total as u64) as usize
        }
    });
    if c.voxel > 0.0 && skinned {
        ch.skipped
            .push("not voxelised: it is skinned or morphed".into());
    }
    if c.voxel > 0.0 && !skinned {
        // Voxelised in model space, then brought back under the node.
        let place = placement_of(src, prim);
        let inverse = place.inverse();
        let placed: Vec<FVertex> = v
            .iter()
            .map(|x| FVertex {
                p: place.transform_point3(pos(x)).to_array(),
                ..*x
            })
            .collect();
        let mut edge = c.voxel;
        for _ in 0..5 {
            match voxelise(&placed, idx, &surface, edge, &c.palette, c.max_colors) {
                Some((nv, ni, count))
                    if target.is_none_or(|t| ni.len() / 3 <= t) || edge >= c.voxel * 16.0 =>
                {
                    ch.distinct = nv
                        .iter()
                        .map(|x| x.c.map(f32::to_bits))
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    *sources = vec![0; nv.len()];
                    *v = nv
                        .into_iter()
                        .map(|x| FVertex {
                            p: inverse.transform_point3(pos(&x)).to_array(),
                            n: inverse
                                .transpose()
                                .transform_vector3(Vec3::from(x.n))
                                .normalize_or(Vec3::Y)
                                .to_array(),
                            ..x
                        })
                        .collect();
                    *idx = ni;
                    ch.voxel = Some((edge, count));
                    ch.normals = true;
                    ch.colors = true;
                    break;
                }
                // Past the budget or the 256-cell grid: coarser.
                _ => edge *= 2.0,
            }
        }
        if ch.voxel.is_some() {
            ch.faces.1 = idx.len() / 3;
            return ch;
        }
        ch.skipped
            .push("not voxelised: wider than 256 voxels of 16 times the edge".into());
    }
    if let Some(t) = target.filter(|&t| t != usize::MAX) {
        decimate(v, idx, sources, t);
    }
    if c.bevel > 0.0 {
        let hard = match c.normals {
            Normals::Angle(a) => a,
            _ => 30.0,
        };
        let before = (v.clone(), idx.clone(), sources.clone());
        if let Some(b) = bevel(v, idx, sources, c.bevel, hard) {
            ch.bevel = Some(b);
            // The chamfers count against the budget: simplify the mesh
            // under them and bevel again.
            if let Some(t) = target.filter(|&t| t != usize::MAX)
                && idx.len() / 3 > t
            {
                (*v, *idx, *sources) = before;
                decimate(v, idx, sources, t.saturating_sub(b.1).max(t / 2));
                ch.bevel = bevel(v, idx, sources, c.bevel, hard);
                if idx.len() / 3 > t {
                    decimate(v, idx, sources, t);
                    ch.skipped
                        .push("the chamfers were simplified to stay within the budget".into());
                }
            }
            if c.normals == Normals::Any {
                // The chamfers' own faces need normals: each its face's.
                shade(v, idx, sources, Normals::Angle(hard));
                ch.normals = true;
            }
        }
    }
    if c.bakes() {
        let flat = c.normals == Normals::Flat;
        if flat {
            unweld(v, idx, sources);
        }
        let mut colors: Vec<[f32; 4]> = if flat {
            v.as_chunks::<3>()
                .0
                .iter()
                .flat_map(|t| {
                    let m = surface.over([&t[0], &t[1], &t[2]]);
                    [m; 3]
                })
                .collect()
        } else {
            v.iter().map(|x| surface.at(x.uv, x.c)).collect()
        };
        ch.distinct = reduce(&mut colors, &c.palette, c.max_colors);
        for (x, col) in v.iter_mut().zip(colors) {
            x.c = col;
        }
        ch.colors = true;
    }
    if c.normals != Normals::Any {
        shade(v, idx, sources, c.normals);
        ch.normals = true;
    }
    ch.faces.1 = idx.len() / 3;
    ch
}

/// The report's lines for a model's conformed primitives.
pub(in crate::cook) fn notes(c: &Conform, src: &GltfSource, changes: &[Changes]) -> Vec<String> {
    if changes.is_empty() {
        return Vec::new();
    }
    let (before, after): (usize, usize) = changes
        .iter()
        .fold((0, 0), |(a, b), x| (a + x.faces.0, b + x.faces.1));
    let mut what = vec![format!("{before} → {after} triangles")];
    let (budget, name) = c.budget(src);
    if budget > 0 {
        let drawn = drawn_triangles(src);
        what.push(format!(
            "ArtShape.{name} {budget} ({drawn} drawn before{})",
            if drawn > budget as usize {
                ", decimated"
            } else {
                ", within"
            }
        ));
    }
    if c.normals != Normals::Any {
        what.push(format!("normals {:?}", c.normals).to_lowercase());
    }
    let voxels: usize = changes.iter().filter_map(|x| x.voxel).map(|x| x.1).sum();
    if let Some(edge) = changes
        .iter()
        .filter_map(|x| x.voxel)
        .map(|x| x.0)
        .reduce(f32::max)
    {
        what.push(format!(
            "voxelised at {edge} m ({voxels} voxels{})",
            if edge > c.voxel {
                format!(", ArtShape.voxel {} coarsened to fit", c.voxel)
            } else {
                String::new()
            }
        ));
    }
    let (edges, added) = changes
        .iter()
        .filter_map(|x| x.bevel)
        .fold((0, 0), |(a, b), x| (a + x.0, b + x.1));
    if edges > 0 {
        what.push(format!(
            "bevel {} m on {edges} hard edges (+{added} triangles)",
            c.bevel
        ));
    }
    if c.bakes() {
        what.push(format!(
            "vertex colours baked, {} distinct ({}), materials drawn white without a base colour texture",
            changes.iter().map(|x| x.distinct).max().unwrap_or(0),
            if c.palette.is_empty() {
                format!("ArtPalette.maxColors {}", c.max_colors)
            } else {
                format!("on ArtPalette.colors' {}", c.palette.len())
            }
        ));
    }
    if let Some(l) = &c.line {
        what.push(format!("outline hull data for ArtLine width {}", l.width));
    }
    let mut out = vec![format!("conform: {}", what.join("; "))];
    let mut skipped: Vec<&String> = changes.iter().flat_map(|x| &x.skipped).collect();
    skipped.sort();
    skipped.dedup();
    out.extend(skipped.into_iter().map(|s| format!("conform: {s}")));
    out
}

/// Per cooked vertex, the area-weighted normal over every corner at its
/// position: the direction the outline hull extrudes along.
pub(in crate::cook) fn hull_normals(pos: &[[f32; 3]], idx: &[u32]) -> Vec<[f32; 3]> {
    let mut sum: BTreeMap<[u32; 3], Vec3> = BTreeMap::new();
    for t in idx.as_chunks::<3>().0 {
        let [a, b, c] = [0, 1, 2].map(|k| Vec3::from(pos[t[k] as usize]));
        let n = (b - a).cross(c - a);
        for &i in t {
            *sum.entry(pkey(pos[i as usize])).or_insert(Vec3::ZERO) += n;
        }
    }
    pos.iter()
        .map(|p| {
            sum.get(&pkey(*p))
                .copied()
                .unwrap_or(Vec3::Y)
                .normalize_or(Vec3::Y)
                .to_array()
        })
        .collect()
}

/// The `outline` section's header (the module docs give the layout).
pub(in crate::cook) fn hull_header(l: &Line, primitives: usize) -> Vec<u8> {
    let words: [u32; 8] = [
        1,
        l.width.to_bits(),
        l.color[0].to_bits(),
        l.color[1].to_bits(),
        l.color[2].to_bits(),
        l.place,
        l.distance.to_bits(),
        primitives as u32,
    ];
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

// ── textures ─────────────────────────────────────────────────────────────

/// The longest side of the model's default scene, in its units.
fn extent(src: &GltfSource) -> f32 {
    use glam::{Mat4, Quat};
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
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
                for p in mesh.primitives() {
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
            }
            stack.extend(node.children().map(|c| (c, world)));
        }
    }
    if lo.x > hi.x {
        0.0
    } else {
        (hi - lo).max_element()
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

/// A texture of the model `src` as the contract holds it: the image, the
/// kind it cooks as, whether it has mips, and the report's line (`None`:
/// unchanged).
pub(in crate::cook) fn texture(
    c: &Conform,
    src: &GltfSource,
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
    if c.density > 0.0 {
        let size = extent(src);
        let want = (c.density * size).max(4.0);
        let pow2 = 2f32.powf(want.log2().round()) as u32;
        if pow2 < target {
            target = pow2;
            what.push(format!(
                "ArtTexels.density {} over the model's {size:.2} m",
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
    let note = (!what.is_empty()).then(|| format!("conform: {}", what.join("; ")));
    (img, k, mips, note)
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

    #[test]
    fn a_contract_without_3d_fields_conforms_nothing() {
        assert_eq!(of(""), None);
        // Zero is no constraint; 2D-only and camera fields are not this module's.
        assert_eq!(
            of(
                "ArtShape {\n  propFaces 0\n  voxel 0\n}\nArtPixels {\n  unit 16\n}\nArtCamera {\n  projection orthographic\n}\nArtPalette {\n  maxColors 16\n}\n"
            ),
            None
        );
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
