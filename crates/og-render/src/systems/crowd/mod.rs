//! Crowds ([`crate::RenderScene::crowds`]): thousands of copies of one
//! skinned model, each posed by a baked clip (skin.rs, [`crate::BakedClip`])
//! at its own frame, drawn at the level of detail its distance asks for.
//!
//! A crowd is the GPU-driven path's skinned workload. Each frame its
//! members are one table (a record of 80 bytes each, [`MemberGpu`]); under
//! GPU culling (`crowd_gpu.rs`, `shaders/crowd_cull.wgsl`) a compute pass
//! per culled view (the camera, its planar reflection, each shadow pass)
//! tests every member's clip box against the view's planes and, for the
//! camera, the depth pyramid of the frame's occluders (cull_gpu.rs), picks
//! its level from the camera, and compacts the survivors, level by level
//! and in member order (a deterministic prefix sum), into the view's
//! region of skinned instance records; every (part, level) is one indexed
//! indirect draw of the skinned pipelines, which blend the clip's two
//! baked frames in the vertex stage (`skin_storage.wgsl` `palette_joint`).
//! The CPU does no work per member beyond writing its record: no sort, no
//! bounds, no per-view list.
//!
//! **Levels.** `errors` are the cook's errors of the model's levels
//! (`lod.rs`): a member draws the coarsest level whose error, projected at
//! its distance from the camera, stays under [`crate::lod::LOD_PIXEL_ERROR`]
//! (times the scene's and the crowd's bias), as [`crate::lod::select`]
//! picks a model's level, without the hysteresis (a member has no identity
//! across frames here; a level whose error stays under a pixel changes
//! nothing visible as it switches). Every view draws a member at the level
//! the camera picked, so its shadow is the shape it shows.
//!
//! **Impostors.** A crowd with an [`CrowdImpostor`] draws its farthest
//! members as animated octahedral-style cards (Assassin's Creed Unity's
//! and Total War's far crowds): the renderer bakes the crowd's full mesh
//! on the GPU once (`crowd_impostor.rs`), posed at `frames` frames of the
//! impostor's clip and seen from `views` directions around the vertical
//! at `elevation` above the horizon, into an atlas of `views` columns by
//! `frames` rows: the colour detail (its parts' textures and vertex
//! colours, without their materials' colours) with its coverage, and per
//! texel its normal and depth in the view's basis and which part it is.
//! Crowds whose parts share meshes share an atlas: a card colours each
//! part by its crowd's material, so outfits cost no atlas each. The card
//! is the last level of the chain: its error is a texel of the atlas on
//! the model (`2 × radius / size`, at least the coarsest mesh level's), so a
//! member draws it once a texel covers less than a pixel. A card faces
//! the camera's direction around the vertical, tilted to the bake's
//! elevation, blends the two views and the two frames around the member's
//! direction and place in its walk, writes its surface's depth and is lit
//! as the meshes are; in the shadow passes it faces the light. A member
//! playing a clip other than the impostor's keeps its coarsest mesh.
//! Under GPU culling the cull writes a card's record (its transform, its
//! place in the clip, its look) where a mesh level's would be and one
//! indexed indirect draw of a quad per crowd; on the CPU path every
//! card-level member's record is drawn whole (the depth test keeps what the
//! GPU cull would).
//!
//! **Bounds.** A member's box is its clip's box over every frame
//! (`skin::clip_box`, the union over the crowd's parts and levels) through
//! its transform: a blend of two frames stays inside it.
//!
//! **Motion.** Under TAA (or motion blur) each member's previous transform
//! takes one palette slot after the frame's other slots, and its record
//! names its previous clip's frame, so its motion vectors need no previous
//! baked palette (skin.rs, the palette buffer's layout).
//!
//! **The CPU path.** A frame culled on the CPU (`Culling::Cpu`, a device
//! without compute, indirect first instances or room for the crowd's
//! records), or one with secondary views or reflection probe faces (whose
//! skinned lists are the CPU's), draws its crowds through the skinned
//! instance lists: [`cpu_records`] picks each member's level by the same
//! rule and gives it one record per part, culled as every skinned instance
//! is. The two paths draw the same members at the same levels.

