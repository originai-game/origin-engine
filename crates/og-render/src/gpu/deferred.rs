//! The pipeline cache: every pipeline the renderer draws with, made when
//! it is first asked for, with its state.
//!
//! Every pipeline is a program to compile before a pass draws with it:
//! the renderer making all of its fixed pipelines up front cost the web
//! page of the retired WebGL2 tier 400 to 550 ms before its first frame
//! (web finding #11), most of them for passes the first frame does not
//! draw. So a pipeline (or a set of them, or a pipeline per sample count:
//! [`Deferred`], [`Sampled`]) is an entry made from its descriptor when it
//! is first asked for, and is in one of these states ([`State`], Bevy's
//! `PipelineCache` (MIT/Apache) in this shape):
//!
//! - **Queued**: asked for in the background, held until the renderer's
//!   second frame has been submitted ([`Priority::Background`],
//!   [`release_background`]): the full lit variants (pipeline_features.rs).
//!   A renderer that draws one frame never compiles them, and its drop
//!   waits for none.
//! - **Creating**: the host's compiler has it. Natively that is the
//!   compile workers (wgpu's native device makes pipelines from any
//!   thread, as Bevy's cache compiles on a task pool); on the web it is
//!   the browser's `createRenderPipelineAsync` (wgpu's
//!   `create_render_pipeline_async`, [`web`]), since a cold driver compiles one for seconds,
//!   and synchronous ones in one GPU-process task past Chrome's 15 s
//!   watchdog lose the device. An entry that cannot leave the renderer's
//!   thread natively ([`Deferred::new`]) is made there when asked for.
//! - **Ready**.
//!
//! A frame asks for what it will draw before it encodes its passes
//! ([`Start`]), so its compiles overlap each other and the CPU work
//! between them, and one rule decides what a use does with an entry that
//! is not ready, the same on every host ([`FramePolicy`]):
//!
//! - a frame that waits (the default; a scripted, recorded or capture
//!   frame) waits for it: natively for its worker, on the web by having the
//!   browser make it at once;
//! - a display frame draws without it: a draw in a pass drawn again next
//!   frame ([`Skipping`]) is skipped ([`Deferred::get`]), and work whose
//!   output later frames keep (a bake, a scatter chunk's placement, the
//!   compute before the frame's passes) is left for a later frame
//!   ([`Deferred::ready`]), its readers with it. A compute dispatch is never
//!   skipped: what it writes is read later in the same frame. Either way the
//!   frame counts what it left ([`frame_skipped`]). A renderer's first
//!   frame is never shown without what it draws: natively it waits, as a
//!   capture does; on the web the host draws it again until it left
//!   nothing.
//!
//! A pipeline still compiling when its entry is dropped is waited for
//! ([`Pending`]): the driver cannot stop a compile.

use std::cell::{Cell, OnceCell};
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock};

/// A closure that makes a value off the renderer's thread (natively; on
/// the web, where nothing is made off it, any closure).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type MakeAnywhere<T> = Box<dyn FnOnce() -> T + Send>;
#[cfg(target_arch = "wasm32")]
pub(crate) type MakeAnywhere<T> = Box<dyn FnOnce() -> T>;

/// What a closure or module needs to be made off the renderer's thread:
/// `Send + Sync` natively, nothing on the web.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait Anywhere: Send + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync> Anywhere for T {}
#[cfg(target_arch = "wasm32")]
pub(crate) trait Anywhere {}
#[cfg(target_arch = "wasm32")]
impl<T> Anywhere for T {}

/// What a closure run once needs to be run off the renderer's thread:
/// `Send` natively, nothing on the web.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait AnywhereOnce: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> AnywhereOnce for T {}
#[cfg(target_arch = "wasm32")]
pub(crate) trait AnywhereOnce {}
#[cfg(target_arch = "wasm32")]
impl<T> AnywhereOnce for T {}

/// An entry's closure: natively run once; on the web run once to ask for
/// its pipelines asynchronously and again to make it from them
/// ([`web::probe`]).
#[cfg(not(target_arch = "wasm32"))]
type MakeHere<T> = Box<dyn FnOnce() -> T>;
#[cfg(not(target_arch = "wasm32"))]
type MakeEntry<T> = MakeAnywhere<T>;
#[cfg(target_arch = "wasm32")]
type MakeHere<T> = Box<dyn Fn() -> T>;
#[cfg(target_arch = "wasm32")]
type MakeEntry<T> = Box<dyn Fn() -> T>;

/// How an entry not made yet is made.
enum Make<T> {
    /// When asked for, here.
    Here(MakeHere<T>),
    /// When asked for, by the host's compiler.
    Anywhere(MakeEntry<T>),
    /// Asked for: queued or creating.
    Asked(Pending<T>),
}

/// An entry's state ([`Deferred::state`]): not asked for yet, or one of
/// the cache's states (see the module's documentation).
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Unasked,
    Queued,
    Creating,
    Ready,
}

/// The frame drawing now: whether it waits for the pipelines it meets, and
/// what it has left for a pipeline not ready ([`FramePolicy`]).
struct Frame {
    waits: bool,
    skipping: u32,
    skipped: u32,
    depth: u32,
}

thread_local! {
    static FRAME: Cell<Frame> = const {
        Cell::new(Frame { waits: true, skipping: 0, skipped: 0, depth: 0 })
    };
    /// What the last frame left for a pipeline not ready ([`frame_skipped`]).
    static LAST_SKIPPED: Cell<u32> = const { Cell::new(0) };
}

fn frame<R>(f: impl FnOnce(&mut Frame) -> R) -> R {
    FRAME.with(|c| {
        let mut frame = c.replace(Frame {
            waits: true,
            skipping: 0,
            skipped: 0,
            depth: 0,
        });
        let r = f(&mut frame);
        c.set(frame);
        r
    })
}

/// The frame's policy, open while the renderer draws one (the outermost
/// frame's, for a frame drawn inside another): whether it waits for every
/// pipeline it meets, or draws without one that is not ready (a display
/// frame). Outside a frame, every use waits.
pub(crate) struct FramePolicy(bool);

impl FramePolicy {
    pub(crate) fn open(waits: bool) -> FramePolicy {
        frame(|f| {
            f.depth += 1;
            if f.depth == 1 {
                f.skipped = 0;
            }
            FramePolicy(std::mem::replace(&mut f.waits, waits))
        })
    }
}

impl Drop for FramePolicy {
    fn drop(&mut self) {
        frame(|f| {
            f.depth -= 1;
            f.waits = self.0;
            if f.depth == 0 {
                LAST_SKIPPED.with(|l| l.set(f.skipped));
            }
        });
    }
}

/// A pass drawn again next frame, open while this lives: in a display
/// frame, a draw in it whose pipeline is not ready is skipped
/// ([`Deferred::get`]). Not the passes whose output later frames keep
/// (`FramePass::redrawn`).
pub(crate) struct Skipping(());

impl Skipping {
    pub(crate) fn open() -> Skipping {
        frame(|f| f.skipping += 1);
        Skipping(())
    }
}

impl Drop for Skipping {
    fn drop(&mut self) {
        frame(|f| f.skipping -= 1);
    }
}

/// Whether a use now waits for its pipeline.
pub(crate) fn waits() -> bool {
    frame(|f| f.waits)
}

/// Whether a draw now skips a pipeline that is not ready.
fn skips() -> bool {
    frame(|f| !f.waits && f.skipping > 0)
}

fn skipped() {
    frame(|f| f.skipped += 1);
}

/// What the frame drawing now has left so far ([`frame_skipped`]).
pub(crate) fn skipped_now() -> u32 {
    frame(|f| f.skipped)
}

/// The draws the frame drawn last skipped and the work it left for a
/// later frame, each for a pipeline that was not ready: 0 when it was
/// drawn whole. The web host draws its first frame again until it is (a
/// renderer's first frame waits natively).
pub fn frame_skipped() -> u32 {
    LAST_SKIPPED.with(|l| l.get())
}

