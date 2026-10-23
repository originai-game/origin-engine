//! Area lights and light cookies (docs/design.md §2.6).
//!
//! A [`crate::LightKind::Rect`] light is clustered with the point and spot
//! lights and shaded by area.wgsl with linearly transformed cosines (the
//! GGX fit of Heitz, Dupuy, Hill and Neubelt's `ltc_code`, BSD-licensed,
//! `third_party/ltc`, resampled from 64 × 64 to 32 × 32 half floats so the
//! table is one 16 KiB uniform: frame binding 11, no texture slot taken
//! from WebGL2's full skinned pipeline). Its record in the light table:
//! `color_kind.w` -1 one-sided, -2 two-sided; the half-width vector in
//! `direction_slot.xyz` (no shadow slot); the half-height in `cone.xyz`.
//! Shaders that do not know rect lights (terrain, scattered copies, 3D
//! fluids) light them as a point at their centre.
//!
//! Cookies ([`LightCookie`]) take layers of the local shadow array past the
//! shadow layers, each texel the cookie's linear colour packed 8:8:8 in an
//! exact integer (the array is R32Float, read with `textureLoad`, so no
//! texture slot either): one layer per spot cookie, six faces per point
//! cookie (the texture is six square faces stacked top to bottom, +X, -X,
//! +Y, -Y, +Z, -Z). A spot carries its layer in `color_kind.w` (2 + layer),
//! a point its first face in `cone.y` (-(layer + 1)). At most
//! [`MAX_COOKIE_LAYERS`] layers: cookies past them are not drawn.

use crate::frame::LightGpu;
use crate::scene::{ColorSpace, LightKind, RenderScene, TextureHandle};

/// A light's cookie: the light by index into [`RenderScene::lights`] and
/// the texture projected from it (a point light's: six faces stacked).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightCookie {
    pub light: usize,
    pub texture: TextureHandle,
}

/// The most local shadow array layers cookies take.
pub const MAX_COOKIE_LAYERS: usize = 24;

const LTC: &[u8; 16384] = include_bytes!("../../../third_party/ltc/ltc32.bin");

/// The LTC table's uniform buffer (frame binding 11).
pub(crate) fn ltc_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("ltc table"),
            contents: LTC,
            usage: wgpu::BufferUsages::UNIFORM,
        },
    )
}

pub(crate) fn ltc_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 11,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// One cookie's place in the local shadow array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Placed {
    texture: TextureHandle,
    point: bool,
    /// Its first layer past the shadow layers.
    offset: u32,
    /// What was uploaded: the texels' address and size.
    source: (usize, u32, u32),
}

/// The cookies placed in the local shadow array.
#[derive(Default)]
pub(crate) struct Cookies {
    placed: Vec<Placed>,
    /// The array they were written to (its first layer) and its identity.
    written: Option<(u32, usize)>,
}

impl Cookies {
    /// The layers this frame's cookies need.
    pub fn layers(scene: &RenderScene) -> u32 {
        let mut seen: Vec<(TextureHandle, bool)> = Vec::new();
        let mut layers = 0u32;
        for c in &scene.light_cookies {
            let Some(light) = scene.lights.get(c.light) else {
                continue;
            };
            let point = match light.kind {
                LightKind::Point => true,
                LightKind::Spot { .. } => false,
                LightKind::Rect { .. } => continue,
            };
            if seen.contains(&(c.texture, point)) || scene.texture(c.texture).is_none() {
                continue;
            }
            let need = if point { 6 } else { 1 };
            if layers + need > MAX_COOKIE_LAYERS as u32 {
                continue;
            }
            seen.push((c.texture, point));
            layers += need;
        }
        layers
    }