pub(crate) mod crowd_gpu;
pub(crate) mod crowd_impostor;

use crate::scene::{Camera, MaterialHandle, MeshHandle};
use crate::skin::{BakedClip, BakedPose, JointBox, SkinnedInstanceGpu};
use glam::{Mat4, Vec3};

/// The crowd cull (`shaders/crowd_cull.wgsl`) with the instance helpers and
/// the shared depth pyramid test it reads.
pub(crate) const CROWD_CULL_WGSL: &str = concat!(
    wgsl!("instance.wgsl"),
    wgsl!("crowd_cull.wgsl"),
    wgsl!("cull_scan.wgsl"),
    wgsl!("cull_compact.wgsl"),
    wgsl!("cull_plane.wgsl"),
    wgsl!("cull_box.wgsl"),
    wgsl!("cull_hiz.wgsl"),
    wgsl!("lod.wgsl")
);

/// The most levels a crowd draws: its mesh levels (`Crowd::errors`) and,
/// with an impostor, its card.
pub const MAX_CROWD_LEVELS: usize = 4;

/// A crowd's far level: animated impostor cards (module docs, **Impostors**).
/// Its zero value takes the defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CrowdImpostor {
    /// The clip it is baked from ([`crate::RenderScene::baked_clips`]):
    /// members playing another clip keep their coarsest mesh.
    pub clip: u32,
    /// Directions around the vertical it is baked from: 4 to 32 (0: 16).
    pub views: u32,
    /// Frames of the clip it holds, evenly over it: 2 to 32 (0: 16).
    pub frames: u32,
    /// Pixels a side of each view: 16 to 256 (0: 64).
    pub size: u32,
    /// How far above the horizon it is seen from, in degrees: 0 to 60 (0:
    /// the default, 10; a crowd seen from a person's height, or from a
    /// tower, picks its own).
    pub elevation: f32,
}

impl CrowdImpostor {
    /// Its views, frames, pixels a side and elevation (degrees) with the
    /// defaults filled in and each clamped to its range.
    pub fn resolved(&self) -> (u32, u32, u32, f32) {
        let pick = |v: u32, default: u32, lo: u32, hi: u32| {
            if v == 0 { default } else { v.clamp(lo, hi) }
        };
        let elevation = if self.elevation > 0.0 && self.elevation.is_finite() {
            self.elevation.min(60.0)
        } else {
            10.0
        };
        (
            pick(self.views, 16, 4, 32),
            pick(self.frames, 16, 2, 32),
            pick(self.size, 64, 16, 256),
            elevation,
        )
    }
}

/// One part of a crowd's model: its skinned meshes, level 0 the full mesh
/// and each after it coarser, and its material. A part with fewer levels
/// than the crowd draws its coarsest past them.
#[derive(Clone, Debug, PartialEq)]
pub struct CrowdPart {
    pub levels: Vec<MeshHandle>,
    pub material: MaterialHandle,
}

/// One member of a crowd: where it stands and the baked frame it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrowdMember {
    pub transform: Mat4,
    /// Its clip ([`crate::RenderScene::baked_clips`]) and frame, and the
    /// frame before's clip, frame and transform for motion vectors.
    pub pose: BakedPose,
    /// Its own hit flash and tint ([`crate::InstanceFx`]).
    pub fx: crate::InstanceFx,
    /// Darkened by the shadow maps.
    pub receive_shadows: bool,
}

impl CrowdMember {
    /// A member at `transform` showing frame `frame` of clip `clip`, the
    /// same as the frame before, no look, receiving shadows.
    pub fn new(transform: Mat4, clip: u32, frame: f32) -> Self {
        CrowdMember {
            transform,
            pose: BakedPose {
                clip,
                frame,
                previous_clip: clip,
                previous_frame: frame,
                previous_transform: transform,
            },
            fx: crate::InstanceFx::NONE,
            receive_shadows: true,
        }
    }
}

