//! Scatter: detail meshes (grass, flowers, stones) over a terrain where one
//! of its layers shows, near the camera (phase 2; drawn from cached chunks
//! in phase 3).
//!
//! Copies sit on a jittered grid fixed to the terrain: one candidate per
//! cell of `1 / sqrt(density)` metres, turned and sized by a hash of the
//! cell and the seed, kept with the probability of its layer's weight
//! there, so a copy keeps its place, size and turn as the camera moves.
//! [`Terrain::scatter`] answers them for a camera on the CPU (tests, tools).
//!
//! **Drawn from chunks.** A [`ScatterDraw`] in [`crate::RenderScene::scatters`]
//! has the renderer place its copies itself, 32 × 32 cells at a time: a
//! chunk's world transforms are built once (when it first comes within the
//! distance, and again only when its terrain data changes), uploaded into
//! an instance buffer of its own and kept while it is in use. Each frame
//! costs a cull of the chunks against every pass's volume and one instanced
//! draw per chunk; the vertex shader drops copies past the distance and
//! shrinks and thins them out over the fade: each copy shrinks to nothing
//! over its own part of the fade, chosen by a hash of its place, so the
//! field thins as grass does (`shaders/scatter*.wgsl`). No pixel of the
//! distance fade is dithered: a static dither left grain TAA cannot
//! resolve. Only the crossfade from a copy's mesh to its impostor
//! thresholds pixels, by the blue-noise tile turned each render frame
//! ([`crossfade_noise_offset`]), the same in the lit, prepass and shadow
//! passes. A streamed terrain's chunks are built
//! once the level-0 tiles under them have arrived.
//!
//! **Wind.** [`crate::RenderScene::wind`] sways the copies in the vertex
//! stage of every pass (lit, prepass and shadow): each vertex above the
//! copy's origin moves downwind by the copy's lean at the tick times its
//! height times its height over the mesh's (so the root stays put and the
//! tip moves furthest), lowered to keep its distance from the root. The
//! lean is `strength × (1 + gust × w)`, `w` a wave of two octaves that
//! travels downwind at `speed` over [`WIND_WAVELENGTH`] (a little offset
//! per copy, from its position), so gusts sweep across the field. Its
//! phases are reduced on the CPU in f64 from the tick, so a long session
//! sways as smoothly as its first minute, and the previous frame's are
//! kept for the motion vectors. A chunk's bounds grow by the furthest lean.

use crate::ground::{
    Ground, GroundDraw, GroundGpu, MAX_TERRAIN_LAYERS, Terrain, TerrainData, TerrainSource,
    TileGround, horizontal_scale, smoothstep,
};
use crate::material::Side;
use crate::scene::{MaterialHandle, MeshHandle};
use crate::terrain_tiles::TerrainLayout;
use glam::{Mat3, Mat4, Vec2, Vec3};
use std::collections::HashMap;
use std::sync::Arc;

mod draw;
mod pipelines;
mod placement;
mod prepare;
mod renderer;
mod source;
pub(crate) use placement::*;
use renderer::*;
pub(crate) use source::{scatter_lit_source, scatter_shadow_source};

/// The most instances one [`TerrainScatter`] places in a frame on the CPU.
pub const MAX_SCATTER_INSTANCES: usize = 65_536;

/// Cells per chunk side.
const CHUNK_CELLS: u32 = 32;
/// Chunks a frame builds for a streamed terrain (all of them for a field in
/// memory or a settled frame).
const CHUNKS_PER_FRAME: usize = 16;
/// Frames a chunk out of use is kept.
const KEEP_FRAMES: u64 = 240;

