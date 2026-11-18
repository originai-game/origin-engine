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
        }
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