/// A crowd: one skinned model drawn once per member (module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct Crowd {
    /// The model's skinned parts, each a chain of levels.
    pub parts: Vec<CrowdPart>,
    /// Each level's error from the full mesh in the model's units (level
    /// 0 first, 0), at most [`MAX_CROWD_LEVELS`]; one level (`[0.0]`, the
    /// default) draws the full mesh everywhere.
    pub errors: Vec<f32>,
    /// The model's bounding sphere in its space, which a level's distance
    /// is measured from (as a model's, `lod::pixels_per_unit`).
    pub center: Vec3,
    pub radius: f32,
    /// Multiplies the projected errors with the scene's
    /// [`crate::RenderScene::lod_bias`]: 2 keeps detail twice as far.
    pub lod_bias: f32,
    /// Drawn into the shadow maps.
    pub cast_shadows: bool,
    /// The three.js layer mask its members are on (`views.rs`).
    pub layers: u32,
    /// Its far level, cards baked from its full mesh (module docs).
    pub impostor: Option<CrowdImpostor>,
    /// Its members. They all mirror the model or none does
    /// ([`Crowd::mirrored`]): a mirrored member goes in a crowd of its own.
    pub members: Vec<CrowdMember>,
}

impl Default for Crowd {
    fn default() -> Self {
        Crowd {
            parts: Vec::new(),
            errors: vec![0.0],
            center: Vec3::ZERO,
            radius: 0.0,
            lod_bias: 1.0,
            cast_shadows: true,
            layers: 1,
            impostor: None,
            members: Vec::new(),
        }
    }
}

impl Crowd {
    /// Whether its members mirror the model (a transform with a negative
    /// determinant, [`crate::mirrors`]): its draws then take clockwise as
    /// the front, as a mirrored instance's do. The first member's answer
    /// is the crowd's.
    pub fn mirrored(&self) -> bool {
        self.members
            .first()
            .is_some_and(|m| crate::mirrors(&m.transform))
    }

    /// How many levels it draws (1 to [`MAX_CROWD_LEVELS`]): its mesh
    /// levels, and its card last when it has an impostor.
    pub fn levels(&self) -> usize {
        self.mesh_levels() + self.impostor.is_some() as usize
    }

    /// How many of its levels are meshes (an impostor's card takes the last
    /// of [`MAX_CROWD_LEVELS`] when it has one).
    pub fn mesh_levels(&self) -> usize {
        let most = MAX_CROWD_LEVELS - self.impostor.is_some() as usize;
        self.errors.len().clamp(1, most)
    }

    /// Each level's error, rising (lod.rs's rule): its mesh levels', then
    /// its card's (a texel of the atlas on the model, at least the
    /// coarsest mesh's).
    pub fn level_errors(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.levels());
        let mut e = 0.0f32;
        for l in 0..self.mesh_levels() {
            e = e.max(self.errors.get(l).copied().unwrap_or(0.0).max(0.0));
            out.push(e);
        }
        if let Some(i) = &self.impostor {
            let size = i.resolved().2 as f32;
            out.push(e.max(2.0 * self.radius.max(0.0) / size));
        }
        out
    }
}

/// The level a member of `crowd` at `transform` draws seen by `camera`
/// in a view `view_height` pixels tall, with the scene's `bias`: the
/// coarsest whose projected error stays under a pixel ([`crate::lod`]).
/// `crowd_cull.wgsl` `member_level` computes the same.
pub fn member_level(
    crowd: &Crowd,
    transform: &Mat4,
    camera: &Camera,
    view_height: f32,
    bias: f32,
) -> usize {
    let levels = crowd.levels();
    if levels < 2 {
        return 0;
    }
    let errors = crowd.level_errors();
    let scale = max_scale(transform);
    let per_unit = crate::lod::pixels_per_unit(
        camera,
        transform.transform_point3(crowd.center),
        crowd.radius * scale,
        view_height,
    );
    crate::lod::select(&errors, per_unit * scale, bias * crowd.lod_bias, None)
}

