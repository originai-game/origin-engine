//! Fog of war over the lit image: the cells of an `og_env::sight` map as a
//! texture, and the effect pass that dims the ground a team has only
//! explored and covers what it has never seen (fxpost.wgsl `fs_sight`).
//! The map is world state; the renderer draws the levels it is given and
//! keeps nothing of its own but their upload.

use glam::{Vec2, Vec3};
use std::sync::Arc;

/// The plane a fog of war's map lies in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SightPlane {
    /// The ground (x, z): each pixel's surface, from the prepass depth.
    #[default]
    Ground,
    /// The 2D world's plane (x, y): where each pixel's ray meets z = 0.
    Flat,
}

/// A fog of war drawn this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct FogOfWar {
    /// The map's min corner in its plane, from the frame's origin.
    pub min: Vec2,
    /// A cell's side (m), and the cells along each axis.
    pub cell: f32,
    pub columns: u32,
    pub rows: u32,
    /// Each cell's level, row by row from the min corner
    /// (`og_env::sight::{UNSEEN, EXPLORED, VISIBLE}`).
    pub levels: Arc<[u8]>,
    pub plane: SightPlane,
    /// What covers the unseen (linear).
    pub color: Vec3,
    /// How much of the scene shows where the team has been and no longer
    /// sees, 0..1.
    pub explored: f32,
}

impl FogOfWar {
    /// Whether the map has cells to draw.
    pub(crate) fn drawn(&self) -> bool {
        self.columns > 0
            && self.rows > 0
            && self.cell > 0.0
            && self.levels.len() == (self.columns * self.rows) as usize
    }

    /// `FxPost.sight` (fxpost_uniform.wgsl).
    pub(crate) fn rows(&self) -> [[f32; 4]; 3] {
        [
            [
                self.min.x,
                self.min.y,
                1.0 / (self.columns as f32 * self.cell),
                1.0 / (self.rows as f32 * self.cell),
            ],
            [
                self.color.x,
                self.color.y,
                self.color.z,
                self.explored.clamp(0.0, 1.0),
            ],
            [
                1.0,
                f32::from(u8::from(self.plane == SightPlane::Flat)),
                0.0,
                0.0,
            ],
        ]
    }
}

/// The map's texture: a texel per cell, 0 unseen, a half explored, 1 seen.
pub(crate) struct SightTexture {
    pub view: wgpu::TextureView,
    texture: wgpu::Texture,
    /// Counts the texture's remakes: what binds its view rebinds.
    pub generation: u64,
    /// The levels it holds.
    held: Option<Arc<[u8]>>,
}

fn texture(device: &wgpu::Device, columns: u32, rows: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fog of war"),
        size: wgpu::Extent3d {
            width: columns,
            height: rows,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// A cell's texel.
fn texel(level: u8) -> u8 {
    match level {
        og_env::sight::UNSEEN => 0,
        og_env::sight::EXPLORED => 128,
        _ => 255,
    }
}

impl SightTexture {
    pub(crate) fn new(device: &wgpu::Device) -> SightTexture {
        let texture = texture(device, 1, 1);
        SightTexture {
            view: texture.create_view(&Default::default()),
            texture,
            generation: 0,
            held: None,
        }
    }

    /// Holds `fog`'s levels: remade for another grid, written when they
    /// changed.
    pub(crate) fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, fog: &FogOfWar) {
        let size = self.texture.size();
        if (size.width, size.height) != (fog.columns, fog.rows) {
            self.texture = texture(device, fog.columns, fog.rows);
            self.view = self.texture.create_view(&Default::default());
            self.generation += 1;
            self.held = None;
        }
        let same = self
            .held
            .as_ref()
            .is_some_and(|h| Arc::ptr_eq(h, &fog.levels) || **h == *fog.levels);
        if same {
            return;
        }
        let texels: Vec<u8> = fog.levels.iter().map(|&l| texel(l)).collect();
        queue.write_texture(
            self.texture.as_image_copy(),
            &texels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(fog.columns),
                rows_per_image: Some(fog.rows),
            },
            wgpu::Extent3d {
                width: fog.columns,
                height: fog.rows,
                depth_or_array_layers: 1,
            },
        );
        self.held = Some(fog.levels.clone());
    }
}