/// How copies are scattered: the zero value is not usable; use `default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainScatter {
    /// The layer whose weight (control map, slope and altitude rules,
    /// normalised over the layers) places the instances.
    pub layer: u32,
    /// Instances per square metre where the layer's weight is 1.
    pub density: f32,
    /// How far from the camera (along x and z) instances are drawn.
    pub distance: f32,
    /// The outer part of `distance` over which they shrink and thin out (0..1).
    pub fade: f32,
    /// The range of their uniform scale.
    pub scale: [f32; 2],
    /// How far they lean with the ground: 0 upright, 1 along its normal.
    pub align: f32,
    pub seed: u32,
    /// How much the [`crate::RenderScene::wind`] sways them: 1 as it
    /// blows, 0 still (stones). Not part of the placement.
    pub sway: f32,
    /// Clusters (G4): copies grow only where a smooth noise of patches
    /// `noise_scale` metres across rises past `noise_threshold` (0..1; 0
    /// everywhere), their edges dithered over a short band.
    pub noise_scale: f32,
    pub noise_threshold: f32,
    /// How far (m, along x and z) this scatter's copies and those of the
    /// scatters placed before it on the same terrain keep apart: the larger
    /// of the two footprints. 0: none.
    pub footprint: f32,
    /// Placed before every scatter of lower priority on the same terrain
    /// (equal: the scene's order), whose copies keep clear of its footprints.
    pub priority: u32,
    /// How far above the scene's water ([`crate::RenderScene::water`])
    /// the ground must stand for a copy to grow (m); 0: anywhere.
    pub avoid_water: f32,
    /// Thins the copies where its red channel is below 1, over the
    /// terrain's extent as its control map. Drawn only (the GPU cull,
    /// `scatter_cull.rs`; no effect without compute); not part of the
    /// placement.
    pub density_map: Option<crate::TextureHandle>,
    /// Placed on the GPU (`scatter_place.rs`), a visual mode: the same
    /// rules from the level-0 tiles the GPU holds, nothing on the CPU but
    /// planning each chunk once, its copies matching the CPU's
    /// ([`Terrain::scatter_among`], still the answer for gameplay) but for
    /// the last bits of the arithmetic. Where compute does not run (or a
    /// chunk's footprint chain is too long or wide), the CPU places it.
    /// On by default; false places every chunk on the CPU.
    pub gpu: bool,
}

/// What places a scatter's copies besides its own rules
/// ([`Terrain::scatter_among`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScatterContext {
    /// The scatters over the same terrain placed before it (a higher
    /// priority, or the same and earlier in the scene), first placed
    /// first: its copies keep clear of their footprints.
    pub before: Vec<TerrainScatter>,
    /// The water surfaces `avoid_water` keeps clear of: each one's frame
    /// from the world (its surface local y = 0) and its half extent along
    /// local x and z (infinite for an endless sea).
    pub water: Vec<(Mat4, Vec2)>,
}

