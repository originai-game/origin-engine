//! A texture that shows one frame of footage at a time (og.VideoPlayer,
//! docs/design.md §2.9): the frames are a texture array's layers, held here
//! as the device encodes them, and the scene holds one texture of a frame's
//! size whose contents become the frame shown when it changes. Materials,
//! sprites, the overlay and UI images sample it as any texture, so footage
//! needs no shader of its own and takes one frame of GPU memory, not the
//! whole clip's. Several players of one video each have a texture of their
//! own ([`FrameTexture::view`]): the decoded frames are shared, each view
//! uploads the frame its player shows.

use crate::scene::{RenderScene, TextureFilter, TextureHandle, TextureSource};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{ColorSpace, Mips, TextureData};

    fn frame(v: u8) -> TextureSource {
        TextureSource::Rgba(TextureData {
            width: 4,
            height: 2,
            rgba: vec![v; 32],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        })
    }

    #[test]
    fn the_texture_is_replaced_only_when_the_frame_changes() {
        let mut scene = RenderScene::new();
        let data = TextureArrayData {
            layers: (0..5).map(|i| frame(i * 50)).collect(),
        };
        let mut v = FrameTexture::new(&mut scene, data, false).unwrap();
        let i = v.texture().index();
        let rev = |s: &RenderScene| s.texture_revisions()[i];
        let r0 = rev(&scene);
        assert!(!v.show(&mut scene, 0), "frame 0 is already shown");
        assert_eq!(rev(&scene), r0);
        assert!(v.show(&mut scene, 3));
        assert_eq!(rev(&scene), r0 + 1);
        assert!(matches!(&scene.textures()[i], TextureSource::Rgba(t) if t.rgba[0] == 150));
        assert!(!v.show(&mut scene, 3));
        // Past the last frame: the last.
        assert!(v.show(&mut scene, 99));
        assert_eq!(v.shown(), 4);
        assert_eq!(v.info().layers, 5);
        // A second view: its own texture, at frame 0 while the first shows 4.
        let mut w = v.view(&mut scene);
        assert_ne!(w.texture(), v.texture());
        assert!(w.show(&mut scene, 2));
        let at = |s: &RenderScene, t: TextureHandle| match &s.textures()[t.index()] {
            TextureSource::Rgba(d) => d.rgba[0],
            _ => 0,
        };
        assert_eq!(at(&scene, w.texture()), 100);
        assert_eq!(at(&scene, v.texture()), 200);
    }
}
