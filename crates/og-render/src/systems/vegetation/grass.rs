//! Grass: blades generated on the GPU every frame over a terrain layer, to
//! the horizon (R6, `shaders/grass*.wgsl`).
//!
//! A [`GrassDraw`] in [`crate::RenderScene::grass`] grows a field over one
//! of the frame's terrains where its layer shows. Nothing is stored per
//! blade: candidates are the points of a progressive blue-noise tile
//! ([`blue_noise_tile`]) copied over 16 m chunks, each copy offset by a
//! hash of its chunk and cell, and a candidate becomes a blade while its
//! rank is below the density there (the layer's weight times the distance
//! table, [`DistanceLod`]), so the field thins out evenly with distance
//! and the survivors widen to keep the ground covered. Blades take one of
//! three levels of detail (strips of 7, 3 and 1 rows), dithered over a 4 m
//! band at each boundary; past the blades the terrain takes the field's
//! colour ([`far_gpu`], terrain_lit.wgsl `grass_far`), so the far field is
//! the same colour as the near one and there is no seam.
//!
//! **Each frame**, the CPU lists the chunks in the camera's view (and, for
//! a field that casts, those whose blades' shadows reach into it) within
//! `distance`, nearest first, each cut into slices of [`SLICE`]
//! candidates by how many it may keep. Three compute passes follow
//! (grass_generate.wgsl): every candidate is evaluated at its own slot
//! (placement, layer weight, holes, the blade's frustum test, its level
//! and the wind's bend), a prefix sum of the slices' counts gives each
//! slice its place, and the blades are compacted, in candidate order, into
//! the instance buffer the three indexed indirect draws read. Every
//! blade's place follows from the candidates alone, so a frame lays out
//! the same instances on every run (bevy_grass appended with atomics).
//!
//! **Drawn** in the main camera's prepass and lit pass with pipelines and
//! groups of their own (group 1: the field's uniform and blades; no
//! binding is added to any other pipeline), and with `cast_shadows` into
//! the first shadow cascade. Wind is og.Wind at the tick (its travelling
//! wave, gust patches scrolled downwind and a sway per blade, all reduced
//! in f64 on the CPU): presentation only, and the previous frame's bend is
//! kept in each blade for the motion vectors.
//!
//! Ported from bevy_grass v0.20.3 (third_party/bevy-grass: src/mirror.rs,
//! src/shader_model.rs, src/settings.rs and the shaders). MIT License,
//! Copyright (c) 2026 Gediminas Mikelenas. Permission is hereby granted,
//! free of charge, to any person obtaining a copy of this software and
//! associated documentation files (the "Software"), to deal in the
//! Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or
//! sell copies of the Software, and to permit persons to whom the Software
//! is furnished to do so, subject to the following conditions: The above
//! copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software. THE SOFTWARE IS
//! PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
//! INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
//! FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//! FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.

use crate::ground::{GroundDraw, GroundGpu, horizontal_scale};
use crate::wind::WindFieldGpu;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::sync::OnceLock;

mod draw;
mod plan;
mod prepare;
mod renderer;
mod uniforms;

pub use plan::*;
pub(crate) use renderer::*;
pub(crate) use uniforms::*;

/// A field of grass: the zero value is not usable; use `default()`, a
/// meadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grass {
    /// The terrain layer whose weight places the blades; with a height
    /// blend, as the blend draws it with every layer's height level (as
    /// [`crate::TerrainScatter::layer`]).
    pub layer: u32,
    /// Blades per square metre where the layer's weight is 1, near the camera.
    pub density: f32,
    /// How far from the camera blades are drawn (m); they thin out to
    /// nothing there.
    pub distance: f32,
    /// The terrain takes the field's average colour under its blades,
    /// where they thin out and past them.
    pub far_tint: bool,
    /// The tallest blade (m) and how much shorter blades are, 0..1.
    pub height: f32,
    pub height_variation: f32,
    /// A blade's width at its root (m).
    pub width: f32,
    /// How far blades droop along their length, 0..1.
    pub curve: f32,
    /// How far blades lean at rest along their face, as a share of height.
    pub lean: f32,
    /// The clumps' size (m), how far blades lean out from their clump's
    /// centre, and how far they are pulled in towards it (tufts), 0..1.
    pub clump_size: f32,
    pub clump_lean: f32,
    pub tufts: f32,
    /// Linear colours: roots, tips and dry grass.
    pub base_color: Vec3,
    pub tip_color: Vec3,
    pub dry_color: Vec3,
    /// How much blades vary in colour by clump, blade and patch, 0..1.
    pub color_variation: f32,
    /// The share of clumps that are dry, and how dry tips get, 0..1.
    pub dryness: f32,
    /// How much light passes through a blade lit from behind, 0..1.
    pub translucency: f32,
    /// The upper blade's gloss, 0..1, and the blade's roughness.
    pub sheen: f32,
    pub roughness: f32,
    /// How much og.Wind bends the blades (0: still), how stiff they are
    /// (0..1), and how much full-detail tips flutter.
    pub sway: f32,
    pub stiffness: f32,
    pub flutter: f32,
    /// Blades cast shadows into the first [`SHADOW_CASCADES`] cascades.
    pub cast_shadows: bool,
    pub seed: u32,
    /// Seconds trampled blades take to stand again (`trample.rs`).
    pub recovery: f32,
    /// Up to three more kinds of blade among the field's own (its height,
    /// width, curve and colours are the first): each clump grows one.
    pub types: [Option<GrassType>; 3],
    /// Where `types` grow (red, green and blue the shares of the first,
    /// second and third; the rest the field's own kind), over the
    /// terrain's extent as its control map; `None`: by their shares.
    pub type_map: Option<crate::TextureHandle>,
    /// Multiply the density and the height (their red channels), laid
    /// out as `type_map`.
    pub density_map: Option<crate::TextureHandle>,
    pub height_map: Option<crate::TextureHandle>,
}

