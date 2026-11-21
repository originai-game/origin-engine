//! Per-engine texture preparation. One worker, not one thread per texture;
//! dropping the engine closes its queue and discards unfinished answers.

use crate::{Hooks, TextureWork};
use og_bundle::runtime::CookedTexture;
use og_render::{TextureFormats, TextureSource};
use std::collections::HashMap;
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

    /// Whether `texture` is still being prepared, its work started if it
    /// had none. It copies no prepared source: [`Self::prepare`] answers
    /// a ready one with a copy of its whole chain. A host that prepares
    /// textures itself is asked.
    pub(super) fn pending(
        &mut self,
        host: &dyn Hooks,
        texture: &CookedTexture,
        formats: Option<&TextureFormats>,
    ) -> bool {
        let key = key(texture, formats);
        match self.start(host, texture, formats, &key) {
            Started::Entry => !self.entries.get_mut(&key).unwrap().poll(&texture.id),
            Started::Host => matches!(host.texture(texture, formats), super::Fetch::Pending),
            Started::Failed(_) => false,
        }
    }

    /// The prepared sources of `textures`, for a caller that puts them in
    /// place and asks no more: each is moved out of the cache with its
    /// entry, not copied, unless its flag says another user wants it after
    /// this one. All or none: while any is being prepared the answer is
    /// `Pending` and no ready one is taken, and a failure takes none.
    pub(super) fn take(
        &mut self,
        host: &dyn Hooks,
        textures: &[(&CookedTexture, bool)],
        formats: Option<&TextureFormats>,
    ) -> super::Fetch<Vec<TextureSource>> {
        use super::Fetch;
        let failed = |t: &CookedTexture, e: &str| Fetch::Failed(format!("{}: {e}", t.id));
        let keys: Vec<_> = textures.iter().map(|(t, _)| key(t, formats)).collect();
        let mut hosted = false;
        for ((t, _), key) in textures.iter().zip(&keys) {
            match self.start(host, t, formats, key) {
                Started::Entry => {}
                Started::Host => hosted = true,
                Started::Failed(e) => return failed(t, &e),
            }
        }
        if hosted {
            // The host's own: it keeps what it made.
            let mut out = Vec::with_capacity(textures.len());
            for (t, _) in textures {
                match host.texture(t, formats) {
                    Fetch::Ready(source) => out.push(source),
                    Fetch::Pending => return Fetch::Pending,
                    Fetch::Failed(e) => return failed(t, &e),
                }
            }
            return Fetch::Ready(out);
        }
        let mut pending = false;
        for ((t, _), key) in textures.iter().zip(&keys) {
            pending |= !self.entries.get_mut(key).unwrap().poll(&t.id);
        }
        if pending {
            return Fetch::Pending;
        }
        for ((t, _), key) in textures.iter().zip(&keys) {
            if let Some(Err(e)) = &self.entries[key].ready {
                return failed(t, e);
            }
        }
        let mut out: Vec<TextureSource> = Vec::with_capacity(textures.len());
        for (i, ((_, keep), key)) in textures.iter().zip(&keys).enumerate() {
            // Named twice: taken once, and a copy for the other.
            if let Some(first) = keys[..i].iter().position(|k| k == key) {
                out.push(out[first].clone());
                continue;
            }
            let ready = match keep {
                true => self.entries[key].ready.clone(),
                false => self.entries.remove(key).and_then(|mut e| e.ready.take()),
            };
            match ready {
                Some(Ok(source)) => out.push(source),
                _ => unreachable!("every answer arrived and none failed"),
            }
        }
        Fetch::Ready(out)
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
    (texture.id.clone(), texture.prepared_key(formats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dispatch, Fetch};
    use std::sync::atomic::AtomicUsize;

    /// A host whose worker makes texture "b" only once the gate opens,
    /// and counts what it made.
    struct Gated {
        open: Arc<AtomicBool>,
        made: Arc<AtomicUsize>,
    }
    impl Hooks for Gated {
        fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
        fn texture_work(
            &self,
            texture: &CookedTexture,
            _: Option<&TextureFormats>,
        ) -> Option<TextureWork> {
            let (open, made, held) = (self.open.clone(), self.made.clone(), texture.id == "b");
            let side = texture.meta.width;
            Some(Box::new(move || {
                while held && !open.load(Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                made.fetch_add(1, Ordering::AcqRel);
                Ok(TextureSource::Rgba(og_render::TextureData {
                    width: side,
                    height: side,
                    rgba: vec![7; (side * side * 4) as usize],
                    color_space: og_render::ColorSpace::Srgb,
                    mips: og_render::Mips::None,
                }))
            }))
        }
    }

    fn cooked(id: &str, side: u32) -> CookedTexture {
        CookedTexture {
            id: id.into(),
            meta: og_bundle::format::TextureMeta {
                width: side,
                height: side,
                levels: 1,
                kind: og_bundle::format::kind::COLOR,
                codec: og_bundle::format::codec::RGBA8,
                color_space: 1,
                has_alpha: 1,
                supercompression: 0,
            },
            ktx2: vec![side as u8],
            mips: None,
            orientation: 1,
        }
    }

    /// A model's textures are taken together or not at all: while one is
    /// being prepared the ready ones stay in the cache (made once, never
    /// again), and once all are ready each is moved out with its entry
    /// unless a later user is named for it.
    #[test]
    fn taking_waits_for_every_texture_and_consumes_none_before() {
        let (open, made) = (
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicUsize::new(0)),
        );
        let host = Gated {
            open: open.clone(),
            made: made.clone(),
        };
        let (a, b) = (cooked("a", 2), cooked("b", 4));
        let mut textures = Textures::default();
        let wait = |textures: &mut Textures, t: &CookedTexture| {
            for _ in 0..5000 {
                if !textures.pending(&host, t, None) {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            panic!("{} was never prepared", t.id);
        };
        assert!(matches!(
            textures.take(&host, &[(&a, false), (&b, false)], None),
            Fetch::Pending
        ));
        wait(&mut textures, &a);
        for _ in 0..3 {
            let both = textures.take(&host, &[(&a, false), (&b, false)], None);
            assert!(matches!(both, Fetch::Pending), "b is still gated");
            assert!(textures.pending(&host, &b, None));
        }
        let mut ids = textures.ids();
        ids.sort();
        assert_eq!(ids, ["a", "b"], "a's ready source was not consumed");
        assert_eq!(made.load(Ordering::Acquire), 1);
        open.store(true, Ordering::Release);
        wait(&mut textures, &b);
        // Another user wants b after this one: it stays, a goes.
        let Fetch::Ready(sources) = textures.take(&host, &[(&a, false), (&b, true)], None) else {
            panic!("both are ready");
        };
        let sides: Vec<u32> = sources.iter().map(|s| s.size().0).collect();
        assert_eq!((sides, textures.ids()), (vec![2, 4], vec!["b".to_string()]));
        // Named twice by its last user: taken once and copied.
        let Fetch::Ready(sources) = textures.take(&host, &[(&b, false), (&b, false)], None) else {
            panic!("b is ready");
        };
        assert_eq!((sources.len(), textures.ids().len()), (2, 0));
        assert_eq!(made.load(Ordering::Acquire), 2, "each was made once");
    }
}