/// The entries made on the renderer's thread without a compiler (natively
/// [`Deferred::new`]'s; on the web those that ask for no pipeline
/// asynchronously, all of them on a page that compiles synchronously):
/// each is asked for and made at once ([`pipeline_counts`]).
static MADE_HERE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The entries asked for so far and how many of them are made, on every
/// host: those made on the renderer's thread, and those the host's
/// compiler was handed (the compile workers natively, the browser's
/// asynchronous calls on the web) and has finished, queued ones included.
/// Both only go up, and are equal when nothing compiles: a loading
/// screen's count (og.loading.status).
pub fn pipeline_counts() -> (u32, u32) {
    let here = MADE_HERE.load(std::sync::atomic::Ordering::Relaxed);
    let (asked, made) = compiler_counts();
    (here + asked, here + made)
}

#[cfg(not(target_arch = "wasm32"))]
fn compiler_counts() -> (u32, u32) {
    use std::sync::atomic::Ordering::Relaxed;
    // Finished first: never more made than asked.
    let finished = FINISHED.load(Relaxed);
    (STARTED.load(Relaxed).max(finished), finished)
}

#[cfg(target_arch = "wasm32")]
fn compiler_counts() -> (u32, u32) {
    let queued = web::QUEUED.with(|q| q.get());
    let stats = web::pipeline_stats();
    (queued + stats.requested, stats.settled)
}

/// `make()` on the renderer's thread, counted made here and timed as
/// waited ([`take_waited_us`]).
fn made_here<T>(make: impl FnOnce() -> T) -> T {
    MADE_HERE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    timed(make)
}

#[cfg(target_arch = "wasm32")]
pub use web::{
    PipelineCompile, PipelineMode, PipelineStats, pipeline_compiles, pipeline_stats,
    set_pipeline_mode, warming,
};
#[cfg(target_arch = "wasm32")]
pub(crate) use web::{compute_pipeline, render_pipeline};

/// A shader module made by `make`: on the web, one an entry's first run
/// made is handed to the run that makes the entry ([`web::probe`]), so
/// its WGSL is parsed once.
pub(crate) fn recorded_module(
    label: Option<&str>,
    make: impl FnOnce() -> wgpu::ShaderModule,
) -> wgpu::ShaderModule {
    #[cfg(target_arch = "wasm32")]
    return web::module(label, make);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = label;
        make()
    }
}