    /// Places this frame's cookies from layer `base` of `texture` (an
    /// R32Float array `size` texels square) and writes those not already
    /// there. `id` names the array: a new array is written afresh.
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        texture: &wgpu::Texture,
        base: u32,
        size: u32,
    ) {
        let id = texture as *const wgpu::Texture as usize;
        if self.written != Some((base, id)) {
            self.placed.clear();
            self.written = Some((base, id));
        }
        let mut placed: Vec<Placed> = Vec::new();
        let mut next = 0u32;
        for c in &scene.light_cookies {
            let Some(light) = scene.lights.get(c.light) else {
                continue;
            };
            let point = match light.kind {
                LightKind::Point => true,
                LightKind::Spot { .. } => false,
                LightKind::Rect { .. } => continue,
            };
            if placed
                .iter()
                .any(|p| p.texture == c.texture && p.point == point)
            {
                continue;
            }
            let Some(data) = scene.texture(c.texture) else {
                continue;
            };
            let need = if point { 6 } else { 1 };
            if next + need > MAX_COOKIE_LAYERS as u32 {
                continue;
            }
            let p = Placed {
                texture: c.texture,
                point,
                offset: next,
                source: (data.rgba.as_ptr() as usize, data.width, data.height),
            };
            next += need;
            if !self.placed.contains(&p) {
                let faces = if point { 6 } else { 1 };
                let face_h = (data.height / faces).max(1);
                for face in 0..faces {
                    let texels = pack_face(
                        &data.rgba,
                        data.width,
                        face_h,
                        face * face_h,
                        data.color_space,
                        size,
                    );
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: base + p.offset + face,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        bytemuck::cast_slice(&texels),
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(size * 4),
                            rows_per_image: Some(size),
                        },
                        wgpu::Extent3d {
                            width: size,
                            height: size,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
            placed.push(p);
        }
        self.placed = placed;
    }

    /// Marks the records of lights with a placed cookie (`kept`: the
    /// scene's light index of each record) with their layers.
    pub fn patch(&self, scene: &RenderScene, kept: &[usize], gpu: &mut [LightGpu], base: u32) {
        if self.placed.is_empty() {
            return;
        }
        for c in &scene.light_cookies {
            let Some(k) = kept.iter().position(|&i| i == c.light) else {
                continue;
            };
            let point = matches!(scene.lights[c.light].kind, LightKind::Point);
            let Some(p) = self
                .placed
                .iter()
                .find(|p| p.texture == c.texture && p.point == point)
            else {
                continue;
            };
            let layer = (base + p.offset) as f32;
            let g = &mut gpu[k];
            if point {
                g.cone[1] = -(layer + 1.0);
            } else {
                g.color_kind[3] = 2.0 + layer;
            }
        }
    }
}

/// One face (`rows` rows from `top`) of an RGBA8 image resampled
/// bilinearly to `size` square, each texel's linear colour packed 8:8:8.
fn pack_face(
    rgba: &[u8],
    width: u32,
    rows: u32,
    top: u32,
    space: ColorSpace,
    size: u32,
) -> Vec<f32> {
    let lut: Vec<f32> = (0..256)
        .map(|v| {
            let c = v as f32 / 255.0;
            match space {
                ColorSpace::Srgb if c <= 0.04045 => c / 12.92,
                ColorSpace::Srgb => ((c + 0.055) / 1.055).powf(2.4),
                _ => c,
            }
        })
        .collect();
    let texel = |x: u32, y: u32, k: usize| {
        let i = (((top + y.min(rows - 1)) * width + x.min(width - 1)) * 4) as usize + k;
        rgba.get(i).map_or(0.0, |&v| lut[v as usize])
    };
    let mut out = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5) / size as f32 * width as f32 - 0.5;
            let v = (y as f32 + 0.5) / size as f32 * rows as f32 - 0.5;
            let (x0, y0) = (u.floor().max(0.0) as u32, v.floor().max(0.0) as u32);
            let (fx, fy) = (
                (u - u.floor()).clamp(0.0, 1.0),
                (v - v.floor()).clamp(0.0, 1.0),
            );
            let mut packed = 0u32;
            for k in 0..3 {
                let a = texel(x0, y0, k) * (1.0 - fx) + texel(x0 + 1, y0, k) * fx;
                let b = texel(x0, y0 + 1, k) * (1.0 - fx) + texel(x0 + 1, y0 + 1, k) * fx;
                let c = (a * (1.0 - fy) + b * fy).clamp(0.0, 1.0);
                packed = (packed << 8) | (c * 255.0 + 0.5) as u32;
            }
            out.push(packed as f32);
        }
    }
    out
}

/// A rect light's record (`frame::local_lights`).
pub(crate) fn rect_record(
    position: glam::Vec3,
    range: f32,
    radiance: glam::Vec3,
    half_width: glam::Vec3,
    half_height: glam::Vec3,
    two_sided: bool,
) -> LightGpu {
    LightGpu {
        position_range: position.extend(range).to_array(),
        color_kind: radiance
            .extend(if two_sided { -2.0 } else { -1.0 })
            .to_array(),
        direction_slot: half_width.extend(-1.0).to_array(),
        cone: half_height.extend(0.0).to_array(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_linear_colour_exactly() {
        // A 2 × 2 linear image resampled to 2 × 2 keeps its texels.
        let rgba = [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 10, 20, 30, 255,
        ];
        let out = pack_face(&rgba, 2, 2, 0, ColorSpace::Linear, 2);
        assert_eq!(out[0] as u32, 0xff0000);
        assert_eq!(out[1] as u32, 0x00ff00);
        assert_eq!(out[3] as u32, (10 << 16) | (20 << 8) | 30);
        // Exact in an f32.
        assert_eq!((out[3] as u32) as f32, out[3]);
    }

    #[test]
    fn ltc_table_starts_with_the_identity() {
        // The first texel of ltc_1 is (1, 0, 0, 2e-5): a, b = 1, 0.
        let a = u16::from_le_bytes([LTC[0], LTC[1]]);
        assert_eq!(a, 0x3c00);
    }
}
