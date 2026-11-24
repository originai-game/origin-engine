//! What the main camera's cull made of each drawable in a frame drawn:
//! the record the probe answers `visible` and `dump_drawn` from
//! ([`crate::Renderer::cull_record`]). It is written by what decides,
//! as it decides, and by nothing else:
//!
//! - the sort (`instances.rs`) leaves out empty meshes, wireframe
//!   materials (the lines pass draws them) and instances fading in that
//!   show nothing yet;
//! - the GPU cull's camera pass writes a word per instance (`cull.wgsl`
//!   `SEEN_*`): kept, off its layers, outside its planes or behind the
//!   occluders' depth, read back with its counts;
//! - on the GL tier (or a frame past the device's limits) the CPU cull of
//!   the camera's opaque batches;
//! - the transparent sort, the skinned cull and the sprites' build.
//!
//! The record exists where the probe does (every build but `publish`).

/// What the main camera's cull made of one drawable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Culled {
    /// It survives the cull: drawn.
    Drawn,
    /// Off the camera's layers ([`crate::RenderScene::camera_layers`]).
    Layer,
    /// Its bounds lie outside the camera's view (past near or far too).
    OutsideView,
    /// Inside the view, behind the occluders' depth (the GPU cull's
    /// occlusion test).
    Occluded,
    /// Fading in ([`crate::InstanceFx::reveal`]) with none of it shown yet.
    FadingIn,
    /// A wireframe material's: drawn by the lines pass, which the record
    /// does not follow.
    Lines,
    /// Nothing to draw: a mesh not loaded or empty, a sprite of no size.
    Empty,
}

impl Culled {
    /// The camera pass's word for an instance (`cull.wgsl` `SEEN_*`);
    /// `None` for one of a batch it does not draw (a transparent one,
    /// which the CPU sorts and records).
    pub(crate) fn seen(word: u32) -> Option<Culled> {
        match word {
            1 => Some(Culled::Drawn),
            2 => Some(Culled::Layer),
            3 => Some(Culled::OutsideView),
            4 => Some(Culled::Occluded),
            _ => None,
        }
    }
}

/// The main camera's cull of one frame, parallel to the scene's lists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CullRecord {
    /// The frame it records (`FrameStats::frame`).
    pub frame: u64,
    /// Per [`crate::RenderScene::instances`].
    pub instances: Vec<Culled>,
    /// Per [`crate::RenderScene::skinned_instances`].
    pub skinned: Vec<Culled>,
    /// Per [`crate::RenderScene::sprites`].
    pub sprites: Vec<Culled>,
}

/// Where the GPU cull's words of a frame go in its record: the culled
/// instances in their order (the sorted ones, then the skinned boxes from
/// `base`), each one's index in its list.
#[derive(Clone, Debug, Default)]
pub(crate) struct GpuSlots {
    pub sorted: Vec<u32>,
    pub base: u32,
    pub skinned: Vec<u32>,
}

/// A frame's record as its culls write it: whole once the GPU's words,
/// when it culled on the GPU, are in.
#[derive(Clone, Debug, Default)]
pub(crate) struct RecordFrame {
    pub record: CullRecord,
    pub gpu: Option<GpuSlots>,
}

impl RecordFrame {
    /// Starts frame `frame` of a scene of `instances` instances, `skinned`
    /// skinned ones and `sprites` sprites: each is empty until a cull
    /// records it.
    pub fn begin(&mut self, frame: u64, instances: usize, skinned: usize, sprites: usize) {
        let r = &mut self.record;
        r.frame = frame;
        for (list, n) in [
            (&mut r.instances, instances),
            (&mut r.skinned, skinned),
            (&mut r.sprites, sprites),
        ] {
            list.clear();
            list.resize(n, Culled::Empty);
        }
        self.gpu = None;
    }

    /// The camera pass's words (per culled instance, `cull.wgsl` `scan`)
    /// into the record: it is whole.
    pub fn apply(&mut self, words: &[u32]) {
        let Some(slots) = self.gpu.take() else {
            return;
        };
        let r = &mut self.record;
        let at = |list: &mut [Culled], i: u32, word: Option<&u32>| {
            if let (Some(c), Some(v)) = (
                word.and_then(|&w| Culled::seen(w)),
                list.get_mut(i as usize),
            ) {
                *v = c;
            }
        };
        for (k, &i) in slots.sorted.iter().enumerate() {
            at(&mut r.instances, i, words.get(k));
        }
        for (j, &i) in slots.skinned.iter().enumerate() {
            at(&mut r.skinned, i, words.get(slots.base as usize + j));
        }
    }
}

/// The records a renderer keeps: the last frame's, the frames whose GPU
/// words were copied and are on their way (the web reads them frames
/// later), and the newest whole one of those.
#[derive(Default)]
pub(crate) struct Records {
    /// The frame being drawn, or the last one drawn; `None` before any.
    pub frame: Option<RecordFrame>,
    asked: Vec<RecordFrame>,
    whole: Option<CullRecord>,
}

/// Frames kept waiting for their words: as many as the readbacks in flight.
const ASKED: usize = 4;

impl Records {
    /// After a frame's submit: a frame whose words were copied to a
    /// readback waits for them.
    pub fn submitted(&mut self, copied: bool) {
        if let Some(f) = self.frame.as_ref().filter(|f| copied && f.gpu.is_some()) {
            if self.asked.len() == ASKED {
                self.asked.remove(0);
            }
            self.asked.push(f.clone());
        }
    }

    /// Frame `frame`'s words landed: its record is whole.
    pub fn landed(&mut self, frame: u64, words: &[u32]) {
        if let Some(f) = self
            .frame
            .as_mut()
            .filter(|f| f.record.frame == frame && f.gpu.is_some())
        {
            f.apply(words);
        }
        if let Some(i) = self.asked.iter().position(|f| f.record.frame == frame) {
            let mut f = self.asked.remove(i);
            f.apply(words);
            self.whole = Some(f.record);
        }
        self.asked.retain(|f| f.record.frame > frame);
    }

    /// The last frame drawn's record when it is whole; else (its GPU
    /// words still on their way) the newest whole one.
    pub fn newest(&self) -> Option<&CullRecord> {
        match &self.frame {
            Some(f) if f.gpu.is_none() => Some(&f.record),
            _ => self.whole.as_ref(),
        }
    }

    /// The frame drawn last when its GPU words were copied for a later
    /// read (the web's): its drawables' owners are to be kept until then.
    pub fn asked(&self) -> Option<u64> {
        let last = self.frame.as_ref()?.record.frame;
        self.asked
            .last()
            .filter(|f| f.record.frame == last)
            .map(|_| last)
    }

    /// The frame drawn last, while its GPU words are on their way.
    pub fn pending(&self) -> Option<u64> {
        self.frame
            .as_ref()
            .filter(|f| f.gpu.is_some())
            .map(|f| f.record.frame)
    }
}
