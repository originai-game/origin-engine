//! Per-engine texture preparation. One worker, not one thread per texture;
//! dropping the engine closes its queue and discards unfinished answers.

use crate::{Hooks, TextureWork};
use og_bundle::runtime::CookedTexture;
use og_render::{TextureFormats, TextureSource};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

type Job = Box<dyn FnOnce() + Send>;
type Answer = (Result<TextureSource, String>, f64, f64);

struct Entry {
    receiver: Receiver<Answer>,
    ready: Option<Result<TextureSource, String>>,
    start: f64,
    cancelled: Arc<AtomicBool>,
}

impl Drop for Entry {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

impl Entry {
    /// Whether texture `id`'s answer has arrived, without copying it.
    fn poll(&mut self, id: &str) -> bool {
        if self.ready.is_none() {
            match self.receiver.try_recv() {
                Ok((answer, offset, ms)) => {
                    crate::author::trace::texture_work(id, self.start, offset, ms);
                    self.ready = Some(answer);
                }
                Err(TryRecvError::Empty) => return false,
                Err(TryRecvError::Disconnected) => {
                    self.ready = Some(Err("texture worker stopped".into()));
                }
            }
        }
        true
    }
}

/// What [`Textures::start`] found or made for a texture.
enum Started {
    /// Its entry is in the cache, its work queued or done.
    Entry,
    /// The host prepares it itself (`Hooks::texture`): no worker, no entry.
    Host,
    Failed(String),
}

#[derive(Default)]
pub(super) struct Textures {
    worker: Option<Sender<Job>>,
    entries: HashMap<(String, u64), Entry>,
}

impl Textures {
    pub(super) fn forget(&mut self, ids: &[String]) {
        self.entries.retain(|(id, _), _| !ids.contains(id));
    }

    pub(super) fn ids(&self) -> Vec<String> {
        self.entries.keys().map(|(id, _)| id.clone()).collect()
    }

    pub(super) fn forget_sources(
        &mut self,
        closed: &[CookedTexture],
        kept: &[CookedTexture],
        formats: Option<&TextureFormats>,
    ) {
        let mut removed: std::collections::HashSet<_> =
            closed.iter().map(|t| key(t, formats)).collect();
        for texture in kept {
            removed.remove(&key(texture, formats));
        }
        self.entries.retain(|key, _| !removed.contains(key));
    }

    pub(super) fn prepare(
        &mut self,
        host: &dyn Hooks,
        texture: &CookedTexture,
        formats: Option<&TextureFormats>,
    ) -> super::Fetch<TextureSource> {
        use super::Fetch;
        // Runtime bundles may use the same id for different maps at once.
        // They must not continually cancel each other's preparation.
        let key = key(texture, formats);
        match self.start(host, texture, formats, &key) {
            Started::Entry => {}
            Started::Host => return host.texture(texture, formats),
            Started::Failed(e) => return Fetch::Failed(e),
        }
        let entry = self.entries.get_mut(&key).unwrap();
        if !entry.poll(&texture.id) {
            return Fetch::Pending;
        }
        match entry.ready.as_ref().unwrap() {
            Ok(source) => Fetch::Ready(source.clone()),
            Err(e) => Fetch::Failed(e.clone()),
        }
    }

    /// Queues `texture`'s work on the worker unless it has an entry.
    fn start(
        &mut self,
        host: &dyn Hooks,
        texture: &CookedTexture,
        formats: Option<&TextureFormats>,
        key: &(String, u64),
    ) -> Started {
        if !self.entries.contains_key(key) {
            let Some(work): Option<TextureWork> = host.texture_work(texture, formats) else {
                return Started::Host;
            };
            if self.worker.is_none() {
                let (send, receive) = mpsc::channel::<Job>();
                match std::thread::Builder::new()
                    .name("og-textures".into())
                    .spawn(move || {
                        while let Ok(job) = receive.recv() {
                            job();
                        }
                    }) {
                    Ok(_) => self.worker = Some(send),
                    Err(e) => return Started::Failed(format!("texture worker: {e}")),
                }
            }
            let (send, receiver) = mpsc::channel();
            let queued = Instant::now();
            let start = crate::author::trace::start();
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancel = cancelled.clone();
            let job = Box::new(move || {
                // A dropped receiver cancels work that has not started yet.
                // Sending a result to a dropped engine never keeps it alive.
                if cancel.load(Ordering::Acquire) {
                    return;
                }
                let offset = queued.elapsed().as_secs_f64() * 1000.0;
                let began = Instant::now();
                let answer = work();
                let ms = began.elapsed().as_secs_f64() * 1000.0;
                let _ = send.send((answer, offset, ms));
            });
            if self.worker.as_ref().unwrap().send(job).is_err() {
                self.worker = None;
                return Started::Failed("texture worker stopped".into());
            }
            self.entries.insert(
                key.clone(),
                Entry {
                    receiver,
                    ready: None,
                    start,
                    cancelled,
                },
            );
        }
        Started::Entry
    }
}

fn key(texture: &CookedTexture, formats: Option<&TextureFormats>) -> (String, u64) {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    texture.ktx2.hash(&mut hash);
    texture.target_format(formats).hash(&mut hash);
    let m = &texture.meta;
    [
        m.width,
        m.height,
        m.levels,
        m.kind,
        m.codec,
        m.color_space,
        m.has_alpha,
        m.supercompression,
    ]
    .hash(&mut hash);
    (texture.id.clone(), hash.finish())
}
