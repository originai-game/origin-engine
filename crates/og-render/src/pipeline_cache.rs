//! Compiled pipelines kept across runs where the backend supports it
//! (wgpu's `PIPELINE_CACHE`: Vulkan). A host names a directory with
//! [`set_pipeline_cache_dir`]; each renderer on such an adapter reads
//! `<dir>/<wgpu::util::pipeline_cache_key>` when it starts, hands the cache
//! to every pipeline it makes, and writes it back after its first frame and
//! when it is dropped. Metal, WebGPU and WebGL2 have no such cache in wgpu
//! (their drivers or browsers keep their own); there nothing changes and
//! `RenderInfo::pipeline_cache` is false.

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::sync::Mutex;

    static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

    /// Where renderers made from now on keep compiled pipelines (`None`: nowhere).
    pub fn set_pipeline_cache_dir(dir: Option<PathBuf>) {
        *DIR.lock().unwrap_or_else(|e| e.into_inner()) = dir;
    }

    pub(crate) fn dir() -> Option<PathBuf> {
        DIR.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    thread_local! {
        /// The caches of this thread's devices (a renderer and its device stay
        /// on the thread that made them).
        static CACHES: RefCell<Vec<(wgpu::Device, wgpu::PipelineCache)>> = const { RefCell::new(Vec::new()) };
    }

    /// The pipeline cache of `device`, if its renderer keeps one.
    pub(crate) fn of(device: &wgpu::Device) -> Option<wgpu::PipelineCache> {
        CACHES.with(|c| {
            c.borrow()
                .iter()
                .find(|(d, _)| d == device)
                .map(|(_, cache)| cache.clone())
        })
    }

    /// A renderer's cache and the file it lives in.
    pub(crate) struct DiskCache {
        device: wgpu::Device,
        cache: wgpu::PipelineCache,
        path: PathBuf,
        /// The size of the data last written (nothing new: nothing to write).
        written: usize,
    }

    impl DiskCache {
        /// Opens (or starts) the cache of `device` in `dir` and registers it, when
        /// the device has `PIPELINE_CACHE` and the backend has a cache key.
        pub fn open(
            device: &wgpu::Device,
            info: &wgpu::AdapterInfo,
            dir: PathBuf,
        ) -> Option<DiskCache> {
            if !device.features().contains(wgpu::Features::PIPELINE_CACHE) {
                return None;
            }
            let key = wgpu::util::pipeline_cache_key(info)?;
            let path = dir.join(key);
            let data = std::fs::read(&path).ok();
            // SAFETY: the data is what `get_data` wrote for this adapter (the
            // key names it); `fallback` starts an empty cache when the driver
            // refuses it.
            let cache = unsafe {
                device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    label: Some("og-render pipelines"),
                    data: data.as_deref(),
                    fallback: true,
                })
            };
            CACHES.with(|c| c.borrow_mut().push((device.clone(), cache.clone())));
            Some(DiskCache {
                device: device.clone(),
                cache,
                path,
                written: data.map_or(0, |d| d.len()),
            })
        }

        /// Writes the cache when it grew (to a temporary file, then renamed).
        pub fn save(&mut self) {
            let Some(data) = self.cache.get_data() else {
                return;
            };
            if data.len() == self.written {
                return;
            }
            let tmp = self.path.with_extension("tmp");
            let wrote = self
                .path
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|_| std::fs::write(&tmp, &data))
                .and_then(|_| std::fs::rename(&tmp, &self.path));
            match wrote {
                Ok(()) => self.written = data.len(),
                Err(e) => log::warn!("og-render: pipeline cache {}: {e}", self.path.display()),
            }
        }
    }

    impl Drop for DiskCache {
        fn drop(&mut self) {
            self.save();
            let device = self.device.clone();
            CACHES.with(|c| c.borrow_mut().retain(|(d, _)| *d != device));
        }
    }
}

/// The browser has no wgpu pipeline cache: nothing is kept, and nothing of
/// this reaches the wasm.
#[cfg(target_arch = "wasm32")]
mod web {
    use std::path::PathBuf;

    /// Ignored in the browser.
    pub fn set_pipeline_cache_dir(_dir: Option<PathBuf>) {}

    pub(crate) fn dir() -> Option<PathBuf> {
        None
    }

    pub(crate) fn of(_device: &wgpu::Device) -> Option<wgpu::PipelineCache> {
        None
    }

    pub(crate) struct DiskCache;

    impl DiskCache {
        pub fn open(
            _device: &wgpu::Device,
            _info: &wgpu::AdapterInfo,
            _dir: PathBuf,
        ) -> Option<DiskCache> {
            None
        }

        pub fn save(&mut self) {}
    }
}