impl ScatterContext {
    /// The context of scatter `index` of `all` (each with its terrain's
    /// data index): the usable scatters over the same data placed before
    /// it, and `water` (world surfaces: their transform, size and whether
    /// endless).
    pub fn of(
        all: &[(usize, TerrainScatter)],
        index: usize,
        layers: usize,
        water: impl IntoIterator<Item = (Mat4, Vec2, bool)>,
    ) -> ScatterContext {
        let (data, me) = all[index];
        let mut before: Vec<(u32, usize, TerrainScatter)> = all
            .iter()
            .enumerate()
            .filter(|&(i, &(d, o))| {
                d == data
                    && i != index
                    && o.usable(layers)
                    && (o.priority > me.priority || (o.priority == me.priority && i < index))
            })
            .map(|(i, &(_, o))| (o.priority, i, o))
            .collect();
        og_core::sort::sort_by(&mut before, |a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        ScatterContext {
            before: before.into_iter().map(|b| b.2).collect(),
            water: water
                .into_iter()
                .map(|(m, size, endless)| {
                    let half = if endless { Vec2::INFINITY } else { size * 0.5 };
                    (m.inverse(), half)
                })
                .collect(),
        }
    }

    /// Whether any rule of `s` or the scatters before it reads the water.
    fn reads_water(&self, s: &TerrainScatter) -> bool {
        s.avoid_water > 0.0 || self.before.iter().any(|b| b.avoid_water > 0.0)
    }
}

impl Default for TerrainScatter {
    fn default() -> Self {
        TerrainScatter {
            layer: 0,
            density: 2.0,
            distance: 40.0,
            fade: 0.3,
            scale: [0.8, 1.2],
            align: 0.0,
            seed: 1,
            sway: 1.0,
            noise_scale: 8.0,
            noise_threshold: 0.0,
            footprint: 0.0,
            priority: 0,
            avoid_water: 0.0,
            density_map: None,
            gpu: true,
        }
    }
}

/// The material shader that sways a placed model in the wind (R6): its
/// `vertex_offset` hook moves each vertex above the model's origin as
/// `scatter_sway` moves a scattered copy's (the same lean, gusts and
/// wave), with its height in the model's units. Parameters, written each
/// frame by its user (og-runtime's `og.Model.sway`):
/// `wind` (xy the wind's horizontal direction, z its strength times the
/// sway, w its gust), `wave` (xy [`wind_phases`], z the wave's wavenumber,
/// w the model's top above its origin). A hooked material draws through
/// the lit, prepass and shadow pipelines of every other material shader,
/// static or skinned, so the shadows and the depth sway with it.
pub fn wind_shader() -> crate::MaterialShader {
    use crate::hooks::{ParamKind, ShaderParam};
    let param = |name: &str| ShaderParam {
        name: name.into(),
        kind: ParamKind::Vec4,
        default: [0.0; 4],
    };
    crate::MaterialShader {
        name: "og.wind".into(),
        code: WIND_HOOK.into(),
        params: vec![param("wind"), param("wave")],
        mode: Default::default(),
    }
}

/// `wind_shader`'s hook: `scatter_sway` (scatter_common.wgsl) over the
/// vertex's own place, the vertex's height its local y.
const WIND_HOOK: &str = r#"
fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  let lean = p.wind.z;
  let top = p.wave.w;
  let y = h.local.y;
  if (lean <= 0.0 || top <= 0.0 || y <= 0.0) {
    return vec3<f32>(0.0);
  }
  let dir = p.wind.xy;
  // The place in the world (the same bits wherever the render origin is).
  let at = h.world.xz + h.origin.xz;
  let x = p.wave.z * dot(at, dir);
  let wave = (sin(x - p.wave.x) + 0.5 * sin(2.3 * x + 1.7 - p.wave.y)) / 1.5;
  let d = lean * (1.0 + p.wind.w * wave) * y * min(y / top, 1.0);
  let drop = y - sqrt(max(y * y - d * d, 0.0));
  return vec3<f32>(dir.x * d, -drop, dir.y * d);
}
"#;

impl TerrainScatter {
    /// What places its copies, as bits (a chunk's key).
    fn placement_bits(&self) -> [u32; 11] {
        [
            self.density,
            self.scale[0],
            self.scale[1],
            self.align,
            self.layer as f32,
            self.seed as f32,
            self.noise_scale,
            self.noise_threshold,
            self.footprint,
            self.priority as f32,
            self.avoid_water,
        ]
        .map(f32::to_bits)
    }

    fn usable(&self, layers: usize) -> bool {
        (self.layer as usize) < layers.min(MAX_TERRAIN_LAYERS)
            && self.density > 0.0
            && self.distance > 0.0
            && self.density.is_finite()
            && self.distance.is_finite()
    }

    /// The grid's cell size in the terrain's space.
    pub(crate) fn cell(&self) -> f32 {
        1.0 / self.density.clamp(1e-4, 400.0).sqrt()
    }

    /// Where on screen the fade starts (world metres).
    pub fn fade_from(&self) -> f32 {
        self.distance * (1.0 - self.fade.clamp(0.0, 1.0))
    }
}