/// Another kind of blade in a [`Grass`] field: its shape and colours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrassType {
    pub height: f32,
    pub width: f32,
    pub curve: f32,
    pub base_color: Vec3,
    pub tip_color: Vec3,
    /// Clumps of this kind per clump of the field's own, without a type map.
    pub share: f32,
}

impl Default for GrassType {
    fn default() -> Self {
        let g = Grass::default();
        GrassType {
            height: g.height,
            width: g.width,
            curve: g.curve,
            base_color: g.base_color,
            tip_color: g.tip_color,
            share: 1.0,
        }
    }
}

impl Default for Grass {
    fn default() -> Self {
        Grass {
            layer: 0,
            density: 300.0,
            distance: 150.0,
            far_tint: true,
            height: 0.45,
            height_variation: 0.4,
            width: 0.012,
            curve: 0.5,
            lean: 0.3,
            clump_size: 1.2,
            clump_lean: 0.4,
            tufts: 0.0,
            base_color: Vec3::new(0.04, 0.085, 0.024),
            tip_color: Vec3::new(0.235, 0.39, 0.11),
            dry_color: Vec3::new(0.45, 0.38, 0.17),
            color_variation: 0.3,
            dryness: 0.15,
            translucency: 0.6,
            sheen: 0.4,
            roughness: 0.6,
            sway: 1.0,
            stiffness: 0.5,
            flutter: 0.3,
            cast_shadows: true,
            seed: 1,
            recovery: 4.0,
            types: [None; 3],
            type_map: None,
            density_map: None,
            height_map: None,
        }
    }
}

/// A field of grass drawn this frame over terrain `terrain` (an index into
/// [`crate::RenderScene::terrains`], a drawn one: its data, placement and
/// layers place the blades).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrassDraw {
    pub terrain: usize,
    pub grass: Grass,
}

impl Grass {
    /// The field's kinds of blade, its own first: (height, width, share).
    fn kinds(&self) -> [Option<(f32, f32, f32)>; 4] {
        let mut out = [Some((self.height, self.width, 1.0)), None, None, None];
        for (k, t) in self.types.iter().enumerate() {
            out[k + 1] = t.map(|t| (t.height.max(0.0), t.width.max(0.0), t.share.max(0.0)));
        }
        out
    }

    /// The tallest blade of any kind (m).
    pub fn tallest(&self) -> f32 {
        self.kinds().iter().flatten().fold(0.0, |a, k| a.max(k.0))
    }

    /// The widest root of any kind (m).
    pub fn widest(&self) -> f32 {
        self.kinds().iter().flatten().fold(0.0, |a, k| a.max(k.1))
    }

    pub(crate) fn usable(&self) -> bool {
        self.density > 0.0
            && self.distance > 0.0
            && self.height > 0.0
            && self.width > 0.0
            && self.density.is_finite()
            && self.distance.is_finite()
    }
}

// ── the CPU model (twins of grass_generate.wgsl) ───────────────────────────

/// A chunk's side (m in the world, divided by the terrain's horizontal scale).
pub const CHUNK: f32 = 16.0;
/// Points in the blue-noise tile.
pub const TILE_POINTS: u32 = 1024;
/// Candidates per slice: one workgroup's share of a chunk.
pub const SLICE: u32 = 512;
/// The most slices a field generates in a frame (the farthest are dropped).
pub const MAX_SLICES: u32 = 8192;
/// The most blades a field draws in a frame: past it, [`plan`] thins the
/// whole field evenly (its survivors widen) rather than dropping the far
/// chunks.
pub const MAX_BLADES: u32 = 1 << 21;
/// Rows of each level of detail's strip (15, 7 and 3 vertices).
pub const LOD_ROWS: [u32; 3] = [7, 3, 1];
/// How much a gust brightens the blades it presses down (and the ground
/// past them): grass_common.wgsl `grass_gust_sheen`.
const GUST_SHEEN: f32 = 1.5;
/// The narrowest a blade draws (px).
const MIN_WIDTH_PX: f32 = 1.0;
/// The sun's cascades blades cast into (G5): the near field's two, where
/// a blade's shadow is wider than a texel.
pub(crate) const SHADOW_CASCADES: usize = 2;
/// How much of a blade's width blocks a ray crossing the field's canopy
/// (the canopy's extinction is blades per square metre times their width
/// times this, per metre of the ray).
const CANOPY_BLOCKING: f32 = 0.5;
/// How far a blade seen edge-on turns towards the eye.
const TURN_TO_EYE: f32 = 0.4;

