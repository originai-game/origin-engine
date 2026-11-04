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

use crate::scene::{Camera, MaterialHandle, MeshHandle};
use crate::skin::{BakedClip, BakedPose, JointBox, SkinnedInstanceGpu};
use glam::{Mat4, Vec3};

/// The crowd cull (`shaders/crowd_cull.wgsl`) with the instance helpers and
/// the shared depth pyramid test it reads.
pub(crate) const CROWD_CULL_WGSL: &str = concat!(
    wgsl!("instance.wgsl"),
    wgsl!("crowd_cull.wgsl"),
    wgsl!("cull_hiz.wgsl")
);

/// The most levels a crowd draws (`Crowd::errors`).
pub const MAX_CROWD_LEVELS: usize = 4;

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
            members: Vec::new(),
        }
    }
}

impl Crowd {
    /// How many levels it draws (1 to [`MAX_CROWD_LEVELS`]).
    pub fn levels(&self) -> usize {
        self.errors.len().clamp(1, MAX_CROWD_LEVELS)
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
    let scale = max_scale(transform);
    let per_unit = crate::lod::pixels_per_unit(
        camera,
        transform.transform_point3(crowd.center),
        crowd.radius * scale,
        view_height,
    );
    crate::lod::select(
        &crowd.errors[..levels],
        per_unit * scale,
        bias * crowd.lod_bias,
        None,
    )
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
    /// Its first member, members, levels, its first draw.
    pub members: [u32; 4],
    /// Its parts, whether it casts shadows, may be occluded (no material
    /// vertex hook), its first clip box (`clip_base + clip`), then its layers.
    pub info: [u32; 4],
    pub layers: [u32; 4],
    /// Its levels' errors, each at least the one before, times its bias.
    pub errors: [f32; 4],
    /// Its sphere's centre and radius.
    pub sphere: [f32; 4],
    /// x: the scene's level of detail bias times its own.
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

/// The frame's crowds as tables (`build`).
#[derive(Default)]
pub(crate) struct CrowdFrame {
    pub members: Vec<MemberGpu>,
    pub crowds: Vec<CrowdGpu>,
    pub clips: Vec<ClipGpu>,
    pub draws: Vec<CrowdDraw>,
    /// Each member's previous transform (TAA and motion blur only), in
    /// member order: the palette slots after the frame's others.
    pub previous: Vec<[[f32; 4]; 4]>,
    /// Whether any member moved or played on since the frame before.
    pub moved: bool,
}

impl CrowdFrame {
    pub fn len(&self) -> u32 {
        self.members.len() as u32
    }
}

/// What `build` reads of the renderer's mesh cache, per mesh: whether it
/// is skinned, its index count and its joints' bind boxes.
pub(crate) struct MeshInfo<'a> {
    pub skinned: bool,
    pub index_count: u32,
    pub joints: &'a [JointBox],
}

/// Checks the scene's crowds and lays them out as tables: their members'
/// records (TAA's previous transforms with them, when `motion`), each
/// (crowd, clip)'s box over its parts and levels, and each (crowd, part,
/// level) draw, at the scene's `lod_bias`. `offset` is a clip's first slot
/// in the baked buffer;
/// `hooked`, whether a material moves its vertices (a vertex hook: its
/// members may draw outside their boxes, so nothing occludes them).
#[allow(clippy::too_many_arguments)]
pub(crate) fn build<'m>(
    crowds: &[Crowd],
    clips: &[BakedClip],
    mesh: &dyn Fn(usize) -> Option<MeshInfo<'m>>,
    materials: usize,
    hooked: &dyn Fn(usize) -> bool,
    offset: &dyn Fn(u32) -> usize,
    motion: bool,
    lod_bias: f32,
) -> Result<CrowdFrame, crate::RenderError> {
    use crate::RenderError;
    let mut f = CrowdFrame::default();
    let n: usize = crowds.iter().map(|c| c.members.len()).sum();
    f.members.reserve(n);
    if motion {
        f.previous.reserve(n);
    }
    for (ci, c) in crowds.iter().enumerate() {
        // A crowd with nothing to draw has no row (its draws would be
        // written by its first member).
        if c.parts.is_empty() || c.members.is_empty() {
            continue;
        }
        let levels = c.levels();
        let draw_base = f.draws.len() as u32;
        for level in 0..levels {
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
        let mut boxed = vec![false; clips.len()];
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
                if !boxed[clip as usize] {
                    boxed[clip as usize] = true;
                    let (lo, hi) = crowd_clip_box(c, k, mesh).unwrap_or((Vec3::ZERO, Vec3::ZERO));
                    f.clips[(clip_base + clip) as usize] = ClipGpu {
                        info: [offset(clip) as u32, k.joints, k.frame_count() as u32, 0],
                        lo: lo.extend(0.0).to_array(),
                        hi: hi.extend(0.0).to_array(),
                    };
                }
            }
            last_clip = b.clip;
            let [flash, color, tint] = crate::instances::fx_bits(&m.fx);
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
        let mut e = 0.0f32;
        for (l, slot) in errors.iter_mut().enumerate().take(levels) {
            e = e.max(c.errors.get(l).copied().unwrap_or(0.0).max(0.0));
            *slot = e;
        }
        let hooked = c.parts.iter().any(|p| hooked(p.material.index()));
        f.crowds.push(CrowdGpu {
            members: [first, c.members.len() as u32, levels as u32, draw_base],
            info: [
                c.parts.len() as u32,
                c.cast_shadows as u32,
                !hooked as u32,
                clip_base,
            ],
            layers: [c.layers, 0, 0, 0],
            errors,
            sphere: [c.center.x, c.center.y, c.center.z, c.radius.max(0.0)],
            lod: [lod_bias * c.lod_bias, 0.0, 0.0, 0.0],
        });
    }
    Ok(f)
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

/// The CPU path's crowd records: each member at its level ([`member_level`])
/// as one record per part, grouped (crowd, level, part) into runs of
/// `(draw, first, count)` over the records, with each record's world box.
/// `previous_base`: the palette slot of the first member's previous
/// transform (0: none).
#[allow(clippy::type_complexity)]
pub(crate) fn cpu_records(
    crowds: &[Crowd],
    frame: &CrowdFrame,
    camera: &Camera,
    view_height: f32,
    bias: f32,
    previous_base: Option<u32>,
) -> (
    Vec<(u32, u32, u32)>,
    Vec<SkinnedInstanceGpu>,
    Vec<(Vec3, Vec3)>,
) {
    let mut runs = Vec::new();
    let mut data = Vec::new();
    let mut boxes = Vec::new();
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
            .map(|m| member_level(c, &m.transform, camera, view_height, bias) as u32)
            .collect();
        for l in 0..levels {
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
    }
    (runs, data, boxes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Projection;
    use crate::skin::SkinData;

    /// A two-joint clip: joint 1 lifted by `f` metres at frame `f`.
    fn lift_clip(frames: usize) -> BakedClip {
        BakedClip {
            joints: 2,
            frames: (0..frames)
                .flat_map(|f| [Mat4::IDENTITY, Mat4::from_translation(Vec3::Y * f as f32)])
                .collect(),
        }
    }

    fn crowd_of(members: Vec<CrowdMember>, levels: usize) -> Crowd {
        Crowd {
            parts: vec![CrowdPart {
                levels: (0..levels).map(|l| MeshHandle::from_index(l)).collect(),
                material: MaterialHandle::from_index(0),
            }],
            errors: [0.0, 0.02, 0.08][..levels].to_vec(),
            center: Vec3::new(0.0, 1.0, 0.0),
            radius: 1.0,
            members,
            ..Default::default()
        }
    }

    /// Each mesh: a vertex at the origin on joint 0, one at (0, 1, 0) on joint 1.
    fn joints() -> Vec<JointBox> {
        let skin = SkinData {
            joints: vec![[0, 0, 0, 0], [1, 0, 0, 0]],
            weights: vec![[1.0, 0.0, 0.0, 0.0]; 2],
        };
        crate::skin::joint_boxes(&[[0.0; 3], [0.0, 1.0, 0.0]], &skin)
    }

    fn frame_of(
        crowds: &[Crowd],
        clips: &[BakedClip],
        boxes: &[JointBox],
        motion: bool,
    ) -> CrowdFrame {
        build(
            crowds,
            clips,
            &|_| {
                Some(MeshInfo {
                    skinned: true,
                    index_count: 36,
                    joints: boxes,
                })
            },
            1,
            &|_| false,
            &|c| 100 * c as usize + 7,
            motion,
            1.0,
        )
        .expect("a valid crowd")
    }

    #[test]
    fn a_member_s_level_is_the_model_rule_at_its_distance() {
        let camera = Camera {
            position: Vec3::ZERO,
            projection: Projection::Perspective {
                fov_y_degrees: 50.0,
            },
            ..Camera::default()
        };
        let crowd = crowd_of(Vec::new(), 3);
        for d in [3.0f32, 12.0, 40.0, 160.0] {
            let at = Mat4::from_translation(Vec3::new(0.0, -1.0, -d));
            let per_unit =
                crate::lod::pixels_per_unit(&camera, Vec3::new(0.0, 0.0, -d), 1.0, 720.0);
            let want = crate::lod::select(&crowd.errors, per_unit, 1.0, None);
            assert_eq!(
                member_level(&crowd, &at, &camera, 720.0, 1.0),
                want,
                "at {d} m"
            );
        }
        // Near: the full mesh; far: the coarsest. A bias of 0 keeps the full mesh.
        let far = Mat4::from_translation(Vec3::new(0.0, 0.0, -400.0));
        assert_eq!(
            member_level(&crowd, &Mat4::IDENTITY, &camera, 720.0, 1.0),
            0
        );
        assert_eq!(member_level(&crowd, &far, &camera, 720.0, 1.0), 2);
        assert_eq!(member_level(&crowd, &far, &camera, 720.0, 0.0), 0);
        // Scale counts: a member twice the size keeps detail twice as far.
        let mid = Mat4::from_translation(Vec3::new(0.0, 0.0, -100.0));
        let big = mid * Mat4::from_scale(Vec3::splat(2.0));
        assert!(
            member_level(&crowd, &big, &camera, 720.0, 1.0)
                < member_level(&crowd, &mid, &camera, 720.0, 1.0)
        );
    }

    #[test]
    fn cpu_records_group_members_by_level_and_pose_them_as_the_shader_blends() {
        let camera = Camera {
            position: Vec3::new(0.0, 1.0, 0.0),
            ..Camera::default()
        };
        // Members alternating near and far, at frames 0.25, 1.5, 2.75, 3.0.
        let members: Vec<CrowdMember> = (0..4)
            .map(|i| {
                let z = if i % 2 == 0 { -2.0 } else { -300.0 };
                let mut m = CrowdMember::new(
                    Mat4::from_translation(Vec3::Z * z),
                    1,
                    0.25 + 1.25 * i as f32,
                );
                m.pose.previous_frame = 0.5;
                m
            })
            .collect();
        let crowds = vec![crowd_of(members, 3)];
        let clips = vec![lift_clip(2), lift_clip(4)];
        let boxes = joints();
        let f = frame_of(&crowds, &clips, &boxes, true);
        assert_eq!(f.draws.len(), 3);
        assert_eq!(f.previous.len(), 4);
        let (runs, data, bx) = cpu_records(&crowds, &f, &camera, 720.0, 1.0, Some(50));
        // Level 0 holds members 0 and 2 in order, level 2 members 1 and 3.
        assert_eq!(runs, vec![(0, 0, 2), (2, 2, 2)]);
        assert_eq!(data.len(), 4);
        // Clip 1 starts at slot 107 (the offset given), two joints a frame:
        // frame 0.25 is frame 0 a quarter on; 2.75 frame 2 three quarters
        // on; 3.0, the last frame, is the frame before it all the way.
        let slot = |r: &SkinnedInstanceGpu| (r.palette[0], -1.0 - r.palette[3]);
        assert_eq!(slot(&data[0]), (107.0, 0.25));
        assert_eq!(slot(&data[1]), (111.0, 0.75));
        assert_eq!(slot(&data[3]), (111.0, 1.0));
        assert_eq!(data[0].palette[1], 2.0);
        // Previous transforms at slot 50 + member, plus one; previous frame 0.5.
        let prev: Vec<f32> = data.iter().map(|r| r.morph1[0]).collect();
        assert_eq!(prev, vec![51.0, 53.0, 52.0, 54.0]);
        assert_eq!((data[0].morph1[1], data[0].morph1[2]), (107.0, 0.5));
        // Each box holds the member's lifted joint at every frame of the clip.
        for (r, (lo, hi)) in data.iter().zip(&bx) {
            let t = Vec3::from_array(r.transform[3]);
            assert!(lo.y <= t.y && hi.y >= t.y + 1.0 + 3.0, "{lo} {hi}");
        }
        // Without motion: no previous slots, records name none.
        let f = frame_of(&crowds, &clips, &boxes, false);
        let (_, data, _) = cpu_records(&crowds, &f, &camera, 720.0, 1.0, None);
        assert!(f.previous.is_empty() && data.iter().all(|r| r.morph1[0] == 0.0));
    }

    #[test]
    fn a_crowd_box_holds_every_frame_and_a_missing_clip_is_an_error() {
        let boxes = joints();
        let crowds = vec![crowd_of(vec![CrowdMember::new(Mat4::IDENTITY, 0, 0.0)], 1)];
        let f = frame_of(&crowds, &[lift_clip(5)], &boxes, false);
        let c = f.clips[0];
        assert_eq!((c.lo[1], c.hi[1]), (0.0, 5.0));
        assert_eq!(c.info, [7, 2, 5, 0]);
        let bad = vec![crowd_of(vec![CrowdMember::new(Mat4::IDENTITY, 3, 0.0)], 1)];
        let err = build(
            &bad,
            &[lift_clip(5)],
            &|_| None,
            1,
            &|_| false,
            &|_| 0,
            false,
            1.0,
        );
        assert!(err.is_err());
        // An empty crowd has no row.
        let f = frame_of(&[crowd_of(Vec::new(), 1)], &[lift_clip(2)], &boxes, false);
        assert!(f.crowds.is_empty() && f.draws.is_empty());
    }
}
