//! A texture that shows one frame of footage at a time (og.VideoPlayer,
//! docs/design.md §2.9): the frames are a texture array's layers, held here
//! as the device encodes them, and the scene holds one texture of a frame's
//! size whose contents become the frame shown when it changes. Materials,
//! sprites, the overlay and UI images sample it as any texture, so footage
//! needs no shader of its own and takes one frame of GPU memory, not the
//! whole clip's. Several players of one video each have a texture of their
//! own ([`FrameTexture::view`]): the decoded frames are shared, each view
//! uploads the frame its player shows.
//!
//! A streamed video (decoded as it plays, G3-2) draws into a stream
//! texture instead ([`RenderScene::add_stream_texture`]): a GPU texture of
//! the frame's size, allocated as a render target is (transparent black
//! until its first frame) and written in place as frames are decoded, so a
//! new frame rebinds no material. Natively the host hands each decoded
//! frame's RGBA8 rows to [`RenderScene::write_stream_texture`], uploaded
//! before the next frame draws.

use crate::scene::{RenderScene, TextureFilter, TextureHandle, TextureSource};
use crate::targets::{TargetKind, TargetTexture};
use std::sync::Arc;

/// WGSL module that folds a browser's packed-alpha video frame (colour above,
/// matte below) into an RGBA8 sRGB stream texture.
pub(crate) fn fold_source() -> String {
    concat!(
        wgsl!("common.wgsl"),
        wgsl!("color.wgsl"),
        wgsl!("video_fold.wgsl")
    )
    .to_string()
}

impl RenderScene {
    /// Adds a streamed video's texture, `width` x `height` 8-bit sRGB,
    /// sampled as any texture; `nearest` samples it texel by texel.
    /// `packed_alpha`: the web's decoded frames are packed alpha (`alpha
    /// true`), twice `height` tall with the matte below the colour, folded
    /// into the texture's alpha on the GPU as they are copied. Natively the
    /// host folds them before [`RenderScene::write_stream_texture`].
    pub fn add_stream_texture(
        &mut self,
        width: u32,
        height: u32,
        nearest: bool,
        packed_alpha: bool,
    ) -> TextureHandle {
        let t = self.add_texture_source(TextureSource::Target(TargetTexture {
            width: width.max(1),
            height: height.max(1),
            kind: TargetKind::Stream,
        }));
        if nearest {
            self.set_texture_filter(t, TextureFilter::Nearest);
        }
        #[cfg(target_arch = "wasm32")]
        if packed_alpha {
            self.stream_packed.insert(t.index());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = packed_alpha;
        t
    }

    /// Shows `rgba` (the texture's width x height RGBA8 rows, sRGB) in a
    /// stream texture from the next frame on. False for a handle that is
    /// not a live stream texture or rows of another size.
    pub fn write_stream_texture(&mut self, handle: TextureHandle, rgba: Arc<Vec<u8>>) -> bool {
        let i = handle.index();
        let Ok(TextureSource::Target(t)) = self.texture_source(i) else {
            return false;
        };
        if t.kind != TargetKind::Stream
            || !self.texture_live(i)
            || rgba.len() != t.width as usize * t.height as usize * 4
        {
            return false;
        }
        self.stream_serial += 1;
        let serial = self.stream_serial;
        self.stream_frames.insert(i, (serial, rgba));
        true
    }

    /// On the web: shows a decoded VideoFrame (its display size the
    /// texture's) in a stream texture from the next frame on; the renderer
    /// copies it on the GPU and closes it, as it closes a frame this one
    /// replaces before it was drawn. False (the frame closed) for a handle
    /// that is not a live stream texture.
    #[cfg(target_arch = "wasm32")]
    pub fn write_stream_frame(
        &mut self,
        handle: TextureHandle,
        frame: wgpu::web_sys::VideoFrame,
    ) -> bool {
        let i = handle.index();
        let ok = matches!(self.texture_source(i), Ok(TextureSource::Target(t)) if t.kind == TargetKind::Stream)
            && self.texture_live(i);
        if !ok {
            frame.close();
            return false;
        }
        if let Some(old) = self.stream_external.get_mut().insert(i, frame) {
            old.close();
        }
        true
    }

    /// The web's VideoFrames waiting for their copy (the renderer takes them).
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn stream_external_cell(
        &self,
    ) -> &std::cell::RefCell<std::collections::BTreeMap<usize, wgpu::web_sys::VideoFrame>> {
        &self.stream_external
    }

    /// Each stream texture's newest frame: its serial and its rows.
    pub(crate) fn stream_frames(&self) -> &std::collections::BTreeMap<usize, (u64, Arc<Vec<u8>>)> {
        &self.stream_frames
    }
}
use crate::texture_array::{TextureArrayData, TextureArrayInfo};

/// The frames and the texture that shows one of them.
pub struct FrameTexture {
    /// Shared by every view of the video.
    layers: std::rc::Rc<Vec<TextureSource>>,
    texture: TextureHandle,
    shown: usize,
    info: TextureArrayInfo,
}

impl FrameTexture {
    /// Adds the texture to `scene`, showing frame 0; `nearest` samples it
    /// texel by texel (pixel-art footage). Refused when the frames disagree.
    pub fn new(
        scene: &mut RenderScene,
        frames: TextureArrayData,
        nearest: bool,
    ) -> Result<FrameTexture, String> {
        let info = frames.info()?;
        let texture = scene.add_texture_source(frames.layers[0].clone());
        if nearest {
            scene.set_texture_filter(texture, TextureFilter::Nearest);
        }
        Ok(FrameTexture {
            layers: std::rc::Rc::new(frames.layers),
            texture,
            shown: 0,
            info,
        })
    }

    /// Another texture of the same frames, showing frame 0 (a second
    /// player of the video): its own upload, the frames not copied.
    pub fn view(&self, scene: &mut RenderScene) -> FrameTexture {
        let texture = scene.add_texture_source(self.layers[0].clone());
        let filter = scene.texture_filter(self.texture);
        scene.set_texture_filter(texture, filter);
        FrameTexture {
            layers: self.layers.clone(),
            texture,
            shown: 0,
            info: self.info,
        }
    }

    /// The texture that shows the frame.
    pub fn texture(&self) -> TextureHandle {
        self.texture
    }

    /// The frames' shape and the bytes they take as encoded.
    pub fn info(&self) -> TextureArrayInfo {
        self.info
    }

    /// The frame the texture shows.
    pub fn shown(&self) -> usize {
        self.shown
    }

    /// Shows `frame` (past the last, the last): the texture is uploaded
    /// again before the next frame only when it changed. Whether it did.
    pub fn show(&mut self, scene: &mut RenderScene, frame: usize) -> bool {
        let frame = frame.min(self.layers.len() - 1);
        if frame == self.shown {
            return false;
        }
        let replaced = match &self.layers[frame] {
            TextureSource::Rgba(t) => scene.set_texture(self.texture, t.clone()),
            TextureSource::Encoded(u) => scene.set_texture_upload(self.texture, u.clone()),
            TextureSource::Target(_) => false,
        };
        if replaced {
            self.shown = frame;
        }
        replaced
    }
}
