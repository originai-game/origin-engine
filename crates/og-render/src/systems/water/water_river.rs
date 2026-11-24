//! W5: a river's surface ([`crate::Water::river`]): a ribbon along its
//! centreline, its width across, drawn as a surface over a mesh (uv: u
//! across from the left bank, v down the river), and a flow map of its
//! current (`og_env::river`: along the centreline, slowing to the banks)
//! over that uv. The tick floats bodies on the same record, so a floater
//! drifts with the water it is drawn on.
//!
//! Where the course drops steeper than 45 degrees the ribbon hangs: the
//! water falls there, drawn as a sheet of falling water (water.wgsl), and
//! the pool below churns white: wholly at the fall's foot, fading to clear
//! water by [`churn_length`] downstream (the flow map's b).

use crate::Vertex;
use glam::{Mat4, Vec2, Vec3, Vec3Swizzles};
use og_env::river::{River, bank_profile};
use std::sync::Arc;

/// The flow map's texels across the river.
const ACROSS: u32 = 16;
/// The most rows the flow map has (one per point of the course; a longer
/// course is sampled more coarsely), and the course's sampling (m).
const MAX_ROWS: usize = 4096;
const STEP: f32 = 1.0;
/// A drop over a run steeper than this is a fall (1: 45 degrees).
const FALL_SLOPE: f32 = 1.0;

/// How far below a fall of `drop` metres a river `width` across churns
/// white (m). The plunging jet carries air down into its pool and the
/// bubbles rise and clear over a reach that grows with the jet's speed at
/// impact, so with the fall's height (Chanson, *Air Bubble Entrainment in
/// Free-Surface Turbulent Shear Flows*, 1997, ch. 6: the aerated region of
/// a plunging jet lengthens with its impact velocity): three drops here,
/// and never less than one width (a low step still roils the channel
/// across).
pub(crate) fn churn_length(drop: f32, width: f32) -> f32 {
    (3.0 * drop).max(width)
}

/// A ribbon's geometry and flow map ([`ribbon`]).
#[derive(Debug, PartialEq)]
pub(crate) struct Ribbon {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// rgba8 rows of [`ACROSS`] texels, the first row the river's mouth
    /// (the image's top at v 1: three.js' flipY, as flow maps are read): rg
    /// the current over the river's speed (`* 0.5 + 0.5`), b how churned the
    /// water is below a fall.
    pub flow: Vec<[u8; 4]>,
    pub rows: u32,
}

fn vertex(position: Vec3, normal: Vec3, uv: Vec2) -> Vertex {
    Vertex {
        position: position.to_array(),
        normal: normal.to_array(),
        uv: uv.to_array(),
        color: [1.0; 4],
        tangent: [0.0; 4],
    }
}

