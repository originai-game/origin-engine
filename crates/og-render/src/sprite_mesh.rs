//! The sprite batch's triangles ([`RenderScene::sprite_meshes`]): textured,
//! coloured triangles in the world's xy plane that sort and draw among the
//! sprites of their layer (a 2D skeleton's region and weighted-mesh
//! attachments, og-anim's `og.skeleton2d`). Each mesh is one run in the
//! sprite order (`RunSource::Mesh`, keyed by its layer and sort key as a
//! sprite is), drawn from one vertex buffer of triangle corners through
//! pipeline variants of the sprite module (`shaders/sprite_mesh.wgsl`'s
//! `vs_mesh` with sprite.wgsl's `fs`), so 2D lighting, fog and the sprite
//! texture groups and samplers are the sprites' own. Made on the first
//! frame that draws one, so a renderer that never does pays nothing.

use crate::RenderError;
use crate::effects::SpriteBlend;
use crate::scene::{RenderScene, TextureHandle};
use glam::{Vec2, Vec3};

pub(crate) const SPRITE_MESH_WGSL: &str = wgsl!("sprite_mesh.wgsl");

/// One corner of a [`SpriteMesh`]: a world position, its uv in the mesh's
/// texture (v down, the image's first row is v = 0) and its linear RGBA,
/// multiplied with the texel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteMeshVertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
}

/// Textured triangles drawn in the sprite order: at `layer` (lit by 2D
/// lighting when the layer is in its range, as a sprite there is), sorted
/// within the layer by `key` as [`crate::SpriteOrder`] sorts sprites
/// (smaller first; sprites without an order are at 0), then in submission
/// order. `indices` are triangles into `vertices`, three per triangle.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteMesh {
    /// `None` draws the triangles in their vertex colours.
    pub texture: Option<TextureHandle>,
    pub vertices: Vec<SpriteMeshVertex>,
    pub indices: Vec<u32>,
    pub layer: i32,
    pub key: f32,
    pub blend: SpriteBlend,
}

impl Default for SpriteMesh {
    fn default() -> Self {
        SpriteMesh {
            texture: None,
            vertices: Vec::new(),
            indices: Vec::new(),
            layer: 0,
            key: 0.0,
            blend: SpriteBlend::Alpha,
        }
    }
}

/// `struct MeshIn` in sprite_mesh.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MeshVertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
    lit: f32,
}

const MESH_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<MeshVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4, 3 => Float32],
};

pub(crate) const MESH_STRIDE: u64 = std::mem::size_of::<MeshVertex>() as u64;

/// Each mesh's `(first, count)` corners in [`vertices`]' buffer, in scene
/// order, checked against its vertices and the scene's textures.
pub(crate) fn ranges(
    scene: &RenderScene,
    texture_count: usize,
) -> Result<Vec<(u32, u32)>, RenderError> {
    let mut out = Vec::with_capacity(scene.sprite_meshes.len());
    let mut first = 0u32;
    for (i, m) in scene.sprite_meshes.iter().enumerate() {
        if let Some(t) = m.texture
            && t.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "sprite mesh {i}: texture {}",
                t.index()
            )));
        }
        if m.indices.len() % 3 != 0 {
            return Err(RenderError::InvalidData(format!(
                "sprite mesh {i}: {} indices are not whole triangles",
                m.indices.len()
            )));
        }
        if let Some(bad) = m.indices.iter().find(|&&x| x as usize >= m.vertices.len()) {
            return Err(RenderError::InvalidData(format!(
                "sprite mesh {i}: index {bad} of {} vertices",
                m.vertices.len()
            )));
        }
        let count = m.indices.len() as u32;
        out.push((first, count));
        first += count;
    }
    Ok(out)
}

/// The corners of every mesh's triangles, in scene order (`ranges`).
pub(crate) fn vertices(scene: &RenderScene, lit: impl Fn(i32) -> bool) -> Vec<MeshVertex> {
    let mut out = Vec::new();
    for m in &scene.sprite_meshes {
        let lit = f32::from(u8::from(lit(m.layer)));
        out.extend(m.indices.iter().map(|&i| {
            let v = &m.vertices[i as usize];
            MeshVertex {
                position: v.position.to_array(),
                uv: v.uv.to_array(),
                color: v.color,
                lit,
            }
        }));
    }
    out
}

/// The mesh pipelines (alpha and additive) and their vertex buffer.
pub(crate) struct SpriteMeshRenderer {
    alpha: wgpu::RenderPipeline,
    additive: wgpu::RenderPipeline,
    pub buffer: wgpu::Buffer,
}

impl SpriteMeshRenderer {
    /// `frame_layout`, `sprite_layout` and `fx_layout`: the sprite
    /// pipelines' groups 0 to 2 (the frame, a sprite texture, the light map).
    pub fn new(
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        samples: u32,
    ) -> SpriteMeshRenderer {
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite mesh"),
            bind_group_layouts: &[Some(layouts[0]), Some(layouts[1]), Some(layouts[2])],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite mesh"),
            source: wgpu::ShaderSource::Wgsl(source().into()),
        });
        let buffers = [Some(MESH_LAYOUT)];
        let pipeline = |label, blend| {
            crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    label,
                    layout: &pl,
                    module: &module,
                    vs: "vs_mesh",
                    fs: "fs",
                    buffers: &buffers,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        };
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        SpriteMeshRenderer {
            alpha: pipeline("sprite mesh", wgpu::BlendState::ALPHA_BLENDING),
            additive: pipeline("sprite mesh additive", additive),
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite meshes"),
                size: 4096 * MESH_STRIDE,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    /// Draws `count` corners from `first` with `blend`; the pass has the
    /// frame (0), the mesh's texture group (1) and the light map (2) set.
    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        blend: SpriteBlend,
        first: u32,
        count: u32,
    ) {
        pass.set_pipeline(match blend {
            SpriteBlend::Alpha => &self.alpha,
            SpriteBlend::Additive => &self.additive,
        });
        let start = first as u64 * MESH_STRIDE;
        pass.set_vertex_buffer(
            0,
            self.buffer.slice(start..start + count as u64 * MESH_STRIDE),
        );
        pass.draw(0..count, 0..1);
    }
}

/// The module's source as the renderer compiles it.
pub(crate) fn source() -> String {
    format!(
        "{}{}{}{SPRITE_MESH_WGSL}",
        crate::FRAME_WGSL,
        crate::LIGHTING_WGSL,
        crate::SPRITE_WGSL
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri(layer: i32, n: u32) -> SpriteMesh {
        SpriteMesh {
            vertices: vec![
                SpriteMeshVertex {
                    position: Vec3::ZERO,
                    uv: Vec2::ZERO,
                    color: [1.0; 4],
                };
                3
            ],
            indices: (0..3 * n).map(|i| i % 3).collect(),
            layer,
            ..Default::default()
        }
    }

    #[test]
    fn meshes_take_consecutive_corners_and_bad_indices_are_refused() {
        let mut scene = RenderScene::new();
        scene.sprite_meshes = vec![tri(0, 2), tri(3, 1)];
        assert_eq!(ranges(&scene, 0).unwrap(), [(0, 6), (6, 3)]);
        let v = vertices(&scene, |l| l == 3);
        assert_eq!(v.len(), 9);
        assert_eq!((v[0].lit, v[8].lit), (0.0, 1.0));
        scene.sprite_meshes[1].indices[2] = 3;
        assert!(ranges(&scene, 0).is_err());
        scene.sprite_meshes[1].indices.pop();
        assert!(ranges(&scene, 0).is_err(), "two indices are not a triangle");
    }
}