/// Copies of a mesh scattered over a terrain drawn this frame, placed and
/// drawn by the renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScatterDraw {
    /// The terrain: an index into [`crate::RenderScene::terrains`] (its
    /// data, transform and layers place the copies).
    pub terrain: usize,
    pub mesh: MeshHandle,
    /// Its StandardMaterial (opaque or cut out).
    pub material: MaterialHandle,
    pub scatter: TerrainScatter,
    /// Whether the copies are drawn into the shadow passes.
    pub cast_shadows: bool,
    /// The mesh's impostor (`impostor.rs`): copies past
    /// `impostor_distance` draw as it instead of the mesh, dithered across
    /// the last tenth before it.
    pub impostor: Option<crate::Impostor>,
    /// How far from the camera (along x and z) copies draw as their mesh;
    /// 0 (or no impostor): always. Useful below the scatter's `distance`.
    pub impostor_distance: f32,
    /// The scatter of the scene's list whose copies this one draws a
    /// further part of (a tree's leaves, its wood the first part): placed
    /// by that one's rules and where its copies are, swaying with them as
    /// one plant, and leaving the impostor's card to it. `None`: copies of
    /// its own.
    pub with: Option<usize>,
    /// The mesh's coarser levels of detail, finest first: each one's mesh
    /// and its error in the mesh's units (`og-cook`'s `Lod::error`). Where
    /// the copies are culled on the GPU, a copy draws the coarsest level
    /// whose error is under [`crate::scatter_cull::LOD_PIXELS`] on screen;
    /// elsewhere the mesh. A level after a `None` is not drawn.
    pub lods: [Option<(MeshHandle, f32)>; crate::scatter_cull::SCATTER_LEVELS - 1],
}

impl ScatterDraw {
    /// The scatter of `all` that places the copies scatter `index` draws:
    /// itself, or the one it is a part of (which must be no part itself).
    pub(crate) fn placed_by(all: &[ScatterDraw], index: usize) -> usize {
        all[index]
            .with
            .filter(|&h| all.get(h).is_some_and(|d| d.with.is_none()))
            .unwrap_or(index)
    }

    /// The scatters of `all` that place copies, each with its terrain's
    /// index (what [`ScatterContext::of`] orders), and for every scatter
    /// of `all` its place in that list: a part's is its first part's, so
    /// it is placed as one with it and clears no footprint of its own.
    pub(crate) fn placing(all: &[ScatterDraw]) -> (Vec<(usize, TerrainScatter)>, Vec<usize>) {
        let mut placing = Vec::new();
        let mut slot = vec![usize::MAX; all.len()];
        for (i, d) in all.iter().enumerate() {
            if Self::placed_by(all, i) == i {
                slot[i] = placing.len();
                placing.push((d.terrain, d.scatter));
            }
        }
        for i in 0..all.len() {
            slot[i] = slot[Self::placed_by(all, i)];
        }
        (placing, slot)
    }
}

// ── drawing ─────────────────────────────────────────────────────────────────

/// `ScatterUniform` in scatter_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScatterGpu {
    camera: [f32; 4],
    fade: [f32; 4],
    view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    /// xy: og.Wind's horizontal direction (x, z), z: the scatter's sway
    /// (0: still); the lean is the wind field's (`wind_field.rs`).
    wind: [f32; 4],
    /// y: the mesh's top above its origin (m), zw: the scene's origin's x
    /// and z (`origin.rs`).
    wind_shape: [f32; 4],
    /// The impostor's sphere in the model's space (xyz centre, w radius),
    /// its grid and distance (x frames per side, y the distance past which
    /// copies draw as it: 0 never, z 1 when the lit pass has MSAA and so
    /// alpha to coverage, else its edge is the cut), and the direction to
    /// the sun (xyz).
    impostor: [f32; 4],
    impostor_grid: [f32; 4],
    impostor_light: [f32; 4],
    /// The trample map (`trample.rs` `Trample::grid`).
    trample: [f32; 4],
    /// The mesh's canopy centre in its own space (xyz) and 1 in w when it
    /// has foliage ([`crate::RenderScene::set_mesh_foliage`]), for its
    /// leaves' dome normals.
    canopy: [f32; 4],
}

