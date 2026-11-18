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

#[derive(Default)]
pub(super) struct Textures {
    worker: Option<Sender<Job>>,
    entries: HashMap<(String, u64), Entry>,
}

impl Textures {
    pub(super) fn forget(&mut self, ids: &[String]) {
        self.entries.retain(|(id, _), _| !ids.contains(id));
    }

    pub(super) fn prepare(
        &mut self,
        host: &dyn Hooks,
        texture: &CookedTexture,
        formats: Option<&TextureFormats>,
    ) -> super::Fetch<TextureSource> {
        use super::Fetch;
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
        let signature = hash.finish();
        // Runtime bundles may use the same id for different maps at once.
        // They must not continually cancel each other's preparation.
        let key = (texture.id.clone(), signature);
        if !self.entries.contains_key(&key) {
            let Some(work): Option<TextureWork> = host.texture_work(texture, formats) else {
                return host.texture(texture, formats);
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
                    Err(e) => return Fetch::Failed(format!("texture worker: {e}")),
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
                return Fetch::Failed("texture worker stopped".into());
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
        let entry = self.entries.get_mut(&key).unwrap();
        if entry.ready.is_none() {
            match entry.receiver.try_recv() {
                Ok((answer, offset, ms)) => {
                    crate::author::trace::texture_work(&texture.id, entry.start, offset, ms);
                    entry.ready = Some(answer);
                }
                Err(TryRecvError::Empty) => return Fetch::Pending,
                Err(TryRecvError::Disconnected) => {
                    entry.ready = Some(Err("texture worker stopped".into()));
                }
            }
        }
        match entry.ready.as_ref().unwrap() {
            Ok(source) => Fetch::Ready(source.clone()),
            Err(e) => Fetch::Failed(e.clone()),
        }
    }
}