/// The river's ribbon in the surface's frame (`inverse`: the surface's
/// frame from the world), and its flow map.
pub(crate) fn ribbon(river: &River, inverse: Mat4) -> Ribbon {
    // The course every metre at most (one vertex pair and one flow row a
    // point), so a fall's tilted normal and its churn stay where the course
    // falls; a course longer than MAX_ROWS metres is sampled more coarsely.
    let points = river.points();
    let length = river.length() as f32;
    let budget = (MAX_ROWS - 1).saturating_sub(points.len() - 1).max(1);
    let step = (length / budget as f32).max(STEP);
    let mut course: Vec<Vec3> = vec![points[0]];
    for w in points.windows(2) {
        let pieces = (w[0].xz().distance(w[1].xz()) / step).ceil().max(1.0) as usize;
        course.extend((1..=pieces).map(|k| w[0].lerp(w[1], k as f32 / pieces as f32)));
    }
    course.truncate(MAX_ROWS);
    let n = course.len();
    let tangent = |i: usize| -> Vec2 {
        let a = course[i.saturating_sub(1)].xz();
        let b = course[(i + 1).min(n - 1)].xz();
        (b - a).normalize_or(Vec2::X)
    };
    let half = river.width * 0.5;
    // How churned the water is at each point: 1 at the foot of a fall,
    // easing to none [`churn_length`] of the whole fall's drop downstream.
    let mut churn = vec![0.0f32; n];
    // Past the last fall's foot: how far (m) and how far it dropped.
    let mut since: Option<(f32, f32)> = None;
    let mut falling = false;
    for i in 1..n {
        let (a, b) = (course[i - 1], course[i]);
        let run = a.xz().distance(b.xz());
        if a.y - b.y > FALL_SLOPE * run {
            // A fall over several points drops their sum.
            let above = if falling {
                since.map_or(0.0, |s| s.1)
            } else {
                0.0
            };
            since = Some((0.0, above + (a.y - b.y)));
            falling = true;
        } else {
            falling = false;
            if let Some(s) = since.as_mut() {
                s.0 += run;
            }
        }
        churn[i] = since.map_or(0.0, |(d, drop)| {
            // Thinning fast past the foot and easing out: (1 - t)², no
            // slope where it ends.
            let t = (d / churn_length(drop, river.width)).clamp(0.0, 1.0);
            (1.0 - t) * (1.0 - t)
        });
    }
    let mut vertices = Vec::with_capacity(2 * n);
    let mut flow = vec![[128u8, 128, 0, 255]; ACROSS as usize * n];
    for (i, c) in course.iter().enumerate() {
        let t = tangent(i);
        // Right of the tangent facing downstream (+across).
        let right = Vec2::new(-t.y, t.x);
        let v = (i as f32 + 0.5) / n as f32;
        // The surface's normal: up, tilted down the river by the steeper of
        // the two stretches it joins (a fall's edge hangs with the fall).
        let (down, along) = [(i.saturating_sub(1), i), (i, (i + 1).min(n - 1))]
            .into_iter()
            .filter(|(a, b)| a != b)
            .map(|(a, b)| {
                let (a, b) = (course[a], course[b]);
                let run = a.xz().distance(b.xz()).max(1e-4);
                ((b.y - a.y) / run, (b.xz() - a.xz()).normalize_or(t))
            })
            .max_by(|x, y| x.0.abs().total_cmp(&y.0.abs()))
            .unwrap_or((0.0, t));
        let normal = Vec3::new(-along.x * down, 1.0, -along.y * down).normalize();
        let world = |s: f32| Vec3::new(c.x + right.x * s, c.y, c.z + right.y * s);
        let local = |p: Vec3| inverse.transform_point3(p);
        let ln = inverse.transform_vector3(normal).normalize_or(Vec3::Y);
        vertices.push(vertex(local(world(-half)), ln, Vec2::new(0.0, v)));
        vertices.push(vertex(local(world(half)), ln, Vec2::new(1.0, v)));
        let row = n - 1 - i;
        for k in 0..ACROSS {
            let across = -1.0 + 2.0 * (k as f32 + 0.5) / ACROSS as f32;
            let f = t * bank_profile(across);
            let byte = |x: f32| ((x * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
            let white = (churn[i] * 255.0).round() as u8;
            flow[row * ACROSS as usize + k as usize] = [byte(f.x), byte(f.y), white, 255];
        }
    }
    let mut indices = Vec::with_capacity(6 * (n - 1));
    for i in 0..(n - 1) as u32 {
        let (l0, r0, l1, r1) = (2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 3);
        indices.extend_from_slice(&[l0, l1, r1, l0, r1, r0]);
    }
    Ribbon {
        vertices,
        indices,
        flow,
        rows: n as u32,
    }
}

/// A river's ribbon on the GPU, for the surface's frame it was built in.
pub(crate) struct RiverMesh {
    pub river: Arc<River>,
    pub model: Mat4,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub count: u32,
    _flow: wgpu::Texture,
    pub flow: wgpu::TextureView,
}

impl RiverMesh {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        river: Arc<River>,
        model: Mat4,
    ) -> RiverMesh {
        use wgpu::util::DeviceExt;
        let r = ribbon(&river, model.inverse());
        let vertices = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("river"),
                contents: bytemuck::cast_slice(&r.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        let indices = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("river"),
                contents: bytemuck::cast_slice(&r.indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        let flow = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("river flow"),
                size: wgpu::Extent3d {
                    width: ACROSS,
                    height: r.rows,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                texture_binding_view_dimension: None,
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&r.flow),
        );
        let view = flow.create_view(&Default::default());
        RiverMesh {
            river,
            model,
            vertices,
            indices,
            count: r.indices.len() as u32,
            _flow: flow,
            flow: view,
        }
    }

    /// Whether this ribbon is `river`'s in the frame `model`.
    pub(crate) fn fits(&self, river: &Arc<River>, model: Mat4) -> bool {
        Arc::ptr_eq(&self.river, river) && self.model == model
    }
}
