//! 2D-array textures ([`RenderScene::add_texture_array`]): N layers of one
//! size, encoding and mip count uploaded as one `D2Array` texture (og-cook's
//! `type texture-array`). RGBA8 layers get a box-filtered mip chain made
//! here (deterministic, the same on every backend); encoded layers bring
//! their own levels.
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
    /// The layers, as many as the GPU texture holds.
    pub layers: u32,
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
        let per_layer: u64 = (0..levels)
            .map(|l| format.level_bytes(width, height, l) as u64)
            .sum();
        Ok(TextureArrayInfo {
            width,
            height,
            layers,
            levels,
            format,
            color_space,
            bytes: per_layer * u64::from(layers),
        })
    }

    /// Every level of every layer, layer after layer (wgpu's
    /// `TextureDataOrder::LayerMajor`).
    pub fn texels(&self, info: &TextureArrayInfo) -> Vec<u8> {
        let per_layer: Vec<Vec<u8>> = self
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
