//! Render targets (the S8 depth stream, `docs/design.md` §10.8): a texture a
//! [`crate::View`] draws into, sampled like any other by a material map, a
//! sprite or an overlay image (three.js `WebGLRenderTarget` and its
//! `texture`).
//!
//! A target is a texture of the scene's texture table whose source is
//! [`TextureSource::Target`]: [`RenderScene::add_render_target`] adds its
//! colour (and, with [`RenderTargetDesc::depth`], a second texture holding
//! the view's depth), and every handle API takes it as it takes a loaded
//! texture. The renderer allocates it on the GPU as a render attachment,
//! cleared to transparent black until a view first draws into it.
//!
//! - [`TargetFormat::Rgba8`] holds the finished frame of its view (exposure,
//!   post and tone mapping applied, sRGB like a loaded colour texture): a
//!   security monitor, a minimap, a portrait in the UI.
//! - [`TargetFormat::Rgba16Float`] holds the view's linear HDR radiance
//!   before exposure and post: a material that shows it (emissive, unlit)
//!   is exposed and tone mapped once, with the frame it is drawn in (a
//!   mirror, a portal).
//! - The depth texture holds the view's linear view depth from its near to
//!   its far plane as 0 to 1 in red, green and blue (alpha 1): a depth
//!   camera, a soft intersection. It needs the depth prepass, which a view
//!   with a depth target runs.

use crate::TextureHandle;
use crate::scene::{RenderScene, TextureSource};
pub use og_render_data::targets::*;

/// The wgpu formats of a render target's textures.
pub(crate) trait TargetGpu {
    fn wgpu(self) -> wgpu::TextureFormat;
}

impl TargetGpu for TargetFormat {
    fn wgpu(self) -> wgpu::TextureFormat {
        match self {
            TargetFormat::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            TargetFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        }
    }
}

/// A render target's size and storage (three.js `WebGLRenderTarget`'s
/// width, height, type and `depthTexture`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderTargetDesc {
    pub width: u32,
    pub height: u32,
    pub format: TargetFormat,
    /// Keep the view's depth as a second texture ([`RenderTargetHandle::depth`]).
    pub depth: bool,
}

impl Default for RenderTargetDesc {
    fn default() -> Self {
        RenderTargetDesc {
            width: 512,
            height: 512,
            format: TargetFormat::Rgba8,
            depth: false,
        }
    }
}

impl TargetGpu for TargetTexture {
    fn wgpu(self) -> wgpu::TextureFormat {
        match self.kind {
            TargetKind::Color(f) => f.wgpu(),
            TargetKind::Depth => wgpu::TextureFormat::Rgba16Float,
            TargetKind::Stream => wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }
}

/// A render target: its colour texture, and its depth texture when it was
/// made with one. Both are ordinary [`TextureHandle`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderTargetHandle {
    pub color: TextureHandle,
    pub depth: Option<TextureHandle>,
}

impl RenderScene {
    /// Adds a render target: its colour texture (and depth texture) join
    /// the texture table, sampled as any texture is; a [`crate::View`]
    /// naming it draws into them. A side of 0 is taken as 1; a side past
    /// the device's largest texture is refused when the frame uploads it.
    pub fn add_render_target(&mut self, desc: RenderTargetDesc) -> RenderTargetHandle {
        let (width, height) = (desc.width.max(1), desc.height.max(1));
        let color = self.add_texture_source(TextureSource::Target(TargetTexture {
            width,
            height,
            kind: TargetKind::Color(desc.format),
        }));
        let depth = desc.depth.then(|| {
            self.add_texture_source(TextureSource::Target(TargetTexture {
                width,
                height,
                kind: TargetKind::Depth,
            }))
        });
        RenderTargetHandle { color, depth }
    }

    /// The target a texture belongs to, when it is a render target's.
    pub fn target_texture(&self, texture: TextureHandle) -> Option<TargetTexture> {
        if !self.contains_texture(texture) {
            return None;
        }
        match self.textures().get(texture.index())? {
            TextureSource::Target(t) => Some(*t),
            _ => None,
        }
    }
}

/// The GPU texture of a target texture: a render attachment that every
/// pass may sample, cleared to transparent black until drawn.
pub(crate) fn create(device: &wgpu::Device, t: TargetTexture) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(match t.kind {
            TargetKind::Color(_) => "render target",
            TargetKind::Depth => "render target depth",
            TargetKind::Stream => "video stream",
        }),
        size: wgpu::Extent3d {
            width: t.width,
            height: t.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: t.wgpu(),
        // A stream's frames are copied in (queue writes natively, the
        // browser's external image copies on the web).
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | if t.kind == TargetKind::Stream {
                wgpu::TextureUsages::COPY_DST
            } else {
                wgpu::TextureUsages::empty()
            },
        view_formats: &[],
    })
}