/// The web's compiler: the browser's `createRenderPipelineAsync` and
/// `createComputePipelineAsync` (wgpu's `create_*_pipeline_async`), which
/// compile off Chrome's GPU-process main task.
///
/// An entry's closure makes its pipelines with the synchronous calls
/// (`crate::traffic`), so it is run twice: first asking ([`probe`]), where
/// each pipeline it makes is asked of the browser asynchronously and a
/// stand-in takes its place in a value that is then dropped, and once all
/// of them are compiled (or refused) again, taking ([`take`]), where each
/// call gets its compiled pipeline back in the same order (a refused one
/// is made with the synchronous call, which reports why). A use that waits
/// for an entry still compiling makes it at once with the synchronous
/// calls ([`waited`]). The shader modules the first run made are handed to
/// the second.
#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::{Rc, Weak};

    /// How the web host makes the cache's pipelines: project.ogt
    /// `publish { pipelines }`, or auto's pick at the adapter.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum PipelineMode {
        /// Each with the synchronous call, at once.
        Sync,
        /// Asked of the browser asynchronously.
        Async,
        /// Synchronously, but those the renderer warms behind the play
        /// gate ([`warming`]) asynchronously.
        Prewarm,
    }

    /// The render and compute pipelines the cache made each way so far.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct PipelineStats {
        /// Asked of the browser asynchronously, and how many of those are
        /// compiled or refused (`requested - settled` compile now).
        pub requested: u32,
        pub settled: u32,
        /// Made with the synchronous call: in an entry made at once, and in
        /// one made at once by a use that waited for it after it asked (or
        /// for one the browser refused).
        pub sync: u32,
        pub waited: u32,
        /// Entries made asking asynchronously ([`probe`]).
        pub asks: u32,
    }

    /// A pipeline asked for asynchronously: its label, when it was asked
    /// for and made (ms, the global's `performance.now`; `None` while it
    /// compiles), and how: `async` (the browser's), `waited` (made at once
    /// by a use that waited for it) or `refused`.
    #[derive(Clone, Debug)]
    pub struct PipelineCompile {
        pub label: String,
        pub asked: f64,
        pub made: Option<f64>,
        pub how: Option<&'static str>,
    }

    thread_local! {
        static MODE: Cell<PipelineMode> = const { Cell::new(PipelineMode::Sync) };
        static WARMING: Cell<u32> = const { Cell::new(0) };
        static STATS: Cell<PipelineStats> = const {
            Cell::new(PipelineStats { requested: 0, settled: 0, sync: 0, waited: 0, asks: 0 })
        };
        static COMPILES: RefCell<Vec<PipelineCompile>> = const { RefCell::new(Vec::new()) };
        static MAKING: RefCell<Making> = const { RefCell::new(Making::Here) };
        /// The pipelines that stand in for those asked for, made once for
        /// the last device that asked.
        static STAND_INS: RefCell<Option<StandIns>> = const { RefCell::new(None) };
        /// The global's `performance` and its `now`.
        static NOW: Option<(wasm_bindgen::JsValue, js_sys::Function)> = {
            use wasm_bindgen::JsCast;
            let performance = js_sys::Reflect::get(&js_sys::global(), &"performance".into()).ok();
            performance.and_then(|p| {
                let now = js_sys::Reflect::get(&p, &"now".into()).ok()?;
                Some((p, now.dyn_into::<js_sys::Function>().ok()?))
            })
        };
        /// Background entries held until the second frame ([`super::release_background`]).
        pub(super) static HELD: RefCell<Vec<Weak<dyn Release>>> = const { RefCell::new(Vec::new()) };
        /// A frame's entries, asked of the browser once the renderer's call
        /// returns, [`PER_TASK`] a task ([`hold_frame`]), and whether that
        /// is scheduled.
        static FRAME_HELD: RefCell<VecDeque<Weak<dyn Release>>> = const { RefCell::new(VecDeque::new()) };
        static FRAME_SCHEDULED: Cell<bool> = const { Cell::new(false) };
        /// How many entries are held, of both kinds.
        pub(super) static QUEUED: Cell<u32> = const { Cell::new(0) };
    }

    /// How the cache makes its pipelines from now on (`Sync`, the
    /// default: each at once, as natively without workers).
    pub fn set_pipeline_mode(mode: PipelineMode) {
        MODE.with(|m| m.set(mode));
    }

    /// `f()`, the renderer's warm-up behind the play gate: under
    /// [`PipelineMode::Prewarm`] the pipelines it asks for are asked of the
    /// browser asynchronously.
    pub fn warming<R>(f: impl FnOnce() -> R) -> R {
        WARMING.with(|w| w.set(w.get() + 1));
        let r = f();
        WARMING.with(|w| w.set(w.get() - 1));
        r
    }

    pub fn pipeline_stats() -> PipelineStats {
        STATS.with(|s| s.get())
    }

    /// Every pipeline asked for asynchronously so far, in the order asked.
    pub fn pipeline_compiles() -> Vec<PipelineCompile> {
        COMPILES.with(|c| c.borrow().clone())
    }

    fn stat(f: impl FnOnce(&mut PipelineStats)) {
        STATS.with(|s| {
            let mut stats = s.get();
            f(&mut stats);
            s.set(stats);
        });
    }

    fn now() -> f64 {
        NOW.with(|n| {
            n.as_ref()
                .and_then(|(p, f)| f.call0(p).ok()?.as_f64())
                .unwrap_or(0.0)
        })
    }

    /// Whether an entry asked for now asks for its pipelines asynchronously.
    pub(super) fn asks_async() -> bool {
        match MODE.with(|m| m.get()) {
            PipelineMode::Sync => false,
            PipelineMode::Async => true,
            PipelineMode::Prewarm => WARMING.with(|w| w.get()) > 0,
        }
    }

    /// What the pipelines and modules made now are made for.
    enum Making {
        /// An entry made at once (or no entry).
        Here,
        /// An entry a use waited for while its pipelines compiled.
        Waited,
        /// An entry's first run: its pipelines asked for asynchronously.
        Asking(Asked),
        /// The run that makes it from them.
        Taking(Asked),
    }

    /// What an entry's first run asked for, in order.
    #[derive(Default)]
    pub(super) struct Asked {
        modules: VecDeque<(Option<String>, wgpu::ShaderModule)>,
        pipelines: VecDeque<Slot>,
    }

    struct Slot {
        label: Option<String>,
        answer: Rc<RefCell<Answer>>,
        /// Its entry in `COMPILES`.
        compile: usize,
    }

    enum Answer {
        Compiling,
        Render(wgpu::RenderPipeline),
        Compute(wgpu::ComputePipeline),
        Refused,
    }

    impl Asked {
        fn settled(&self) -> bool {
            self.pipelines
                .iter()
                .all(|s| !matches!(*s.answer.borrow(), Answer::Compiling))
        }
    }

    /// `f()` with what it makes made for `making`; what that became.
    fn making<R>(making: Making, f: impl FnOnce() -> R) -> (R, Making) {
        let outer = MAKING.with(|m| m.replace(making));
        let r = f();
        (r, MAKING.with(|m| m.replace(outer)))
    }

    /// An entry asked for: made at once, its first run having asked for
    /// no pipeline, or its pipelines compiling.
    pub(super) enum Probed<T> {
        Made(T),
        Asked(Asked),
    }

    /// `make`'s first run, its pipelines asked of the browser
    /// asynchronously.
    pub(super) fn probe<T>(make: &dyn Fn() -> T) -> Probed<T> {
        stat(|s| s.asks += 1);
        let (value, made) = making(Making::Asking(Asked::default()), make);
        let Making::Asking(asked) = made else {
            unreachable!("restored as it was set")
        };
        if asked.pipelines.is_empty() {
            super::MADE_HERE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Probed::Made(value);
        }
        Probed::Asked(asked)
    }

    /// `make` run again once what it asked for is compiled.
    fn take<T>(make: &dyn Fn() -> T, asked: Asked) -> T {
        making(Making::Taking(asked), make).0
    }

    /// `make` at once, by a use that waited while its pipelines compiled.
    fn waited<T>(make: &dyn Fn() -> T, asked: Asked) -> T {
        let t = now();
        COMPILES.with(|c| {
            let mut compiles = c.borrow_mut();
            for slot in &asked.pipelines {
                let compile = &mut compiles[slot.compile];
                if compile.how.is_none() {
                    compile.made = Some(t);
                    compile.how = Some("waited");
                }
            }
        });
        making(Making::Waited, make).0
    }

    /// `make` at once.
    pub(super) fn here<T>(make: &dyn Fn() -> T) -> T {
        making(Making::Here, || super::made_here(make)).0
    }

    /// A pipeline asked for: its slot's answer, filled in when the
    /// browser's promise settles.
    fn ask(asked: &mut Asked, label: Option<&str>) -> Rc<RefCell<Answer>> {
        let answer = Rc::new(RefCell::new(Answer::Compiling));
        let compile = COMPILES.with(|c| {
            let mut compiles = c.borrow_mut();
            compiles.push(PipelineCompile {
                label: label.unwrap_or("").to_string(),
                asked: now(),
                made: None,
                how: None,
            });
            compiles.len() - 1
        });
        stat(|s| s.requested += 1);
        asked.pipelines.push_back(Slot {
            label: label.map(str::to_owned),
            answer: answer.clone(),
            compile,
        });
        answer
    }

    fn settle(answer: &RefCell<Answer>, compile: usize, made: Option<Answer>) {
        let refused = made.is_none();
        *answer.borrow_mut() = made.unwrap_or(Answer::Refused);
        stat(|s| s.settled += 1);
        let t = now();
        COMPILES.with(|c| {
            let compile = &mut c.borrow_mut()[compile];
            if compile.how.is_none() {
                compile.made = Some(t);
                compile.how = Some(if refused { "refused" } else { "async" });
            }
        });
    }

    /// The next pipeline the first run asked for, if it is the one `label`
    /// names (else the runs differ, and nothing more is taken).
    fn taken(asked: &mut Asked, label: Option<&str>) -> Option<Answer> {
        let slot = asked.pipelines.pop_front()?;
        if slot.label.as_deref() != label {
            asked.pipelines.clear();
            asked.modules.clear();
            return None;
        }
        Some(std::mem::replace(
            &mut *slot.answer.borrow_mut(),
            Answer::Refused,
        ))
    }

    /// The pipeline `desc` makes for the entry being made; `None`: the
    /// caller makes it at once with the synchronous call.
    pub(crate) fn render_pipeline(
        device: &wgpu::Device,
        desc: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> Option<wgpu::RenderPipeline> {
        MAKING.with(|m| match &mut *m.borrow_mut() {
            Making::Asking(asked) => {
                let answer = ask(asked, desc.label);
                let compile = asked.pipelines.back().map_or(0, |s| s.compile);
                #[allow(clippy::disallowed_methods)]
                let made = device.create_render_pipeline_async(desc);
                wasm_bindgen_futures::spawn_local(async move {
                    let made = made.await.ok().map(Answer::Render);
                    settle(&answer, compile, made);
                });
                Some(stand_ins(device, |s| s.render.clone()))
            }
            Making::Taking(asked) => match taken(asked, desc.label) {
                Some(Answer::Render(p)) => Some(p),
                _ => {
                    stat(|s| s.waited += 1);
                    None
                }
            },
            Making::Waited => {
                stat(|s| s.waited += 1);
                None
            }
            Making::Here => {
                stat(|s| s.sync += 1);
                None
            }
        })
    }

    /// [`render_pipeline`] for a compute pipeline.
    pub(crate) fn compute_pipeline(
        device: &wgpu::Device,
        desc: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> Option<wgpu::ComputePipeline> {
        MAKING.with(|m| match &mut *m.borrow_mut() {
            Making::Asking(asked) => {
                let answer = ask(asked, desc.label);
                let compile = asked.pipelines.back().map_or(0, |s| s.compile);
                #[allow(clippy::disallowed_methods)]
                let made = device.create_compute_pipeline_async(desc);
                wasm_bindgen_futures::spawn_local(async move {
                    let made = made.await.ok().map(Answer::Compute);
                    settle(&answer, compile, made);
                });
                Some(stand_ins(device, |s| s.compute.clone()))
            }
            Making::Taking(asked) => match taken(asked, desc.label) {
                Some(Answer::Compute(p)) => Some(p),
                _ => {
                    stat(|s| s.waited += 1);
                    None
                }
            },
            Making::Waited => {
                stat(|s| s.waited += 1);
                None
            }
            Making::Here => {
                stat(|s| s.sync += 1);
                None
            }
        })
    }

    /// A module made for the entry being made ([`super::recorded_module`]).
    pub(super) fn module(
        label: Option<&str>,
        make: impl FnOnce() -> wgpu::ShaderModule,
    ) -> wgpu::ShaderModule {
        let taken = MAKING.with(|m| match &mut *m.borrow_mut() {
            // By label: a module the first run made through a shared
            // [`super::Module`] is not made again.
            Making::Taking(asked) => {
                let i = asked
                    .modules
                    .iter()
                    .position(|(l, _)| l.as_deref() == label)?;
                asked.modules.remove(i).map(|(_, module)| module)
            }
            _ => None,
        });
        if let Some(module) = taken {
            return module;
        }
        let module = make();
        MAKING.with(|m| {
            if let Making::Asking(asked) = &mut *m.borrow_mut() {
                asked
                    .modules
                    .push_back((label.map(str::to_owned), module.clone()));
            }
        });
        module
    }

    struct StandIns {
        device: wgpu::Device,
        render: wgpu::RenderPipeline,
        compute: wgpu::ComputePipeline,
    }

    const STAND_IN: &str = "@vertex fn v() -> @builtin(position) vec4f { return vec4f(0.0); }
@fragment fn f() -> @location(0) vec4f { return vec4f(0.0); }
@compute @workgroup_size(1) fn c() {}";

    /// `f` of `device`'s stand-ins, made the first time it asks.
    fn stand_ins<R>(device: &wgpu::Device, f: impl FnOnce(&StandIns) -> R) -> R {
        STAND_INS.with(|s| {
            let mut s = s.borrow_mut();
            if s.as_ref().is_none_or(|s| s.device != *device) {
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("og stand-in"),
                    source: wgpu::ShaderSource::Wgsl(STAND_IN.into()),
                });
                #[allow(clippy::disallowed_methods)]
                let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("og stand-in"),
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("v"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
                        entry_point: Some("f"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                    }),
                    multiview_mask: None,
                    cache: None,
                });
                #[allow(clippy::disallowed_methods)]
                let compute = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("og stand-in"),
                    layout: None,
                    module: &module,
                    entry_point: Some("c"),
                    compilation_options: Default::default(),
                    cache: None,
                });
                *s = Some(StandIns {
                    device: device.clone(),
                    render,
                    compute,
                });
            }
            f(s.as_ref().expect("made just now"))
        })
    }

    pub(super) trait Release {
        fn release(&self);
        fn held(&self) -> bool;
    }

    /// An entry asked for on the web ([`super::Pending`]).
    pub(super) struct Entry<T> {
        make: super::MakeHere<T>,
        state: RefCell<State<T>>,
    }

    enum State<T> {
        /// Asked for and not run yet: until the second frame (a background
        /// entry), or until a later task (a frame's, [`hold_frame`]), so a
        /// use that waits for it meanwhile makes it once, with the
        /// synchronous calls, and the browser is not asked for it twice.
        Held {
            frame: bool,
        },
        Asked(Asked),
        Made(T),
        Taken,
    }

    impl<T> Entry<T> {
        /// Its value once it is ready.
        pub(super) fn ready(&self) -> Option<T> {
            let state = self.state.replace(State::Taken);
            match state {
                State::Made(value) => Some(value),
                State::Asked(asked) if asked.settled() => Some(take(&*self.make, asked)),
                other => {
                    self.state.replace(other);
                    None
                }
            }
        }

        /// Its value now, made at once if it is not ready.
        pub(super) fn wait(&self) -> T {
            match self.state.replace(State::Taken) {
                State::Held { .. } => {
                    QUEUED.with(|q| q.set(q.get() - 1));
                    here(&*self.make)
                }
                State::Made(value) => value,
                State::Asked(asked) if asked.settled() => take(&*self.make, asked),
                State::Asked(asked) => waited(&*self.make, asked),
                State::Taken => unreachable!("an entry is taken once"),
            }
        }
    }

    impl<T: 'static> Entry<T> {
        /// A frame asks for a background entry: it is asked of the browser
        /// with the frame's, or made at once on a page that compiles
        /// synchronously.
        pub(super) fn promote(self: &Rc<Self>) {
            if !matches!(*self.state.borrow(), State::Held { frame: false }) {
                return;
            }
            if asks_async() {
                self.state.replace(State::Held { frame: true });
                queue_frame(self.clone());
            } else {
                self.release();
            }
        }
    }

    impl<T> Release for Entry<T> {
        fn held(&self) -> bool {
            matches!(*self.state.borrow(), State::Held { .. })
        }

        fn release(&self) {
            let State::Held { frame } = *self.state.borrow() else {
                return;
            };
            QUEUED.with(|q| q.set(q.get() - 1));
            let state = if frame || asks_async() {
                match probe(&*self.make) {
                    Probed::Made(value) => State::Made(value),
                    Probed::Asked(asked) => State::Asked(asked),
                }
            } else {
                State::Made(here(&*self.make))
            };
            self.state.replace(state);
        }
    }

    impl<T> Drop for Entry<T> {
        fn drop(&mut self) {
            if matches!(self.state.get_mut(), State::Held { .. }) {
                QUEUED.with(|q| q.set(q.get() - 1));
            }
        }
    }

    fn held<T>(make: super::MakeHere<T>, frame: bool) -> Rc<Entry<T>> {
        QUEUED.with(|q| q.set(q.get() + 1));
        Rc::new(Entry {
            make,
            state: RefCell::new(State::Held { frame }),
        })
    }

    /// A background entry, held until the second frame.
    pub(super) fn hold<T: 'static>(make: super::MakeHere<T>) -> Rc<Entry<T>> {
        let held = held(make, false);
        let weak: Weak<dyn Release> = Rc::downgrade(&(held.clone() as Rc<dyn Release>));
        HELD.with(|h| h.borrow_mut().push(weak));
        held
    }

    /// A frame's entry, asked of the browser in a later task ([`PER_TASK`]).
    pub(super) fn hold_frame<T: 'static>(make: super::MakeHere<T>) -> Rc<Entry<T>> {
        let held = held(make, true);
        queue_frame(held.clone());
        held
    }

    /// The frame's entries asked of the browser in one task: a use that
    /// waits for one not asked yet makes it once, with the synchronous
    /// calls, and the rest are asked in the tasks after.
    const PER_TASK: usize = 2;

    fn queue_frame<T: 'static>(entry: Rc<Entry<T>>) {
        let weak: Weak<dyn Release> = Rc::downgrade(&(entry as Rc<dyn Release>));
        FRAME_HELD.with(|h| h.borrow_mut().push_back(weak));
        if FRAME_SCHEDULED.with(|s| s.replace(true)) {
            return;
        }
        wasm_bindgen_futures::spawn_local(async {
            loop {
                next_task().await;
                let mut asked = 0;
                while asked < PER_TASK {
                    let Some(h) = FRAME_HELD.with(|h| h.borrow_mut().pop_front()) else {
                        break;
                    };
                    if let Some(h) = h.upgrade()
                        && h.held()
                    {
                        h.release();
                        asked += 1;
                    }
                }
                if FRAME_HELD.with(|h| h.borrow().is_empty()) {
                    break;
                }
            }
            FRAME_SCHEDULED.with(|s| s.set(false));
        });
    }

    /// Resolves in the browser's next task (`setTimeout(0)`).
    async fn next_task() {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let set_timeout = js_sys::Reflect::get(&js_sys::global(), &"setTimeout".into())
                .ok()
                .and_then(|f| wasm_bindgen::JsCast::dyn_into::<js_sys::Function>(f).ok());
            match set_timeout {
                Some(f) => {
                    let _ = f.call2(&wasm_bindgen::JsValue::NULL, &resolve, &0.into());
                }
                None => {
                    let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
                }
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }

    pub(super) fn release_all() {
        let held = HELD.with(|h| std::mem::take(&mut *h.borrow_mut()));
        for h in held {
            if let Some(h) = h.upgrade() {
                h.release();
            }
        }
    }
}

