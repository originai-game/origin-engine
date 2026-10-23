//! 2D-array textures ([`RenderScene::add_texture_array`]): N layers of one
//! size, encoding and mip count uploaded as one `D2Array` texture (og-cook's
//! `type texture-array`). RGBA8 layers get a box-filtered mip chain made
//! here (deterministic, the same on every backend); encoded layers bring
//! their own levels. On WebGL2 an array of one layer, or of a multiple of
//! six, would be taken for a 2D or cube texture, so such an array is
//! padded with copies of its last layer ([`crate::gl_safe_array_layers`]).
//!
//! [`RenderScene::add_texture_array`]: crate::RenderScene::add_texture_array

use crate::scene::{ColorSpace, Mips, TextureSource};
use crate::texture::{TextureDesc, TextureFormat};

pub use crate::scene::TextureArrayHandle;

/// The layers of a texture array, as a host hands them over.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureArrayData {
    pub layers: Vec<TextureSource>,
}

/// What a texture array is on the GPU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureArrayInfo {
    pub width: u32,
    pub height: u32,
    /// The layers asked for (the GPU texture may hold padding past them).
    pub layers: u32,
    /// Layers of the GPU texture.
    pub gpu_layers: u32,
    pub levels: u32,
    pub format: TextureFormat,
    pub color_space: ColorSpace,
    /// Bytes the GPU texture takes, every level of every layer.
    pub bytes: u64,
}

impl TextureArrayData {
    /// Checks the layers agree (size, encoding, colour space, levels) and
    /// says what the GPU texture will be.
    pub fn info(&self) -> Result<TextureArrayInfo, String> {
        let first = self.layers.first().ok_or("a texture array has no layers")?;
        let shape = |s: &TextureSource| -> (u32, u32, TextureFormat, ColorSpace, u32) {
            match s {
                TextureSource::Rgba(t) => (
                    t.width,
                    t.height,
                    TextureFormat::Rgba8,
                    t.color_space,
                    match t.mips {
                        Mips::Generate => TextureDesc::full_chain(t.width, t.height),
                        Mips::None => 1,
                    },
                ),
                // A render target is drawn on the GPU, never a layer.
                TextureSource::Target(t) => {
                    (t.width, t.height, TextureFormat::Rgba8, ColorSpace::Srgb, 1)
                }
                TextureSource::Encoded(u) => (
                    u.desc.width,
                    u.desc.height,
                    u.desc.format,
                    u.desc.color_space,
                    u.desc.mip_levels,
                ),
            }
        };
        let s0 = shape(first);
        for (i, l) in self.layers.iter().enumerate() {
            let s = shape(l);
            if s != s0 {
                return Err(format!(
                    "layer {i} is {}x{} {} with {} levels, layer 0 {}x{} {} with {}",
                    s.0,
                    s.1,
                    s.2.name(),
                    s.4,
                    s0.0,
                    s0.1,
                    s0.2.name(),
                    s0.4
                ));
            }
            match l {
                TextureSource::Rgba(t) if t.rgba.len() != (t.width * t.height * 4) as usize => {
                    return Err(format!(
                        "layer {i}: {} bytes for {}x{}",
                        t.rgba.len(),
                        t.width,
                        t.height
                    ));
                }
                TextureSource::Encoded(u) => u
                    .desc
                    .validate(&u.levels)
                    .map_err(|e| format!("layer {i}: {e}"))?,
                _ => {}
            }
        }
        let (width, height, format, color_space, levels) = s0;
        if width == 0 || height == 0 {
            return Err("a texture array's layers have no texels".into());
        }
        let layers = self.layers.len() as u32;
        let gpu_layers = crate::gl_safe_array_layers(layers);
        let per_layer: u64 = (0..levels)
            .map(|l| format.level_bytes(width, height, l) as u64)
            .sum();
        Ok(TextureArrayInfo {
            width,
            height,
            layers,
            gpu_layers,
            levels,
            format,
            color_space,
            bytes: per_layer * u64::from(gpu_layers),
        })
    }