impl ScatterGpu {
    fn impostor(&mut self, i: &crate::Impostor, distance: f32, light: Vec3, msaa: bool) {
        self.impostor = i.center.extend(i.radius).to_array();
        self.impostor_grid = [
            i.grid as f32,
            distance,
            f32::from(u8::from(msaa)),
            f32::from(u8::from(i.material_atlas)),
        ];
        self.impostor_light = light.extend(0.0).to_array();
    }
}

/// How far the crossfade's blue-noise thresholds turn on render frame
/// `frame_index` (`ScatterUniform.fade.y`): the golden ratio's multiples,
/// so each pixel's thresholds over consecutive frames are a low-
/// discrepancy sequence TAA integrates, while every frame's pattern stays
/// the tile's blue noise (Wolfe, Morrical, Akenine-Möller, Ramamoorthi,
/// *Spatiotemporal Blue Noise Masks*, EGSR 2022, §2; Peters, *Free blue
/// noise textures*, 2016). Reduced in f64, so it does not drift.
fn crossfade_noise_offset(frame_index: u64) -> f32 {
    const GOLDEN: f64 = 0.618_033_988_749_894_8;
    ((frame_index as f64 * GOLDEN).fract()) as f32
}

/// One pass's stride in the uniform buffer: a [`ScatterGpu`] rounded up to
/// the 256-byte dynamic offset alignment, so a slot never reads its
/// neighbour's.
const SLOT: u64 = (std::mem::size_of::<ScatterGpu>() as u64).next_multiple_of(256);
/// The pipelines' kinds past a [`ScatterPass`]'s: its impostor's, and its
/// crowd cards'.
const IMPOSTOR_KIND: u8 = 3;
const CARD_KIND: u8 = 6;

/// A crowd card's record (crowd.rs `card_record`, `CardIn`): a skinned
/// record's stride, its transform's columns, then (its place, flash, flash
/// colour, tint).
const CARD_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: crate::skin::SKINNED_INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x3, 6 => Float32x3, 7 => Float32x3, 8 => Float32x4],
};

/// What a scatter's chunks were built from, the same for all of them: its
/// terrain's data and placement, the placement's parameters, the layers
/// and the mesh.
#[derive(Debug, PartialEq, Eq, Hash)]
struct ScatterKey {
    data: usize,
    /// The data's revision.
    revision: u64,
    /// The placement's parameters: the scatter's, the scatters' placed
    /// before it and, when a rule reads it, the water's (as bits).
    params: Vec<u32>,
    transform: [u32; 16],
    /// The scene's origin the chunk's transforms are relative to
    /// (`origin.rs`): `transform` is its terrain's placement in the world.
    origin: [u32; 3],
    layers: Vec<u32>,
    mesh: usize,
}

/// A [`ScatterKey`] its chunks' keys share, hashed once. A frame keeps the
/// one of the frame before while its scatter's is the same
/// (`Scatters::shared`), so the keys a frame looks up mostly compare by
/// pointer; any two compare by value, as the key did when every chunk
/// carried its own copy.
#[derive(Clone, Debug)]
struct SharedKey {
    key: Arc<ScatterKey>,
    hash: u64,
}

impl SharedKey {
    fn new(key: ScatterKey) -> Self {
        use std::hash::{Hash, Hasher};
        // Fixed keys: equal keys hash the same in every map.
        let mut h = std::hash::DefaultHasher::new();
        key.hash(&mut h);
        SharedKey {
            key: Arc::new(key),
            hash: h.finish(),
        }
    }
}

impl PartialEq for SharedKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.key, &other.key) || (self.hash == other.hash && self.key == other.key)
    }
}

impl Eq for SharedKey {}

impl std::hash::Hash for SharedKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

/// What a chunk was built from: its scatter's [`ScatterKey`] and the
/// chunk's place.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ChunkKey {
    scatter: SharedKey,
    /// For a streamed terrain, the tiles under the chunk.
    under: Vec<usize>,
    chunk: (u32, u32),
    /// Placed on the GPU (`scatter_place.rs`).
    gpu: bool,
}