/// An entry on the web asked for: held until the second frame (or until a
/// use asks for it), or its pipelines compiling.
#[cfg(target_arch = "wasm32")]
pub(crate) struct Pending<T>(std::rc::Rc<web::Entry<T>>);

/// A value being made on a compile worker ([`spawn`]).
///
/// Dropped unreceived, it owns its job still: one not yet started is taken
/// off the queues and dropped here, with what its closure owns (the
/// device, layouts, modules), and one running is waited for (the driver
/// cannot stop a compile). Left to finish on its worker, a compile could
/// still be inside the driver's shader compiler when the process exited,
/// and the exit's teardown ran under it: on NVIDIA's Linux driver the
/// process then died after its tests had passed (heap corruption, SIGSEGV,
/// a worker spinning in the compiler), about one GPU test in 275 under the
/// gate's load. No exit handler can come first: the compiler registers
/// its own as it goes, after any of ours ([`wait_at_exit`] only covers a
/// renderer still alive at exit). So a renderer dropped while the full
/// variants compile in the background waits for them; they start only
/// from its second frame, so one dropped after its first has none.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct Pending<T> {
    answer: std::sync::mpsc::Receiver<std::thread::Result<T>>,
    /// Its job's id in [`Queues`].
    id: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> Drop for Pending<T> {
    fn drop(&mut self) {
        if let Some(job) = cancel(self.id) {
            drop(job);
            return;
        }
        // Running, or done (its answer, received or not, is dropped here;
        // a panic of work nobody uses is not raised again).
        let _ = self.answer.recv();
    }
}