    /// Every level of every GPU layer, layer after layer (wgpu's
    /// `TextureDataOrder::LayerMajor`), padding layers included.
    pub(crate) fn texels(&self, info: &TextureArrayInfo) -> Vec<u8> {
        let mut per_layer: Vec<Vec<u8>> = self
            .layers
            .iter()
            .map(|l| match l {
                TextureSource::Encoded(u) => u.levels.concat(),
                TextureSource::Target(t) => vec![0; t.width as usize * t.height as usize * 4],
                TextureSource::Rgba(t) => {
                    let mut out = t.rgba.clone();
                    let (mut level, mut w, mut h) = (t.rgba.clone(), t.width, t.height);
                    for _ in 1..info.levels {
                        (level, w, h) = box_down(&level, w, h);
                        out.extend_from_slice(&level);
                    }
                    out
                }
            })
            .collect();
        let last = per_layer.last().cloned().unwrap_or_default();
        per_layer.resize(info.gpu_layers as usize, last);
        per_layer.concat()
    }
}

/// Half an RGBA8 level, each texel the rounded mean of its 2x2 block.
fn box_down(rgba: &[u8], w: u32, h: u32) -> (Vec<u8>, u32, u32) {
    let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let mut sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                    sum += u32::from(rgba[((sy * w + sx) * 4 + c) as usize]);
                }
                out[((y * nw + x) * 4 + c) as usize] = ((sum + 2) / 4) as u8;
            }
        }
    }
    (out, nw, nh)
}

/// A texture array on the GPU (a `D2Array` view of it is what a shader
/// binds).
pub(crate) struct GpuTextureArray {
    /// Held for as long as the scene holds the array (read back natively).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub texture: wgpu::Texture,
    pub info: TextureArrayInfo,
    /// The generation of the scene slot it was uploaded for.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub generation: u8,
}

/// Uploads a checked texture array as one `D2Array` texture.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &TextureArrayData,
    info: TextureArrayInfo,
) -> Result<GpuTextureArray, String> {
    use wgpu::util::DeviceExt;
    let format = info
        .format
        .wgpu(info.color_space)
        .ok_or_else(|| format!("{} is not a GPU format here", info.format.name()))?;
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("texture array"),
            size: wgpu::Extent3d {
                width: info.width,
                height: info.height,
                depth_or_array_layers: info.gpu_layers,
            },
            mip_level_count: info.levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &data.texels(&info),
    );
    Ok(GpuTextureArray {
        texture,
        info,
        generation: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::TextureData;

    fn layer(v: u8, w: u32, h: u32) -> TextureSource {
        TextureSource::Rgba(TextureData {
            width: w,
            height: h,
            rgba: vec![v; (w * h * 4) as usize],
            color_space: ColorSpace::Srgb,
            mips: Mips::Generate,
        })
    }

    #[test]
    fn layers_must_agree_and_awkward_counts_are_padded() {
        let ok = TextureArrayData {
            layers: (0..6).map(|i| layer(i * 40, 8, 4)).collect(),
        };
        let info = ok.info().unwrap();
        assert_eq!((info.layers, info.gpu_layers, info.levels), (6, 7, 4));
        let texels = ok.texels(&info);
        let per = (8 * 4 + 4 * 2 + 2 + 1) * 4;
        assert_eq!(texels.len(), per * 7);
        assert_eq!(texels[per * 6], 200, "the padding repeats the last layer");
        assert_eq!(info.bytes, (per * 7) as u64);
        let one = TextureArrayData {
            layers: vec![layer(1, 4, 4)],
        };
        assert_eq!(one.info().unwrap().gpu_layers, 2);
        let mixed = TextureArrayData {
            layers: vec![layer(1, 4, 4), layer(1, 8, 4)],
        };
        assert!(mixed.info().unwrap_err().contains("layer 1 is 8x4"));
        assert!(TextureArrayData { layers: vec![] }.info().is_err());
    }
}