struct Chunk {
    buffer: Option<wgpu::Buffer>,
    /// Its copies as the GPU cull reads them (`scatter_cull.rs`).
    cull_group: Option<wgpu::BindGroup>,
    /// Its copies on the CPU: 0 for a chunk placed on the GPU, whose count
    /// and draws follow its copies in its buffer (`Placed`).
    count: u32,
    placed: Option<Placed>,
    center: Vec3,
    radius: f32,
    used: u64,
}

/// A chunk placed on the GPU: its buffer as its job writes it, and the
/// index count its draw was last given.
struct Placed {
    group: wgpu::BindGroup,
    index_count: u32,
}

/// Chunks a frame places on the GPU for a streamed terrain.
const GPU_CHUNKS_PER_FRAME: usize = 128;

/// One scatter's chunks to draw this frame.
pub(crate) struct ScatterBatch {
    pub mesh: usize,
    pub material: usize,
    /// Its material's shader (the scene's index): the lit and prepass
    /// pipelines run its hooks.
    pub shader: Option<usize>,
    pub cast_shadows: bool,
    pub side: Side,
    offset: u32,
    chunks: Vec<ChunkKey>,
    /// The mesh's coarser levels (`ScatterDraw::lods`): the scene's meshes.
    lods: [Option<usize>; crate::scatter_cull::SCATTER_LEVELS - 1],
    /// Per chunk, where the GPU cull listed its copies (`None`: drawn
    /// whole).
    culled: Vec<Option<crate::scatter_cull::Culled>>,
    /// How far the wind moves a copy at most: the chunks' bounds grow by it.
    sway: f32,
    /// The impostor's material: the chunks' copies past the impostor
    /// distance draw as it.
    impostor: Option<usize>,
}

/// The model impostors of one atlas drawn this frame
/// ([`crate::RenderScene::impostors`]): a range of the frame's impostor
/// instances, with their bounds.
pub(crate) struct ImpostorBatch {
    material: usize,
    cast_shadows: bool,
    offset: u32,
    first: u32,
    count: u32,
    center: Vec3,
    radius: f32,
}

/// A crowd's impostor cards this frame (crowd.rs **Impostors**,
/// `crowd_card.wgsl`): its sphere in the model's space, its atlas's views
/// and frames and the elevation it was baked at (radians), its draw in the
/// crowd cull's table (the GPU path), its records among the frame's CPU
/// card records (the CPU path) and whether it casts shadows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardInput {
    pub center: Vec3,
    pub radius: f32,
    pub views: u32,
    pub frames: u32,
    pub elevation: f32,
    pub draw: u32,
    pub cpu: Option<(u32, u32)>,
    pub cast_shadows: bool,
}

/// A crowd's cards as drawn: its uniform slot, and its material group (the
/// atlas and its parts' colours, `crowd_impostor.rs`) once baked.
pub(crate) struct CardBatch {
    pub input: CardInput,
    offset: u32,
    pub group: Option<wgpu::BindGroup>,
}

/// Where a pass's crowd cards come from: the crowd cull's row (its records
/// and draws), the CPU path's records, or none.
#[derive(Clone, Copy)]
pub(crate) enum CardSource<'a> {
    None,
    Gpu(&'a crate::crowd_gpu::CrowdCulling, u32),
    Cpu,
}

/// What [`ScatterRenderer::prepare`] answers: the scatters' batches, the
/// model impostors' and the crowds' cards.
#[derive(Default)]
pub(crate) struct ScatterBatches {
    pub scatters: Vec<ScatterBatch>,
    pub impostors: Vec<ImpostorBatch>,
    pub cards: Vec<CardBatch>,
}

impl ScatterBatches {
    /// Whether any copy, model impostor or crowd card this frame casts a shadow.
    pub(crate) fn casts(&self) -> bool {
        self.scatters.iter().any(|b| b.cast_shadows)
            || self.impostors.iter().any(|b| b.cast_shadows)
            || self.cards.iter().any(|b| b.input.cast_shadows)
    }
}

/// Which pass a scatter draws into.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScatterPass {
    Lit,
    Prepass,
    Shadow,
}