/// At the process's exit, the jobs no worker has started are dropped and
/// those running are waited for: registered with the C runtime's `atexit`
/// when the workers start, so it runs after `main` returns or
/// `std::process::exit`, before the process ends. Only a renderer still
/// alive at exit has any (a dropped one waited for its own: [`Pending`]),
/// and handlers the driver registered after this one have run by then.
#[cfg(not(target_arch = "wasm32"))]
extern "C" fn wait_at_exit() {
    // Nothing unwinds out of the C runtime's exit.
    let _ = std::panic::catch_unwind(|| {
        let pool = pool();
        let mut queued: Vec<(u64, Job)> = Vec::new();
        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            queued.extend(q.frame.drain(..));
            queued.extend(q.background.drain(..));
            queued.append(&mut q.held);
            if q.running == 0 {
                break;
            }
            q = pool.idle.wait(q).unwrap_or_else(|e| e.into_inner());
        }
        drop(q);
        drop(queued);
    });
}

/// A worker's value, waited for, its panic passed on.
#[cfg(not(target_arch = "wasm32"))]
fn joined<T>(pending: Pending<T>) -> T {
    if let Ok(answer) = pending.answer.try_recv() {
        return match answer {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        };
    }
    // A compile waited for runs next, ahead of the others (the background
    // ones still wait for the frame: [`release_background`]).
    promote(pending.id, true);
    let t0 = std::time::Instant::now();
    let answer = pending.answer.recv();
    WAITED_US.fetch_add(
        t0.elapsed().as_micros() as u64,
        std::sync::atomic::Ordering::Relaxed,
    );
    match answer {
        Ok(Ok(value)) => value,
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => unreachable!("a compile worker always answers"),
    }
}

/// `make()` on the renderer's thread, its time counted as waited
/// ([`take_waited_us`]): what a frame spends making a pipeline it meets is
/// time it does not draw.
fn timed<T>(make: impl FnOnce() -> T) -> T {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let t0 = std::time::Instant::now();
        let v = make();
        WAITED_US.fetch_add(
            t0.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        v
    }
    #[cfg(target_arch = "wasm32")]
    make()
}

/// One pipeline's (or other deferred value's) making, timed
/// ([`log_compiles`]).
#[derive(Clone, Debug)]
pub struct CompileTime {
    /// What it made: the pipeline's label, entry points and constants
    /// where its builder names them ([`note`]), else the code that asked.
    pub name: String,
    pub ms: f64,
    /// Made on a compile worker (else on the renderer's thread).
    pub worker: bool,
}

/// Whether makings are timed ([`log_compiles`]).
static LOGGING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LOG: Mutex<Vec<CompileTime>> = Mutex::new(Vec::new());

std::thread_local! {
    /// What the making running on this thread makes ([`note`]).
    static NOTE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Times every deferred value made from now on, each pipeline compile
/// among them, for [`take_compile_times`]: a measurement harness's
/// (og-gfx `OG_GFX_COMPILES`), off by default.
pub fn log_compiles(on: bool) {
    LOGGING.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The makings timed since the last call, in the order they ended.
pub fn take_compile_times() -> Vec<CompileTime> {
    std::mem::take(&mut *LOG.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Names what the making running on this thread makes, when makings are
/// timed: a pipeline builder's label, entry points and constants.
pub(crate) fn note(name: impl FnOnce() -> String) {
    if LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
        NOTE.with(|n| *n.borrow_mut() = Some(name()));
    }
}

/// `make()`, timed when makings are ([`log_compiles`]), named by its
/// builder's [`note`] or else `asker` (the type of the closure that asked).
fn logged<T>(asker: &'static str, make: impl FnOnce() -> T) -> T {
    if !LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
        return make();
    }
    // A builder that ran outside a timed making left its note behind.
    NOTE.with(|n| n.borrow_mut().take());
    let t0 = std::time::Instant::now();
    let v = make();
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let name = NOTE
        .with(|n| n.borrow_mut().take())
        .unwrap_or_else(|| asker.to_string());
    let worker = std::thread::current()
        .name()
        .is_some_and(|n| n.starts_with("og-render compile"));
    LOG.lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(CompileTime { name, ms, worker });
    v
}

/// The compile workers to start ([`set_compile_workers`]); 0: one a core.
#[cfg(not(target_arch = "wasm32"))]
static WORKERS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// How many compile workers the process starts (0, the default: one a
/// core). Read once, when the first compile is started: a measurement
/// harness sets 1 before its first renderer, so each pipeline compiles
/// alone (og-gfx `OG_GFX_COMPILES`). Nothing on the web.
pub fn set_compile_workers(workers: usize) {
    #[cfg(not(target_arch = "wasm32"))]
    WORKERS.store(workers, std::sync::atomic::Ordering::Relaxed);
    #[cfg(target_arch = "wasm32")]
    let _ = workers;
}

/// A job for the compile workers.
#[cfg(not(target_arch = "wasm32"))]
type Job = Box<dyn FnOnce() + Send>;

/// Microseconds the renderer's thread has spent on pipelines: waiting for
/// the compile workers, or making one itself ([`take_waited_us`]).
#[cfg(not(target_arch = "wasm32"))]
static WAITED_US: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The time the renderer's thread spent on compiles since the last call
/// (`FrameStats::compile_wait_us`; 0 on the web, which waits for none).
pub(crate) fn take_waited_us() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    return WAITED_US.swap(0, std::sync::atomic::Ordering::Relaxed);
    #[cfg(target_arch = "wasm32")]
    0
}

/// When an entry asked for compiles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Priority {
    /// For a frame that is waiting or will wait for it: before any
    /// background work.
    Frame,
    /// Ahead of a need that may come (a pipeline variant with every
    /// feature, [`crate::pipeline_features::Variants`]): held until the
    /// renderer's second frame has been submitted ([`release_background`]),
    /// then compiled while no frame's work is queued.
    Background,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Queues {
    /// Every job is queued with its id ([`Pending`]).
    frame: std::collections::VecDeque<(u64, Job)>,
    /// Released background jobs, each with its id (a wait for one moves
    /// it to `frame`: [`promote`]).
    background: std::collections::VecDeque<(u64, Job)>,
    held: Vec<(u64, Job)>,
    next: u64,
    /// Jobs a worker is running.
    running: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl Queues {
    fn idle(&self) -> bool {
        self.frame.is_empty()
            && self.background.is_empty()
            && self.held.is_empty()
            && self.running == 0
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct Pool {
    queues: Mutex<Queues>,
    work: std::sync::Condvar,
    /// Notified when the last running job ends with nothing queued
    /// ([`settle`]).
    idle: std::sync::Condvar,
}

/// The compile workers: one a core, started on first use and kept for the
/// process. A bounded pool, not a thread a pipeline: a scene's first frame
/// can ask for dozens at once.
#[cfg(not(target_arch = "wasm32"))]
fn pool() -> &'static Pool {
    static POOL: OnceLock<&'static Pool> = OnceLock::new();
    POOL.get_or_init(|| {
        let pool: &'static Pool = Box::leak(Box::new(Pool {
            queues: Mutex::new(Queues::default()),
            work: std::sync::Condvar::new(),
            idle: std::sync::Condvar::new(),
        }));
        let workers = match WORKERS.load(std::sync::atomic::Ordering::Relaxed) {
            0 => std::thread::available_parallelism().map_or(4, |n| n.get()),
            n => n,
        };
        for i in 0..workers {
            std::thread::Builder::new()
                .name(format!("og-render compile {i}"))
                .spawn(move || {
                    loop {
                        let job = {
                            let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
                            loop {
                                if let Some((_, job)) =
                                    q.frame.pop_front().or_else(|| q.background.pop_front())
                                {
                                    q.running += 1;
                                    break job;
                                }
                                q = pool.work.wait(q).unwrap_or_else(|e| e.into_inner());
                            }
                        };
                        job();
                        // Once more after the first compile: the exit
                        // handlers run last-registered first, and what the
                        // driver's shader compiler registers as it loads
                        // in that compile would otherwise tear it down
                        // while a worker is still inside it (NVIDIA Linux:
                        // SIGSEGV or a spinning worker at exit).
                        static AFTER_FIRST: std::sync::Once = std::sync::Once::new();
                        // SAFETY: as at the first registration below.
                        AFTER_FIRST.call_once(|| unsafe {
                            atexit(wait_at_exit);
                        });
                        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
                        q.running -= 1;
                        // (`settle` waits for idle, the exit for no job
                        // running: `wait_at_exit`.)
                        if q.running == 0 {
                            pool.idle.notify_all();
                        }
                    }
                })
                .expect("a pipeline compile worker");
        }
        unsafe extern "C" {
            fn atexit(f: extern "C" fn()) -> std::ffi::c_int;
        }
        // SAFETY: `wait_at_exit` is an `extern "C"` function that does not
        // unwind; the C runtime calls it once, at exit.
        unsafe { atexit(wait_at_exit) };
        pool
    })
}