/// The level `member` of `crowd` draws at the camera's level `level`
/// ([`member_level`]): a member playing a clip other than the impostor's
/// keeps the coarsest mesh in the card's place.
pub fn drawn_level(crowd: &Crowd, member: &CrowdMember, level: usize) -> usize {
    match &crowd.impostor {
        Some(i) if level == crowd.mesh_levels() && member.pose.clip != i.clip => level - 1,
        _ => level,
    }
}

/// The largest scale of `m`'s axes.
pub(crate) fn max_scale(m: &Mat4) -> f32 {
    m.x_axis
        .truncate()
        .length()
        .max(m.y_axis.truncate().length())
        .max(m.z_axis.truncate().length())
}

/// A member as the cull reads it (`crowd_cull.wgsl` `Member`): its
/// transform's columns, its pose (clip, frame, previous clip, previous
/// frame; the clips exact in an f32) and its look (the skinned record's
/// `look`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MemberGpu {
    pub columns: [[f32; 3]; 4],
    pub pose: [f32; 4],
    pub look: [f32; 4],
}

/// A crowd as the cull reads it (`crowd_cull.wgsl` `CrowdGpu`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CrowdGpu {
    /// Its first member, members, levels (its card's among them), its
    /// first draw.
    pub members: [u32; 4],
    /// Its parts, whether it casts shadows, may be occluded (no material
    /// vertex hook), its first clip box (`clip_base + clip`).
    pub info: [u32; 4],
    /// x: its layers; y: its card's draw plus one (0: no impostor), z: the
    /// impostor's clip, w: the frames of the clip it holds.
    pub layers: [u32; 4],
    /// Its levels' errors, each at least the one before ([`Crowd::level_errors`]).
    pub errors: [f32; 4],
    /// Its sphere's centre and radius.
    pub sphere: [f32; 4],
    /// x: the scene's level of detail bias times its own, y: its mesh levels.
    pub lod: [f32; 4],
}

/// A baked clip as a crowd plays it (`crowd_cull.wgsl` `ClipGpu`): its
/// first slot in the baked buffer, its joints and frames, and the crowd's
/// box over it (min, max), per (crowd, clip).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ClipGpu {
    pub info: [u32; 4],
    pub lo: [f32; 4],
    pub hi: [f32; 4],
}

/// One indirect draw of a crowd: a part at a level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CrowdDraw {
    pub crowd: u32,
    pub part: u32,
    pub level: u32,
    pub mesh: usize,
    pub material: usize,
    pub index_count: u32,
}

/// One crowd's cards (its impostor level): drawn as a quad of 6 indices,
/// its draw [`CrowdFrame::card_draw`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CrowdCard {
    /// The scene's crowd.
    pub crowd: u32,
    pub impostor: CrowdImpostor,
    /// Each part's full mesh and its material.
    pub parts: Vec<(usize, usize)>,
    pub cast_shadows: bool,
    pub center: Vec3,
    pub radius: f32,
}

/// The frame's crowds as tables (`build`). The renderer keeps one from
/// frame to frame, so a crowd's tables reuse the memory the last frame's
/// filled.
#[derive(Default)]
pub(crate) struct CrowdFrame {
    pub members: Vec<MemberGpu>,
    pub crowds: Vec<CrowdGpu>,
    pub clips: Vec<ClipGpu>,
    /// The mesh levels' draws, each crowd's (level, part) in order.
    pub draws: Vec<CrowdDraw>,
    /// The crowds' cards, their draws after the mesh levels'.
    pub cards: Vec<CrowdCard>,
    /// Each member's previous transform (TAA and motion blur only), in
    /// member order: the palette slots after the frame's others.
    pub previous: Vec<[[f32; 4]; 4]>,
    /// Whether any member moved or played on since the frame before.
    pub moved: bool,
    /// Per baked clip of the scene, whether a member plays it now or the
    /// frame before: the bakes the palette holds for the crowds.
    pub played: Vec<bool>,
}

impl CrowdFrame {
    pub fn len(&self) -> u32 {
        self.members.len() as u32
    }