// ── drawing ────────────────────────────────────────────────────────────────

/// One field's buffers.
struct FieldGpu {
    generator: wgpu::Buffer,
    draw: wgpu::Buffer,
    slices: wgpu::Buffer,
    /// What `slices` holds, so a frame writes only the blocks that changed.
    slices_held: crate::upload::DirtyUpload,
    draws: wgpu::Buffer,
    /// Slices and blades the buffers hold.
    slice_cap: u32,
    blade_cap: u32,
    gen_group: wgpu::BindGroup,
    draw_group: wgpu::BindGroup,
}

/// One field's work this frame: its buffers' slot, its terrain's draw (the
/// group generation binds), its slices and whether it casts.
pub(crate) struct GrassField {
    slot: usize,
    /// Its maps (generation group 2).
    maps: wgpu::BindGroup,
    terrain_draw: usize,
    arrayed: bool,
    slices: u32,
    cast: bool,
    /// How far from the camera the nearest ground a blade of LOD 1 and of
    /// LOD 2 can shade lies (m): a shadow pass whose receivers are all
    /// nearer draws neither.
    shades_from: [f32; 2],
}

/// The fields to generate and draw this frame.
#[derive(Default)]
pub(crate) struct GrassFrame {
    fields: Vec<GrassField>,
}

impl GrassFrame {
    /// Whether a field of the frame casts shadows.
    pub(crate) fn casts(&self) -> bool {
        self.fields.iter().any(|f| f.cast)
    }
}

/// Which pass grass draws into.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GrassPass {
    Lit,
    Prepass,
    Shadow,
}

/// What [`GrassRenderer::new`] builds its pipelines from.
pub(crate) struct GrassPipelines<'a> {
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub shadow_layout: &'a wgpu::BindGroupLayout,
    pub fx_layout: &'a wgpu::BindGroupLayout,
    pub samples: u32,
    /// The simulated wind field's lean (`wind_field.rs`), read by the generation.
    pub wind_field: wgpu::TextureView,
    /// The trample map (`trample.rs`), read by the generation.
    pub trample: wgpu::TextureView,
    /// The lit features the renderer is made without (pipeline_features.rs).
    pub off: crate::pipeline_features::Features,
}

/// What a frame's grass is prepared for: the main camera.
pub(crate) struct GrassView<'a> {
    pub camera: Vec3,
    pub frame: &'a crate::frame::CameraFrame,
    pub prev_view_proj: Mat4,
    pub height: u32,
    /// The simulated wind field's grid this frame (`wind_field.rs`).
    pub wind_grid: crate::wind_field::WindGrid,
    /// The trample map this frame (`trample.rs` `Trample::grid`).
    pub trample: [f32; 4],
}

/// Generates and draws [`GrassDraw`]s.
pub(crate) struct GrassRenderer {
    gen_layout: wgpu::BindGroupLayout,
    /// The generation's group 2: a field's type, density and height maps
    /// and their sampler; `white` stands in for a map a field lacks.
    maps_layout: wgpu::BindGroupLayout,
    white: wgpu::TextureView,
    map_sampler: wgpu::Sampler,
    draw_layout: wgpu::BindGroupLayout,
    /// The simulated wind field's lean (`wind_field.rs`) and the trample map.
    wind_field: wgpu::TextureView,
    trample: wgpu::TextureView,
    lit_layout: wgpu::PipelineLayout,
    prepass_layout: wgpu::PipelineLayout,
    shadow_layout: wgpu::PipelineLayout,
    samples: u32,
    /// The lit features its lit module is made without (pipeline_features.rs).
    off: crate::pipeline_features::Features,
    /// The lit, prepass and shadow pipelines, asked for the first frame a
    /// field is in the scene (gpu/deferred.rs).
    pipelines: Option<crate::deferred::Deferred<[wgpu::RenderPipeline; 3]>>,
    /// The generation pipelines (evaluate, scan, compact) over each
    /// terrain layout, standard then layer arrays, asked for as the draw
    /// pipelines are.
    generate: [Option<crate::deferred::Deferred<[wgpu::ComputePipeline; 3]>>; 2],
    tile: wgpu::Buffer,
    indices: wgpu::Buffer,
    fields: Vec<FieldGpu>,
    /// The tick the last frame drew (the motion vectors' wind).
    last_tick: Option<u64>,
}