/// Lets the background compiles asked for so far start (a renderer's
/// second frame has been submitted: [`Priority::Background`]).
pub(crate) fn release_background() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let pool = pool();
        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
        if q.held.is_empty() {
            return;
        }
        let held = std::mem::take(&mut q.held);
        q.background.extend(held);
        pool.work.notify_all();
    }
    #[cfg(target_arch = "wasm32")]
    web::release_all();
}

/// Waits until every pipeline compile started so far has finished, in
/// every renderer of the process, the full lit variants compiling in the
/// background included (pipeline_features.rs): from then on the frames
/// draw with the full variants of every pipeline met so far. Display
/// frames drawn while a variant compiles may differ from later ones in
/// their last bit (docs/design.md §2.4), so a test that compares display
/// frames byte for byte settles them first. Natively; nothing on the web,
/// which cannot wait.
pub fn settle_pipelines() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        release_background();
        let pool = pool();
        let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
        while !q.idle() {
            q = pool.idle.wait(q).unwrap_or_else(|e| e.into_inner());
        }
    }
}

/// The jobs handed to the compile workers so far, and those made (or
/// cancelled before a worker took them): [`pipeline_counts`].
#[cfg(not(target_arch = "wasm32"))]
static STARTED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[cfg(not(target_arch = "wasm32"))]
static FINISHED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Runs `make` on the compile workers, in the order started within its
/// priority (a frame starts its longest compiles first).
#[cfg(not(target_arch = "wasm32"))]
fn spawn<T: Send + 'static>(
    priority: Priority,
    make: impl FnOnce() -> T + Send + 'static,
) -> Pending<T> {
    let (answer, pending) = std::sync::mpsc::channel();
    STARTED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let job: Job = Box::new(move || {
        let made = std::panic::catch_unwind(std::panic::AssertUnwindSafe(make));
        FINISHED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = answer.send(made);
    });
    let pool = pool();
    let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
    let id = q.next;
    q.next += 1;
    match priority {
        Priority::Frame => {
            q.frame.push_back((id, job));
            pool.work.notify_one();
        }
        Priority::Background => q.held.push((id, job)),
    }
    Pending {
        answer: pending,
        id,
    }
}

/// Takes the job `id` off the queues if no worker has started it (its
/// [`Pending`] was dropped); the caller drops it, outside the lock.
#[cfg(not(target_arch = "wasm32"))]
fn cancel(id: u64) -> Option<Job> {
    let pool = pool();
    let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
    let job = if let Some(i) = q.held.iter().position(|(j, _)| *j == id) {
        Some(q.held.remove(i).1)
    } else if let Some(i) = q.background.iter().position(|(j, _)| *j == id) {
        q.background.remove(i).map(|(_, job)| job)
    } else {
        let i = q.frame.iter().position(|(j, _)| *j == id);
        i.and_then(|i| q.frame.remove(i)).map(|(_, job)| job)
    };
    if job.is_some() {
        // Never made: it counts as settled.
        FINISHED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if q.idle() {
            pool.idle.notify_all();
        }
    }
    job
}

/// Moves the job `id` to the front of the frame's queue: a background one,
/// held or queued, when a frame asks for it; any queued one, a frame's
/// too, when the renderer `waited` for it (else a frame's job waited for
/// runs only after every one asked before it, and the renderer's thread
/// idles meanwhile). A running or finished one is in no queue.
#[cfg(not(target_arch = "wasm32"))]
fn promote(id: u64, waited: bool) {
    let pool = pool();
    let mut q = pool.queues.lock().unwrap_or_else(|e| e.into_inner());
    let job = if let Some(i) = q.held.iter().position(|(j, _)| *j == id) {
        Some(q.held.remove(i).1)
    } else if let Some(i) = q.background.iter().position(|(j, _)| *j == id) {
        q.background.remove(i).map(|(_, job)| job)
    } else if waited {
        let i = q.frame.iter().position(|(j, _)| *j == id);
        i.and_then(|i| q.frame.remove(i)).map(|(_, job)| job)
    } else {
        None
    };
    if let Some(job) = job {
        q.frame.push_front((id, job));
        pool.work.notify_one();
    }
}

/// Whether the job `id` is held for the second frame ([`State::Queued`]).
#[cfg(all(test, not(target_arch = "wasm32")))]
fn held(id: u64) -> bool {
    let q = pool().queues.lock().unwrap_or_else(|e| e.into_inner());
    q.held.iter().any(|(j, _)| *j == id)
}

/// One entry of the pipeline cache: a value (a pipeline, or a set of them)
/// made by its closure when first asked for, at most once.
pub(crate) struct Deferred<T> {
    value: OnceCell<T>,
    make: Cell<Option<Make<T>>>,
}

impl<T> Deferred<T> {
    fn with(make: Make<T>) -> Deferred<T> {
        Deferred {
            value: OnceCell::new(),
            make: Cell::new(Some(make)),
        }
    }

    /// One made on the renderer's thread when asked for (on the web, by the
    /// host's compiler like every entry).
    pub(crate) fn new<F: Fn() -> T + 'static>(make: F) -> Deferred<T> {
        let asker = std::any::type_name::<F>();
        Deferred::with(Make::Here(Box::new(move || logged(asker, &make))))
    }

    /// One the host's compiler makes when asked for: natively on the
    /// compile workers.
    pub(crate) fn anywhere<F: Fn() -> T + AnywhereOnce + 'static>(make: F) -> Deferred<T> {
        let asker = std::any::type_name::<F>();
        Deferred::with(Make::Anywhere(Box::new(move || logged(asker, &make))))
    }

    /// Whether it has been asked for (or made).
    pub(crate) fn asked(&self) -> bool {
        if self.value.get().is_some() {
            return true;
        }
        let make = self.make.take();
        let asked = matches!(make, Some(Make::Asked(_)));
        self.make.set(make);
        asked
    }
}