/// What [`ScatterRenderer::new`] builds its pipelines from.
pub(crate) struct ScatterPipelines<'a> {
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub material_layout: &'a wgpu::BindGroupLayout,
    pub shadow_layout: &'a wgpu::BindGroupLayout,
    /// The lit passes' effect group (the screen-space AO), group 3 of the lit pipelines.
    pub fx_layout: &'a wgpu::BindGroupLayout,
    /// The lit module's source with a material shader's hooks.
    pub lit_wgsl: Box<dyn Fn(&str) -> String + Send + Sync>,
    pub shadow_wgsl: String,
    pub samples: u32,
    /// The wind field's lean (`wind_field.rs`) the copies and cards sway by.
    pub wind_field: wgpu::TextureView,
    /// The trample map (`trample.rs`) they part by.
    pub trample: wgpu::TextureView,
    /// Whether compute runs: the main camera's copies are culled on the GPU.
    pub compute: bool,
    /// [`crate::RenderInfo::indirect_per_draw`]: the cull's draws read
    /// args of their own (`indirect_own.rs`).
    pub indirect_per_draw: bool,
}

/// Draws [`ScatterDraw`]s from cached chunks.
pub(crate) struct ScatterRenderer {
    lit_layout: wgpu::PipelineLayout,
    /// The lit pass's: `lit_layout` plus the effect group (SSAO).
    lit_fx_layout: wgpu::PipelineLayout,
    shadow_layout: wgpu::PipelineLayout,
    lit_module: wgpu::ShaderModule,
    shadow_module: wgpu::ShaderModule,
    /// The lit module with each material shader's hooks (the scene's
    /// shader index), made when a scatter's material first uses it.
    hooked: HashMap<usize, wgpu::ShaderModule>,
    lit_wgsl: Box<dyn Fn(&str) -> String + Send + Sync>,
    samples: u32,
    pipelines: HashMap<(Option<usize>, Side, u8), wgpu::RenderPipeline>,
    /// The shadow kinds' pipelines, one per target (shadow_target.rs).
    shadow_pipelines: HashMap<(Option<usize>, Side, u8), crate::ShadowSet>,
    uniform_layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    /// The blue-noise tile ([`crate::blue_noise`]) the mesh-to-impostor
    /// crossfade thresholds its pixels by, its bytes packed into a uniform
    /// array (group 2 binding 1): a texture there would take the lit pass's
    /// fragment stage past its 16 sampled textures.
    blue_noise: wgpu::Buffer,
    slots: u64,
    group: wgpu::BindGroup,
    chunks: HashMap<ChunkKey, Chunk>,
    /// Per scatter (its index in the scene's), the [`SharedKey`] its
    /// chunks were last looked up by.
    shared: Vec<Option<SharedKey>>,
    frame: u64,
    /// Chunks built for the last frame.
    pub built: u32,
    /// The tick the last frame drew (the analytic wind's previous phases).
    last_tick: Option<u64>,
    /// The frame's og.Wind (`WindField`, group 2 binding 2).
    wind_uniform: wgpu::Buffer,
    /// The wind field's lean (`wind_field.rs`) and the trample map.
    wind_field: wgpu::TextureView,
    trample: wgpu::TextureView,
    /// The frame's model impostors' transforms, and how many it holds.
    impostor_instances: Option<(wgpu::Buffer, u64)>,
    /// A card's quad (6 indices) and the CPU path's card records, with
    /// their capacity in bytes.
    card_quad: wgpu::Buffer,
    card_instances: Option<(wgpu::Buffer, u64)>,
    /// The GPU cull (with compute) and this frame's culled chunks, in order.
    cull: Option<crate::scatter_cull::ScatterCull>,
    culled_chunks: Vec<ChunkKey>,
    /// The GPU placement (with compute) and this frame's placed chunks, in
    /// the order of their jobs.
    place: Option<crate::scatter_place::ScatterPlace>,
    placed_chunks: Vec<ChunkKey>,
}

#[cfg(test)]
mod tests;