    /// Writes each clip box row's first slot in the baked buffer
    /// (`ClipGpu::info.x`), once the palette has placed the clips
    /// [`Self::played`] names (`offset`). A row no member plays stays empty.
    pub fn place_clips(&mut self, offset: &dyn Fn(u32) -> usize) {
        let n = self.played.len().max(1);
        for (row, c) in self.clips.iter_mut().enumerate() {
            if c.info[1] > 0 {
                c.info[0] = offset((row % n) as u32) as u32;
            }
        }
    }

    /// The draw of card `k` in the cull's draw table (after the meshes').
    pub fn card_draw(&self, k: usize) -> u32 {
        (self.draws.len() + k) as u32
    }

    /// Every draw's index count, the cull's draw table: the mesh levels',
    /// then a quad's (6) per card.
    pub fn index_counts(&self) -> Vec<u32> {
        self.draws
            .iter()
            .map(|d| d.index_count)
            .chain(self.cards.iter().map(|_| 6))
            .collect()
    }
}

/// What `build` reads of the renderer's mesh cache, per mesh: whether it
/// is skinned, its index count and its joints' bind boxes.
pub(crate) struct MeshInfo<'a> {
    pub skinned: bool,
    pub index_count: u32,
    pub joints: &'a [JointBox],
}