impl<T: AnywhereOnce + 'static> Deferred<T> {
    /// Its state, without asking for it.
    #[cfg(test)]
    pub(crate) fn state(&self) -> State {
        if self.poll().is_some() {
            return State::Ready;
        }
        if self.value.get().is_some() {
            return State::Creating;
        }
        let make = self.make.take();
        let state = match &make {
            Some(Make::Asked(pending)) if pending.held() => State::Queued,
            Some(Make::Asked(_)) => State::Creating,
            _ => State::Unasked,
        };
        self.make.set(make);
        state
    }

    /// Asks for it at `priority`, if it is not asked for yet; a background
    /// entry asked for at [`Priority::Frame`] is moved ahead.
    pub(crate) fn start_at(&self, priority: Priority) {
        if self.value.get().is_some() {
            return;
        }
        match self.make.take() {
            #[cfg(not(target_arch = "wasm32"))]
            Some(Make::Here(make)) => {
                let _ = self.value.set(made_here(make));
            }
            #[cfg(not(target_arch = "wasm32"))]
            Some(Make::Anywhere(make)) => self.make.set(Some(Make::Asked(spawn(priority, make)))),
            #[cfg(target_arch = "wasm32")]
            Some(Make::Here(make) | Make::Anywhere(make)) => match priority {
                Priority::Frame if web::asks_async() => {
                    self.make
                        .set(Some(Make::Asked(Pending(web::hold_frame(make)))));
                }
                Priority::Frame => {
                    let _ = self.value.set(web::here(&*make));
                }
                Priority::Background => {
                    self.make.set(Some(Make::Asked(Pending(web::hold(make)))));
                }
            },
            Some(Make::Asked(pending)) => {
                if priority == Priority::Frame {
                    pending.promote();
                }
                self.make.set(Some(Make::Asked(pending)));
            }
            None => {}
        }
    }

    /// The value once it is ready, without waiting or asking for it.
    pub(crate) fn poll(&self) -> Option<&T> {
        if self.value.get().is_none() {
            match self.make.take() {
                #[cfg(not(target_arch = "wasm32"))]
                Some(Make::Asked(pending)) => match pending.answer.try_recv() {
                    Ok(Ok(value)) => {
                        let _ = self.value.set(value);
                    }
                    Ok(Err(panic)) => std::panic::resume_unwind(panic),
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        self.make.set(Some(Make::Asked(pending)));
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        unreachable!("a compile worker always answers")
                    }
                },
                #[cfg(target_arch = "wasm32")]
                Some(Make::Asked(pending)) => match pending.0.ready() {
                    Some(value) => {
                        let _ = self.value.set(value);
                    }
                    None => self.make.set(Some(Make::Asked(pending))),
                },
                other => self.make.set(other),
            }
        }
        self.value.get()
    }

    /// The value, waited for: made now if it was not asked for, its worker
    /// waited for natively, made at once on the web if it still compiles.
    pub(crate) fn wait(&self) -> &T {
        self.value.get_or_init(|| {
            match self.make.take().expect("an entry not made has its maker") {
                #[cfg(not(target_arch = "wasm32"))]
                Make::Here(make) => made_here(make),
                #[cfg(not(target_arch = "wasm32"))]
                Make::Anywhere(make) => made_here(make),
                #[cfg(not(target_arch = "wasm32"))]
                Make::Asked(pending) => joined(pending),
                #[cfg(target_arch = "wasm32")]
                Make::Here(make) | Make::Anywhere(make) => web::here(&*make),
                #[cfg(target_arch = "wasm32")]
                Make::Asked(pending) => pending.0.wait(),
            }
        })
    }

    /// The value for a draw, by the frame's policy ([`FramePolicy`]): in a
    /// pass of a display frame that may skip ([`Skipping`]), asked for and
    /// `None` until it is ready (the draw is skipped and counted); anywhere
    /// else waited for.
    pub(crate) fn get(&self) -> Option<&T> {
        if !skips() {
            return Some(self.wait());
        }
        self.start_at(Priority::Frame);
        let value = self.poll();
        if value.is_none() {
            skipped();
        }
        value
    }

    /// The value for work whose output later frames keep, which leaves
    /// itself for a later frame while it is `None` (counted): asked for and
    /// `None` until it is ready in a display frame, waited for in a frame
    /// that waits.
    pub(crate) fn ready(&self) -> Option<&T> {
        if waits() {
            return Some(self.wait());
        }
        self.start_at(Priority::Frame);
        let value = self.poll();
        if value.is_none() {
            skipped();
        }
        value
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> Pending<T> {
    #[cfg(test)]
    fn held(&self) -> bool {
        held(self.id)
    }

    fn promote(&self) {
        promote(self.id, false);
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: 'static> Pending<T> {
    #[cfg(test)]
    fn held(&self) -> bool {
        use web::Release;
        self.0.held()
    }

    fn promote(&self) {
        self.0.promote();
    }
}

/// Something a frame can ask for ahead of the pass that uses it.
pub(crate) trait Start {
    /// Asks for it ([`Priority::Frame`]) if it is not asked for yet.
    fn start(&self);
}

impl<T: AnywhereOnce + 'static> Start for Deferred<T> {
    fn start(&self) {
        self.start_at(Priority::Frame);
    }
}

/// Asks for every one of `items` (see [`Start`]).
pub(crate) fn start_all(items: &[&dyn Start]) {
    for item in items {
        item.start();
    }
}

/// An entry per samples per pixel: a pipeline (or anything else) that
/// depends on the renderer's sample count, asked for at the current count
/// and again at the first use after [`Sampled::set_samples`] changed it
/// (`RenderQuality::msaa_samples`). One is kept per count (one sample,
/// [`crate::MSAA_SAMPLES`]), so going back costs nothing.
pub(crate) struct Sampled<T> {
    samples: Cell<u32>,
    cells: [OnceCell<Deferred<T>>; 2],
    make: SampledMake<T>,
}

enum SampledMake<T> {
    Here(std::rc::Rc<dyn Fn(u32) -> T>, &'static str),
    #[cfg(not(target_arch = "wasm32"))]
    Anywhere(Arc<dyn Fn(u32) -> T + Send + Sync>, &'static str),
}

impl<T: AnywhereOnce + 'static> Sampled<T> {
    pub(crate) fn new<F: Fn(u32) -> T + 'static>(samples: u32, make: F) -> Sampled<T> {
        let asker = std::any::type_name::<F>();
        Sampled::with(samples, SampledMake::Here(std::rc::Rc::new(make), asker))
    }

    /// One whose entries the host's compiler makes (natively on the compile
    /// workers).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn anywhere<F: Fn(u32) -> T + Anywhere + 'static>(
        samples: u32,
        make: F,
    ) -> Sampled<T> {
        let asker = std::any::type_name::<F>();
        Sampled::with(samples, SampledMake::Anywhere(Arc::new(make), asker))
    }

    /// One whose entries the host's compiler makes.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn anywhere(samples: u32, make: impl Fn(u32) -> T + 'static) -> Sampled<T> {
        Sampled::new(samples, make)
    }

    fn with(samples: u32, make: SampledMake<T>) -> Sampled<T> {
        Sampled {
            samples: Cell::new(samples),
            cells: [OnceCell::new(), OnceCell::new()],
            make,
        }
    }

    /// The entry at the current count.
    pub(crate) fn cell(&self) -> &Deferred<T> {
        let samples = self.samples.get();
        self.cells[(samples > 1) as usize].get_or_init(|| match &self.make {
            SampledMake::Here(make, asker) => {
                let (make, asker) = (make.clone(), *asker);
                Deferred::with(Make::Here(Box::new(move || {
                    logged(asker, || make(samples))
                })))
            }
            #[cfg(not(target_arch = "wasm32"))]
            SampledMake::Anywhere(make, asker) => {
                let (make, asker) = (make.clone(), *asker);
                Deferred::with(Make::Anywhere(Box::new(move || {
                    logged(asker, || make(samples))
                })))
            }
        })
    }

    pub(crate) fn start_at(&self, priority: Priority) {
        self.cell().start_at(priority);
    }

    pub(crate) fn poll(&self) -> Option<&T> {
        self.cell().poll()
    }

    #[cfg(test)]
    pub(crate) fn wait(&self) -> &T {
        self.cell().wait()
    }

    pub(crate) fn get(&self) -> Option<&T> {
        self.cell().get()
    }

    /// Draw at `samples` per pixel from now on.
    pub(crate) fn set_samples(&self, samples: u32) {
        self.samples.set(samples);
    }

    /// The samples per pixel it draws at.
    pub(crate) fn samples(&self) -> u32 {
        self.samples.get()
    }
}

