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
    !crate::world::remote::pending(eng) && hooks().is_none_or(|h| h.loading_idle(eng.id))
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
    LoadingStatus {
        screen: screen(&eng),
        stage: r.stage,
        fraction: r.fraction,
        label: r.label,
        done: r.done,
        idle: idle(&eng),
    }
}