/// Checks the scene's crowds and lays them out as tables in `f` (its
/// last contents cleared): their members' records (TAA's previous
/// transforms with them, when `motion`), each (crowd, clip)'s box over its
/// parts and levels, each (crowd, part, level) draw, at the scene's
/// `lod_bias`, and the clips the members play ([`CrowdFrame::played`]).
/// The one pass a frame over the members. Each clip box row's first slot
/// in the baked buffer is written afterwards ([`CrowdFrame::place_clips`]).
/// `hooked`: whether a material moves its vertices (a vertex hook: its
/// members may draw outside their boxes, so nothing occludes them).
#[allow(clippy::too_many_arguments)]
pub(crate) fn build<'m>(
    f: &mut CrowdFrame,
    crowds: &[Crowd],
    clips: &[BakedClip],
    mesh: &dyn Fn(usize) -> Option<MeshInfo<'m>>,
    materials: usize,
    hooked: &dyn Fn(usize) -> bool,
    motion: bool,
    lod_bias: f32,
) -> Result<(), crate::RenderError> {
    use crate::RenderError;
    f.members.clear();
    f.crowds.clear();
    f.clips.clear();
    f.draws.clear();
    f.cards.clear();
    f.previous.clear();
    f.moved = false;
    f.played.clear();
    f.played.resize(clips.len(), false);
    let n: usize = crowds.iter().map(|c| c.members.len()).sum();
    f.members.reserve(n);
    if motion {
        f.previous.reserve(n);
    }
    // A member without a look of its own (most) takes these bits.
    let no_look = crate::instances::fx_bits(&crate::InstanceFx::NONE);
    for (ci, c) in crowds.iter().enumerate() {
        // A crowd with nothing to draw has no row (its draws would be
        // written by its first member).
        if c.parts.is_empty() || c.members.is_empty() {
            continue;
        }
        let levels = c.levels();
        let mesh_levels = c.mesh_levels();
        let draw_base = f.draws.len() as u32;
        for level in 0..mesh_levels {
            for (pi, p) in c.parts.iter().enumerate() {
                if p.material.index() >= materials {
                    return Err(RenderError::InvalidHandle(format!(
                        "crowd {ci} part {pi}: material {}",
                        p.material.index()
                    )));
                }
                let Some(&m) = p.levels.get(level).or(p.levels.last()) else {
                    return Err(RenderError::InvalidData(format!(
                        "crowd {ci} part {pi}: no meshes"
                    )));
                };
                let info = mesh(m.index()).filter(|i| i.skinned).ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "crowd {ci} part {pi}: mesh {} is not a skinned mesh",
                        m.index()
                    ))
                })?;
                f.draws.push(CrowdDraw {
                    crowd: ci as u32,
                    part: pi as u32,
                    level: level as u32,
                    mesh: m.index(),
                    material: p.material.index(),
                    index_count: info.index_count,
                });
            }
        }
        // Every clip's box over the crowd's meshes, for the clips its
        // members play (the others stay empty: never read).
        let clip_base = f.clips.len() as u32;
        f.clips
            .resize(f.clips.len() + clips.len(), ClipGpu::default());
        let boxed = &mut f.clips[clip_base as usize..];
        let first = f.members.len() as u32;
        let mut last_clip = u32::MAX;
        for (mi, m) in c.members.iter().enumerate() {
            let b = m.pose;
            for clip in [b.clip, b.previous_clip] {
                if clip == last_clip {
                    continue;
                }
                let Some(k) = clips.get(clip as usize).filter(|k| {
                    k.joints > 0 && k.frames.len() % k.joints as usize == 0 && k.frame_count() >= 2
                }) else {
                    return Err(RenderError::InvalidData(format!(
                        "crowd {ci} member {mi}: baked clip {clip} of {} (a clip holds whole frames, at least two)",
                        clips.len()
                    )));
                };
                // A row is boxed once (its joints, at least one, mark it).
                if boxed[clip as usize].info[1] == 0 {
                    f.played[clip as usize] = true;
                    let (lo, hi) = crowd_clip_box(c, k, mesh).unwrap_or((Vec3::ZERO, Vec3::ZERO));
                    boxed[clip as usize] = ClipGpu {
                        info: [0, k.joints, k.frame_count() as u32, 0],
                        lo: lo.extend(0.0).to_array(),
                        hi: hi.extend(0.0).to_array(),
                    };
                }
            }
            last_clip = b.clip;
            let [flash, color, tint] = if m.fx == crate::InstanceFx::NONE {
                no_look
            } else {
                crate::instances::fx_bits(&m.fx)
            };
            f.members.push(MemberGpu {
                columns: crate::instances::gpu_columns(&m.transform),
                pose: [
                    b.clip as f32,
                    b.frame,
                    b.previous_clip as f32,
                    b.previous_frame,
                ],
                look: [
                    crate::instances::layer_bits(0.0, m.receive_shadows),
                    flash,
                    color,
                    tint,
                ],
            });
            if motion {
                f.previous.push(b.previous_transform.to_cols_array_2d());
            }
            f.moved |= b.clip != b.previous_clip
                || b.frame != b.previous_frame
                || b.previous_transform != m.transform;
        }
        // Errors only rise along a chain (lod::select's rule).
        let mut errors = [0.0f32; 4];
        for (slot, e) in errors.iter_mut().zip(c.level_errors()) {
            *slot = e;
        }
        let hooked = c.parts.iter().any(|p| hooked(p.material.index()));
        // The card: its clip must be a whole one; its draw is numbered
        // once every crowd's mesh draws are known (below).
        let mut card = [0u32; 3];
        if let Some(i) = &c.impostor {
            let whole = clips.get(i.clip as usize).is_some_and(|k| {
                k.joints > 0 && k.frames.len() % k.joints as usize == 0 && k.frame_count() >= 2
            });
            if !whole {
                return Err(RenderError::InvalidData(format!(
                    "crowd {ci}: its impostor's baked clip {} of {} (a clip holds whole frames, at least two)",
                    i.clip,
                    clips.len()
                )));
            }
            card = [f.cards.len() as u32 + 1, i.clip, i.resolved().1];
            f.cards.push(CrowdCard {
                crowd: ci as u32,
                impostor: *i,
                parts: c
                    .parts
                    .iter()
                    .map(|p| (p.levels[0].index(), p.material.index()))
                    .collect(),
                cast_shadows: c.cast_shadows,
                center: c.center,
                radius: c.radius.max(0.0),
            });
        }
        f.crowds.push(CrowdGpu {
            members: [first, c.members.len() as u32, levels as u32, draw_base],
            info: [
                c.parts.len() as u32,
                c.cast_shadows as u32,
                !hooked as u32,
                clip_base,
            ],
            layers: [c.layers, card[0], card[1], card[2]],
            errors,
            sphere: [c.center.x, c.center.y, c.center.z, c.radius.max(0.0)],
            lod: [lod_bias * c.lod_bias, mesh_levels as f32, 0.0, 0.0],
        });
    }
    // Each card's draw after the mesh draws (`layers.y`: its draw plus one).
    let meshes = f.draws.len() as u32;
    for g in &mut f.crowds {
        if g.layers[1] > 0 {
            g.layers[1] += meshes;
        }
    }
    Ok(())
}