impl<T: AnywhereOnce + 'static> Start for Sampled<T> {
    fn start(&self) {
        self.start_at(Priority::Frame);
    }
}

impl<T: AnywhereOnce + 'static> Warm for Sampled<T> {
    fn warm(&self) -> bool {
        self.cell().warm()
    }
}

/// A shader module parsed when the first pipeline that uses it is made,
/// on whichever thread that is ([`Deferred::anywhere`]).
pub(crate) struct ModuleCell {
    /// The optional features whose overrides its source declares.
    pub(crate) declared: crate::pipeline_features::Features,
    value: OnceLock<wgpu::ShaderModule>,
    make: Mutex<Option<MakeAnywhere<wgpu::ShaderModule>>>,
}

impl ModuleCell {
    /// The module, parsed now if it was not yet.
    pub(crate) fn get(&self) -> &wgpu::ShaderModule {
        self.value.get_or_init(|| {
            let make = self
                .make
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take()
                .expect("a module is parsed once, by its first use");
            make()
        })
    }
}

impl Deref for ModuleCell {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &wgpu::ShaderModule {
        self.get()
    }
}

/// A shader module parsed when the first pipeline that uses it is made.
pub(crate) type Module = Arc<ModuleCell>;

/// A WGSL module made on first use ([`Module`]).
pub(crate) fn module(device: &wgpu::Device, label: &'static str, source: String) -> Module {
    module_with(device, label, source, crate::shader)
}

/// [`module`] for a module in [`crate::UNBOUNDED_MODULES`].
pub(crate) fn unbounded_module(
    device: &wgpu::Device,
    label: &'static str,
    source: String,
) -> Module {
    module_with(device, label, source, crate::unbounded_shader)
}

// The web's wgpu objects are neither Send nor Sync; nothing there leaves
// the renderer's thread.
#[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
fn module_with(
    device: &wgpu::Device,
    label: &'static str,
    source: String,
    make: fn(&wgpu::Device, &str, String) -> wgpu::ShaderModule,
) -> Module {
    let device = device.clone();
    Arc::new(ModuleCell {
        declared: crate::pipeline_features::Features::declared(&source),
        value: OnceLock::new(),
        make: Mutex::new(Some(Box::new(move || make(&device, label, source)))),
    })
}

/// A shader module a deferred pipeline is made from: one already made, or a [`Module`].
pub(crate) trait AsModule {
    fn module(&self) -> &wgpu::ShaderModule;
}

impl AsModule for wgpu::ShaderModule {
    fn module(&self) -> &wgpu::ShaderModule {
        self
    }
}

impl AsModule for Module {
    fn module(&self) -> &wgpu::ShaderModule {
        self.get()
    }
}

/// Something [`Renderer::warm`](crate::Renderer::warm) can ask for ahead
/// of use.
pub(crate) trait Warm {
    /// Asks for it if it is not asked for yet; true when this call did.
    fn warm(&self) -> bool;
}

impl<T: AnywhereOnce + 'static> Warm for Deferred<T> {
    fn warm(&self) -> bool {
        if self.asked() {
            return false;
        }
        self.start();
        true
    }
}

/// Asks for the first of `items` not asked for yet; false when all are.
pub(crate) fn warm_one(items: &[&dyn Warm]) -> bool {
    items.iter().any(|w| w.warm())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn made_once_when_asked_and_warmed_in_order() {
        let made = std::rc::Rc::new(Cell::new(0));
        let m = made.clone();
        let a = Deferred::new(move || {
            m.set(m.get() + 1);
            7
        });
        assert_eq!(a.state(), State::Unasked);
        assert_eq!(made.get(), 0);
        assert_eq!(*a.wait(), 7);
        assert_eq!(a.get(), Some(&7));
        assert_eq!(made.get(), 1);
        let b = Deferred::new(|| "b");
        let c = Deferred::new(|| "c");
        assert!(warm_one(&[&a, &b, &c]));
        assert!(b.asked() && !c.asked());
        assert!(warm_one(&[&a, &b, &c]));
        assert!(!warm_one(&[&a, &b, &c]));
    }

    /// In a display frame a draw in a pass that may skip skips an entry
    /// still compiling and counts it, work that is kept leaves itself for a
    /// later frame, and every other use waits; a frame that waits waits
    /// everywhere. What the frame left is reported once it ends.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_display_frame_draws_without_what_compiles() {
        let gate = Arc::new(std::sync::Barrier::new(2));
        let g = gate.clone();
        let slow = Deferred::anywhere(move || {
            g.wait();
            5
        });
        {
            let _frame = FramePolicy::open(false);
            {
                let _pass = Skipping::open();
                assert_eq!(slow.get(), None, "skipped while it compiles");
                assert_eq!(slow.state(), State::Creating);
            }
            assert_eq!(slow.ready(), None, "kept work waits for a later frame");
        }
        assert_eq!(frame_skipped(), 2);
        gate.wait();
        {
            let _frame = FramePolicy::open(true);
            let _pass = Skipping::open();
            assert_eq!(slow.get(), Some(&5), "a frame that waits waits");
        }
        assert_eq!(frame_skipped(), 0);
        assert_eq!(slow.state(), State::Ready);
    }

    /// An entry asked for is made once, on a compile worker, and its first
    /// wait waits for it; a worker's panic reaches that wait.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn asked_entries_are_made_on_a_worker_and_waited_for() {
        let here = std::thread::current().id();
        let made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let m = made.clone();
        let a = Deferred::anywhere(move || {
            m.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::thread::current().id()
        });
        a.start();
        a.start();
        assert_ne!(*a.wait(), here);
        assert_eq!(made.load(std::sync::atomic::Ordering::SeqCst), 1);
        let s = Sampled::anywhere(4, |n| (n, std::thread::current().id()));
        s.start();
        assert_eq!(s.wait().0, 4);
        assert_ne!(s.wait().1, here);
        let p = Deferred::anywhere(|| -> u32 { panic!("compile failed") });
        p.start();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| *p.wait()));
        assert_eq!(
            panic.unwrap_err().downcast_ref::<&str>(),
            Some(&"compile failed")
        );
    }

    /// An entry dropped while asked for leaves no work behind: one still
    /// queued is dropped with it, with what its closure owns, and one a
    /// worker is making is waited for (it is never left inside the driver
    /// for the process's exit to tear down under it).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_dropped_entry_owns_its_compile() {
        use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
        let resource = Arc::new(());
        let captured = resource.clone();
        let held = Deferred::anywhere(move || captured.clone());
        held.start_at(Priority::Background);
        assert_eq!(held.state(), State::Queued);
        drop(held);
        assert_eq!(Arc::strong_count(&resource), 1, "a held compile is dropped");
        let captured = resource.clone();
        let sampled = Sampled::anywhere(4, move |_| captured.clone());
        sampled.start_at(Priority::Background);
        drop(sampled);
        assert_eq!(
            Arc::strong_count(&resource),
            1,
            "a held sampled compile is dropped"
        );
        let (began, done) = (
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        );
        let (b, d) = (began.clone(), done.clone());
        let running = Deferred::anywhere(move || {
            b.store(true, SeqCst);
            std::thread::sleep(std::time::Duration::from_millis(200));
            d.store(true, SeqCst);
        });
        running.start();
        while !began.load(SeqCst) {
            std::thread::yield_now();
        }
        drop(running);
        assert!(done.load(SeqCst), "a running compile is waited for");
    }
}
