//! The presentation barrier: where a frame meets the presentation's async
//! stages, by one policy ([`FrameKind`]).
//!
//! Each stage does work off the frame and owes it by a tick: the textures
//! being prepared (the texture workers), og.motion's results (its GPU
//! batches), the terrain tiles a view wants and the glyphs a frame asks
//! for. The pipelines a fade draws with are the reveal's
//! (`crate::present::reveal`, which reads the same policy). A frame either
//! settles them, every one at the world's tick, or steps them with what
//! has come back:
//!
//! - a display frame and a scripted one never wait: the world goes on
//!   while results are late, and the frame draws what is ready;
//! - a recorded frame ([`FrameKind::Recorded`], `og render`) and a probe
//!   capture settle everything, so two runs draw the same frames.
//!
//! [`busy`] is the other side of the same barrier: whether any stage, or
//! the streaming that feeds them, still has work (`og.loading.idle`).
//!
//! Presentation only (docs/design.md §2.4): nothing here reaches a tick.

use crate::engine::Engine;
#[cfg(doc)]
use crate::present::display::FrameKind;

/// Before a frame is drawn at the world's tick: its stages' due work,
/// settled when the frame's kind settles, stepped otherwise.
pub(crate) fn frame(eng: &Engine) {
    if eng.render.borrow().display.kind.settles() {
        settle(eng);
    } else if motion(eng) {
        crate::bind::motion::present::step(eng);
    }
}

/// Every stage's work due by the world's tick, for a frame that settles
/// and a probe capture: the textures being prepared (natively waited for;
/// the web cannot wait for its Worker), og.motion's results, and every
/// terrain tile and glyph the next draw wants. False while og.motion is
/// still behind where it cannot wait (the web): no frame, ask again.
pub(crate) fn settle(eng: &Engine) -> bool {
    textures(eng);
    if motion(eng) && !crate::bind::motion::present::settle(eng) {
        return false;
    }
    let mut rs = eng.render.borrow_mut();
    rs.terrain.settle = true;
    rs.glyphs_settle = true;
    true
}

/// Whether a stage, or what feeds one, still has work: textures being
/// prepared, a model still fading in (or held for its pipelines),
/// pipelines compiling, a stream group loading or preparing, a tick held
/// for a cell's files, a remote call, or the host's own (its page).
pub(crate) fn busy(eng: &Engine) -> bool {
    let now = crate::present::reveal::tick(eng);
    let drawing = {
        let rs = eng.render.borrow();
        rs.decoded.preparing()
            || rs.reveal.active(now)
            || rs.compile
            || crate::present::loading::compiling(&rs)
    };
    let streaming = {
        let a = eng.assets.borrow();
        a.stream.preparing() || a.stream.holding()
    };
    drawing || streaming || crate::engine::hooks().is_some_and(|h| !h.loading_idle(eng.id))
}

/// og.motion only once the game named it (its og.MotionAgent registered,
/// engine/lazy.rs): before that it has no agents and no goals, and a game
/// that never does never reaches the motion module (docs/web-export.md
/// §Modules).
fn motion(eng: &Engine) -> bool {
    eng.world()
        .is_ok_and(|w| w.id_by_name(crate::bind::motion::AGENT).is_ok())
}

/// Natively, until no texture is being prepared: the frame draws the
/// models and maps the world holds instead of hiding a model or drawing
/// the placeholder for as long as the worker happened to take. A model of
/// a stream group still waits for its group's upgrade, as on display.
fn textures(eng: &Engine) {
    #[cfg(not(target_arch = "wasm32"))]
    while eng.render.borrow().decoded.preparing() {
        std::thread::sleep(std::time::Duration::from_millis(1));
        crate::world::assets::retranscode(eng);
    }
    #[cfg(target_arch = "wasm32")]
    let _ = eng;
}
