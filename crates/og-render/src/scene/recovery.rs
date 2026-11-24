//! CPU upload copies with a source that can reconstruct them after upload.

use super::*;
use std::sync::{OnceLock, atomic::AtomicBool};

type Reload = Box<dyn Fn() -> Result<TextureSource, String> + Send + Sync>;

pub(super) struct Texture {
    source: OnceLock<Result<TextureSource, String>>,
    reload: Option<Reload>,
    cpu_read: AtomicBool,
    pub size: (u32, u32),
    pub color_space: ColorSpace,
    /// A bounded journal lets independent renderers catch up on sub-image
    /// edits. A renderer behind the journal uploads the complete source.
    updates: Vec<(u64, [u32; 4])>,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("size", &self.size)
            .field("cached", &self.source.get().is_some())
            .field("recoverable", &self.reload.is_some())
            .finish()
    }
}

impl Texture {
    pub fn new(source: TextureSource) -> Self {
        let color_space = match &source {
            TextureSource::Rgba(t) => t.color_space,
            TextureSource::Encoded(t) => t.desc.color_space,
            TextureSource::Target(_) => ColorSpace::Linear,
        };
        Self {
            size: source.size(),
            color_space,
            source: OnceLock::from(Ok(source)),
            reload: None,
            cpu_read: AtomicBool::new(false),
            updates: Vec::new(),
        }
    }

    pub fn update_region(&mut self, revision: u64, rect: [u32; 4], rgba: &[u8]) -> bool {
        let [x, y, w, h] = rect;
        if w == 0
            || h == 0
            || x.saturating_add(w) > self.size.0
            || y.saturating_add(h) > self.size.1
            || rgba.len() != w as usize * h as usize * 4
        {
            return false;
        }
        if self.source().is_err() {
            return false;
        }
        let Some(Ok(TextureSource::Rgba(data))) = self.source.get_mut() else {
            return false;
        };
        if data.mips != Mips::None {
            return false;
        }
        for row in 0..h as usize {
            let to = ((y as usize + row) * data.width as usize + x as usize) * 4;
            let from = row * w as usize * 4;
            data.rgba[to..to + w as usize * 4].copy_from_slice(&rgba[from..from + w as usize * 4]);
        }
        // A released source's reconstruction closure predates these pixels.
        self.reload = None;
        if self.updates.len() == 64 {
            self.updates.remove(0);
        }
        self.updates.push((revision, rect));
        true
    }

    pub fn updated_region(&self, since: u64) -> Option<[u32; 4]> {
        if since < self.updates.first()?.0 - 1 {
            return None;
        }
        self.updates
            .iter()
            .filter(|(r, _)| *r > since)
            .map(|(_, r)| *r)
            .reduce(|a, b| {
                let (x, y) = (a[0].min(b[0]), a[1].min(b[1]));
                [
                    x,
                    y,
                    (a[0] + a[2]).max(b[0] + b[2]) - x,
                    (a[1] + a[3]).max(b[1] + b[3]) - y,
                ]
            })
    }

    pub fn source(&self) -> Result<&TextureSource, String> {
        self.source
            .get_or_init(|| {
                self.reload
                    .as_ref()
                    .expect("evicted textures have a source")()
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    pub fn cpu_source(&self) -> Option<&TextureSource> {
        self.cpu_read.store(true, Ordering::Relaxed);
        self.source().ok()
    }

    pub fn release(
        &mut self,
        reload: impl Fn() -> Result<TextureSource, String> + Send + Sync + 'static,
    ) {
        if self.reload.is_none() {
            self.reload = Some(Box::new(reload));
        }
        // CPU consumers (palette conversion, lighting, paint, etc.) keep
        // what they actually read this frame. Uploads alone do not pin it.
        if !self.cpu_read.swap(false, Ordering::Relaxed) {
            self.source.take();
        }
    }
}

impl RenderScene {
    /// Replaces an RGBA8, non-mipmapped texture rectangle in place. Pixels
    /// are top-row first; renderers upload only changed rectangles and keep
    /// their existing texture bindings. False for invalid data or a handle
    /// that does not name such a texture.
    pub fn set_texture_region(
        &mut self,
        handle: TextureHandle,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> bool {
        if !self.contains_texture(handle) {
            return false;
        }
        let i = handle.index();
        let revision = self.texture_revisions[i] + 1;
        if !self.textures[i].update_region(revision, rect, rgba) {
            return false;
        }
        self.texture_revisions[i] = revision;
        true
    }

    pub(crate) fn texture_updated_region(&self, index: usize, since: u64) -> Option<[u32; 4]> {
        self.textures.get(index)?.updated_region(since)
    }

    /// Forget an uploaded texture's CPU copy without changing its revision,
    /// handle, dimensions or GPU resource. The caller must first confirm its
    /// current revision was uploaded (not deferred). CPU readers retain their
    /// copy while used; a later read or upload reconstructs it with `reload`.
    /// Replacing/removing the texture removes its recovery source too.
    pub fn release_texture_source(
        &mut self,
        handle: TextureHandle,
        reload: impl Fn() -> Result<TextureSource, String> + Send + Sync + 'static,
    ) {
        if self.contains_texture(handle) {
            self.textures[handle.index()].release(reload);
        }
    }

    pub(crate) fn texture_source(
        &self,
        index: usize,
    ) -> Result<&TextureSource, crate::RenderError> {
        self.textures
            .get(index)
            .ok_or_else(|| crate::RenderError::InvalidHandle(format!("texture {index}")))?
            .source()
            .map_err(crate::RenderError::InvalidData)
    }
}