/// The box (min, max) of every vertex of `crowd`'s meshes posed by any
/// frame of `clip`, in the model's space.
fn crowd_clip_box<'m>(
    crowd: &Crowd,
    clip: &BakedClip,
    mesh: &dyn Fn(usize) -> Option<MeshInfo<'m>>,
) -> Option<(Vec3, Vec3)> {
    let mut out: Option<(Vec3, Vec3)> = None;
    let mut seen: Vec<usize> = Vec::new();
    for p in &crowd.parts {
        for m in &p.levels {
            if seen.contains(&m.index()) {
                continue;
            }
            seen.push(m.index());
            let Some(info) = mesh(m.index()) else {
                continue;
            };
            if let Some((lo, hi)) = crate::skin::clip_box(info.joints, clip) {
                out = Some(match out {
                    Some((a, b)) => (a.min(lo), b.max(hi)),
                    None => (lo, hi),
                });
            }
        }
    }
    out
}

/// The skinned record of member `m` of a crowd: posed by its clip's frame
/// blended towards the next, its previous transform at palette slot
/// `previous` (none: 0) and its previous frame for motion vectors. As
/// `crowd_cull.wgsl` `record` writes it.
pub(crate) fn record(
    m: &MemberGpu,
    clips: &[ClipGpu],
    clip_base: u32,
    previous: u32,
) -> SkinnedInstanceGpu {
    let at = |clip: f32, frame: f32| {
        let c = &clips[(clip_base + clip as u32) as usize];
        let (offset, joints, frames) = (c.info[0], c.info[1], c.info[2]);
        let last = frames.saturating_sub(2) as usize;
        let f = if frame.is_finite() {
            frame.max(0.0)
        } else {
            0.0
        };
        let k = (f as usize).min(last);
        let t = (f - k as f32).clamp(0.0, 1.0);
        (
            (offset as usize + k * joints as usize) as f32,
            joints as f32,
            t,
        )
    };
    let (now, joints, t) = at(m.pose[0], m.pose[1]);
    let (then, _, pt) = at(m.pose[2], m.pose[3]);
    SkinnedInstanceGpu {
        transform: m.columns,
        palette: [now, joints, 0.0, -1.0 - t],
        morph0: [0.0; 4],
        morph1: [previous as f32, then, pt, 0.0],
        look: m.look,
    }
}

/// Where frame `frame` of a clip of `clip_frames` frames falls among the
/// `frames` an impostor holds of it, from 0 up to `frames` (wrapping: the
/// clip loops, its last frame its first). As `crowd_cull.wgsl` `card_row`.
pub(crate) fn card_row(frame: f32, clip_frames: u32, frames: u32) -> f32 {
    let f = if frame.is_finite() {
        frame.max(0.0)
    } else {
        0.0
    };
    let span = clip_frames.saturating_sub(1).max(1) as f32;
    (f / span * frames as f32).rem_euclid(frames as f32)
}

/// A card-level member's record (`crowd_card.wgsl` `CardIn`): its
/// transform's columns where a skinned record's are, then its place among
/// the impostor's frames (its sign whether it receives shadows, as
/// [`crate::instances::layer_bits`]) and its look's flash, flash colour
/// and tint. As `crowd_cull.wgsl` `scatter` writes it.
pub(crate) fn card_record(
    m: &MemberGpu,
    clips: &[ClipGpu],
    clip_base: u32,
    frames: u32,
) -> SkinnedInstanceGpu {
    let clip = &clips[(clip_base + m.pose[0] as u32) as usize];
    let row = card_row(m.pose[1], clip.info[2], frames);
    let receive = m.look[0] >= 0.0;
    SkinnedInstanceGpu {
        transform: m.columns,
        palette: [
            crate::instances::layer_bits(row, receive),
            m.look[1],
            m.look[2],
            m.look[3],
        ],
        morph0: [0.0; 4],
        morph1: [0.0; 4],
        look: [0.0; 4],
    }
}

