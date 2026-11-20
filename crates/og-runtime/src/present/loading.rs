//! The loading screen (IDL `loading`, guides/loading.md §13): who draws it
//! (project.ogt's `publish { loading }`), the stages the game reports and
//! whether it is done. Presentation only: nothing here is world state or
//! reaches a tick. The host tells its page ([`crate::Hooks::loading`]); a
//! native host has none, and keeps what was reported for og.loading.status
//! and the probe.

use crate::engine::{Engine, engine, hooks};
use crate::host::wire::{LoadingScreen, LoadingStageDesc, LoadingStatus};
use std::cell::RefCell;

/// The stage id og.loading.done reports.
pub(crate) const DONE: &str = "done";

/// What the game reported, on one engine.
#[derive(Default)]
pub(crate) struct ModuleState(RefCell<Reported>);

#[derive(Clone, Default)]
struct Reported {
    stage: String,
    fraction: f32,
    label: String,
    done: bool,
}

/// project.ogt's choice (`engine` without one).
fn screen(eng: &Engine) -> LoadingScreen {
    match eng
        .state
        .borrow()
        .project
        .as_ref()
        .and_then(|p| p.loading.as_deref())
    {
        Some("game") => LoadingScreen::Game,
        Some("page") => LoadingScreen::Page,
        _ => LoadingScreen::Engine,
    }
}

fn idle(eng: &Engine) -> bool {
    // A model still fading in is presentation work too.
    let now = crate::present::reveal::tick(eng);
    let drawing = {
        let rs = eng.render.borrow();
        rs.decoded.preparing() || rs.reveal.active(now) || rs.compile || compiling(&rs)
    };
    !drawing
        && !eng.assets.borrow().stream.preparing()
        && !crate::world::remote::pending(eng)
        && hooks().is_none_or(|h| h.loading_idle(eng.id))
}

/// The renderer's pipelines compiled off the frame, and how many are made
/// (og-render's `pipeline_counts`); none without a renderer.
fn pipelines(rs: &crate::present::extract::RenderState) -> (u32, u32) {
    if rs.backend.is_some() {
        og_render::pipeline_counts()
    } else {
        (0, 0)
    }
}

fn compiling(rs: &crate::present::extract::RenderState) -> bool {
    let (requested, ready) = pipelines(rs);
    ready < requested
}

/// At most this many pipelines compile at once for og.render.compile, so
/// the ones a frame needs are not queued behind all of them.
const COMPILING: u32 = 8;
/// At most this many made a frame: natively each is made on the frame's
/// thread, and the loading screen keeps drawing.
const MADE_A_FRAME: u32 = 4;

/// After a frame was drawn: what og.render.compile asked for, a few
/// pipelines a frame, until the renderer has none left to make.
pub(crate) fn compile_step(rs: &mut crate::present::extract::RenderState) {
    if !rs.compile {
        return;
    }
    let Some(backend) = rs.backend.as_mut() else {
        rs.compile = false;
        return;
    };
    for _ in 0..MADE_A_FRAME {
        let (requested, ready) = og_render::pipeline_counts();
        if requested - ready >= COMPILING {
            return;
        }
        if !backend.warm() {
            rs.compile = false;
            return;
        }
    }
}

/// `og.render.compile`: the renderer's pipelines, a few a frame, and the
/// work a played tick first reaches ([`warm_tick`]) after the next frame.
pub fn og_render_compile(world: u32) {
    if let Ok(eng) = engine(world) {
        let mut rs = eng.render.borrow_mut();
        rs.compile = true;
        rs.warm_tick = true;
    }
}

/// Behind the play gate and for og.render.compile, outside a tick: the
/// first-use work of what a played tick reaches, done ahead so the first
/// played frame does not do it. The game's UI documents are built, laid out
/// and painted once on a copy of the UI (with no input), and the physics
/// builds a query view of the world, which makes its height fields' chunk
/// shapes into the cache its step takes them from. Neither runs a game
/// system or a tick: the world, the UI, its stores and every queue a tick
/// reads are as they were, so the ticks after it are the same. What it
/// leaves is caches: compiled code, shaped text, memoised styles and the
/// chunk shapes, each a function of what it was made from.
pub fn warm_tick(id: u32) {
    let Ok(eng) = engine(id) else { return };
    if eng.stepping.get() {
        return;
    }
    let (ui, physics) = {
        let st = eng.state.borrow();
        (st.ui.clone(), st.physics.clone())
    };
    if let Some(ui) = ui {
        let viewport = crate::bind::modules::viewport_of(id);
        let mut copy = ui.lock().clone();
        copy.begin(og_ui::InputFrame::default(), viewport);
        copy.build_layers(&crate::bind::modules::docs::env(id));
        copy.end();
    }
    if let (Some(physics), Ok(world)) = (physics, eng.world()) {
        // A world without the physics settings has nothing to build.
        let _ = physics.query(world);
    }
}

/// `og.loading.stage`.
pub fn og_loading_stage(world: u32, desc: Option<String>) -> Result<(), String> {
    let d: LoadingStageDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    if !(0.0..=1.0).contains(&d.fraction) {
        return Err(format!(
            "LoadingStageDesc.fraction is {}; give how far the stage is, 0 to 1",
            d.fraction
        ));
    }
    if d.id == DONE {
        return og_loading_done(world);
    }
    let eng = engine(world)?;
    {
        let mut r = eng.modules.loading.0.borrow_mut();
        r.stage.clone_from(&d.id);
        r.fraction = d.fraction;
        r.label.clone_from(&d.label);
    }
    if let Some(h) = hooks() {
        h.loading(world, &d.id, f64::from(d.fraction), &d.label);
    }
    Ok(())
}

/// `og.loading.done`: once; again does nothing.
pub fn og_loading_done(world: u32) -> Result<(), String> {
    let eng = engine(world)?;
    {
        let mut r = eng.modules.loading.0.borrow_mut();
        if r.done {
            return Ok(());
        }
        r.done = true;
        r.fraction = 1.0;
    }
    if let Some(h) = hooks() {
        h.loading(world, DONE, 1.0, "");
    }
    Ok(())
}

/// `og.loading.idle`.
pub fn og_loading_idle(world: u32) -> bool {
    engine(world).is_ok_and(|eng| idle(&eng))
}

/// `og.loading.status`.
pub fn og_loading_status(world: u32) -> LoadingStatus {
    let Ok(eng) = engine(world) else {
        return LoadingStatus::default();
    };
    let r = eng.modules.loading.0.borrow().clone();
    let (pipelines_requested, pipelines_ready) = pipelines(&eng.render.borrow());
    LoadingStatus {
        pipelines_requested,
        pipelines_ready,
        screen: screen(&eng),
        stage: r.stage,
        fraction: r.fraction,
        label: r.label,
        done: r.done,
        idle: idle(&eng),
    }
}
