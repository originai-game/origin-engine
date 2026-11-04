//! W5: a river's surface ([`crate::Water::river`]): a ribbon along its
//! centreline, its width across, drawn as a surface over a mesh (uv: u
//! across from the left bank, v down the river), and a flow map of its
//! current (`og_core::river`: along the centreline, slowing to the banks)
//! over that uv. The tick floats bodies on the same record, so a floater
//! drifts with the water it is drawn on.
//!
//! Where the course drops steeper than 45 degrees the ribbon hangs: the
//! water falls there, drawn as a sheet of falling water (water.wgsl), and
//! the pool below churns white, fading over some three widths downstream
//! (the flow map's b).

use crate::Vertex;
use glam::{Mat4, Vec2, Vec3, Vec3Swizzles};
use og_core::river::{River, bank_profile};
use std::sync::Arc;

/// The flow map's texels across the river.
const ACROSS: u32 = 16;
/// The most rows the flow map has (one per centreline point; a longer
/// course is resampled).
const MAX_ROWS: usize = 4096;
/// A drop over a run steeper than this is a fall (1: 45 degrees).
const FALL_SLOPE: f32 = 1.0;

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
    let points = river.points();
    let n = points.len().min(MAX_ROWS);
    // Evenly resampled by index when the course has more points than rows.
    let at = |i: usize| -> Vec3 {
        if points.len() <= MAX_ROWS {
            return points[i];
        }
        let f = i as f32 * (points.len() - 1) as f32 / (n - 1) as f32;
        let k = (f as usize).min(points.len() - 2);
        points[k].lerp(points[k + 1], f - k as f32)
    };
    let course: Vec<Vec3> = (0..n).map(at).collect();
    let tangent = |i: usize| -> Vec2 {
        let a = course[i.saturating_sub(1)].xz();
        let b = course[(i + 1).min(n - 1)].xz();
        (b - a).normalize_or(Vec2::X)
    };
    let half = river.width * 0.5;
    // How churned the water is at each point: 1 at the foot of a fall,
    // fading over three widths downstream.
    let mut churn = vec![0.0f32; n];
    let mut since: Option<f32> = None;
    for i in 1..n {
        let (a, b) = (course[i - 1], course[i]);
        let run = a.xz().distance(b.xz());
        if a.y - b.y > FALL_SLOPE * run {
            since = Some(0.0);
        } else if let Some(d) = since.as_mut() {
            *d += run;
        }
        churn[i] = since.map_or(0.0, |d| (-d / (3.0 * river.width)).exp());
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
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("river"),
            contents: bytemuck::cast_slice(&r.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("river"),
            contents: bytemuck::cast_slice(&r.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A river east then north: its ribbon's edges lie half its width
    /// either side of the centreline in the surface's frame, u from the left
    /// bank to the right and v down the river; its flow map runs along the
    /// course at every row, slowest at the banks; a drop steeper than 45°
    /// tilts the surface's normal down the river.
    #[test]
    fn the_ribbon_follows_the_course_and_its_flow_runs_down_it() {
        let river = River::new(
            &[
                Vec3::new(0.0, 2.0, 0.0),
                Vec3::new(10.0, 2.0, 0.0),
                Vec3::new(12.0, -2.0, 0.0),
                Vec3::new(12.0, -2.0, -10.0),
            ],
            4.0,
            3.0,
        )
        .expect("a river");
        let model = Mat4::from_translation(Vec3::new(5.0, 1.0, 0.0));
        let r = ribbon(&river, model.inverse());
        assert_eq!(r.vertices.len(), 8);
        assert_eq!(r.indices.len(), 18);
        // The source: left bank at -z (facing east, right is +z), in the frame 5 m west and 1 m down.
        assert_eq!(r.vertices[0].position, [-5.0, 1.0, -2.0]);
        assert_eq!(r.vertices[1].position, [-5.0, 1.0, 2.0]);
        assert_eq!(r.vertices[0].uv, [0.0, 0.125]);
        assert_eq!(r.vertices[7].uv, [1.0, 0.875]);
        // The mouth runs north (-z): its right bank is +x.
        assert_eq!(r.vertices[7].position, [9.0, -3.0, -10.0]);
        // Row 0 is the mouth (the image's top): flow -z, rg (0.5, 0); the source's row (3) +x.
        let mid = (ACROSS / 2) as usize;
        let mouth = r.flow[mid];
        assert_eq!(mouth[0], 128);
        assert!(mouth[1] < 10, "{mouth:?}");
        let source = r.flow[3 * ACROSS as usize + mid];
        assert!(source[0] > 245 && source[1] == 128, "{source:?}");
        let bank = r.flow[3 * ACROSS as usize];
        assert!(
            bank[0] < source[0] - 40,
            "{bank:?} at the bank against {source:?} mid-stream"
        );
        // The fall (points 1 and 2 drop 4 m over 2 m): normals tilted east.
        let fall = r.vertices[4].normal;
        assert!(fall[0] > 0.5 && fall[1] < 0.8, "{fall:?}");
        assert!((r.vertices[0].normal[1] - 1.0).abs() < 1e-6);
        // Churned white at the fall's foot (point 2, row 1), calmer 10 m on (point 3, row 0), none above it.
        let foot = r.flow[ACROSS as usize + mid][2];
        assert_eq!(foot, 255);
        assert!(mouth[2] > 0 && mouth[2] < 120, "{mouth:?}");
        assert_eq!(source[2], 0);
    }
}