/// The member's transform from its record's columns.
pub(crate) fn member_transform(m: &MemberGpu) -> Mat4 {
    let c = m.columns;
    Mat4::from_cols(
        Vec3::from_array(c[0]).extend(0.0),
        Vec3::from_array(c[1]).extend(0.0),
        Vec3::from_array(c[2]).extend(0.0),
        Vec3::from_array(c[3]).extend(1.0),
    )
}

/// The CPU path's crowds ([`cpu_records`]).
#[derive(Default)]
pub(crate) struct CpuCrowds {
    /// Mesh levels: runs of `(draw, first, count)` over `data`, grouped
    /// (crowd, level, part), and each record's world box.
    pub runs: Vec<(u32, u32, u32)>,
    pub data: Vec<SkinnedInstanceGpu>,
    pub boxes: Vec<(Vec3, Vec3)>,
    /// Cards: runs of `(card, first, count)` over `cards`.
    pub card_runs: Vec<(u32, u32, u32)>,
    pub cards: Vec<SkinnedInstanceGpu>,
}

/// The CPU path's crowd records: each member at its level ([`member_level`],
/// [`drawn_level`]) as one record per part, or a card's record at its
/// impostor's level. `previous_base`: the palette slot of the first
/// member's previous transform (0: none).
pub(crate) fn cpu_records(
    crowds: &[Crowd],
    frame: &CrowdFrame,
    camera: &Camera,
    view_height: f32,
    bias: f32,
    previous_base: Option<u32>,
) -> CpuCrowds {
    let mut out = CpuCrowds::default();
    let CpuCrowds {
        runs,
        data,
        boxes,
        card_runs,
        cards,
    } = &mut out;
    let mut card = 0u32;
    let mut c_gpu = frame.crowds.iter();
    for c in crowds
        .iter()
        .filter(|c| !c.parts.is_empty() && !c.members.is_empty())
    {
        let g = c_gpu.next().expect("a table row per crowd with parts");
        let [first, count, levels, draw_base] = g.members;
        let parts = g.info[0];
        let clip_base = g.info[3];
        let members = &frame.members[first as usize..(first + count) as usize];
        let level: Vec<u32> = c
            .members
            .iter()
            .map(|m| {
                let l = member_level(c, &m.transform, camera, view_height, bias);
                drawn_level(c, m, l) as u32
            })
            .collect();
        let mesh_levels = c.mesh_levels() as u32;
        for l in 0..levels.min(mesh_levels) {
            for p in 0..parts {
                let start = data.len() as u32;
                for (k, m) in members.iter().enumerate() {
                    if level[k] != l {
                        continue;
                    }
                    let previous = previous_base.map_or(0, |b| b + first + k as u32 + 1);
                    data.push(record(m, &frame.clips, clip_base, previous));
                    let clip = &frame.clips[(clip_base + m.pose[0] as u32) as usize];
                    boxes.push(crate::skin::moved_box(
                        &member_transform(m),
                        Vec3::from_slice(&clip.lo[..3]),
                        Vec3::from_slice(&clip.hi[..3]),
                    ));
                }
                let n = data.len() as u32 - start;
                if n > 0 {
                    runs.push((draw_base + l * parts + p, start, n));
                }
            }
        }
        if c.impostor.is_some() {
            let start = cards.len() as u32;
            for (k, m) in members.iter().enumerate() {
                if level[k] == mesh_levels {
                    cards.push(card_record(m, &frame.clips, clip_base, g.layers[3]));
                }
            }
            let n = cards.len() as u32 - start;
            if n > 0 {
                card_runs.push((card, start, n));
            }
            card += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests;

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("crowd cull", crate::crowd::CROWD_CULL_WGSL.to_string());
    add("crowd bake", crate::crowd_bake_source());
}
