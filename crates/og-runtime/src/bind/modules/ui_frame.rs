//! The UI frame: what `og.ui.begin` read for this tick's UI, before the
//! game's systems. og.ui's frame answers (viewport, pointer, direction,
//! clock) and the TSX tree's layout and input read it; nothing else holds
//! them.

use super::*;
use og_input::frame::InputFrame;

/// One tick's UI frame (`State::ui_frame`), kept until the next begins.
#[derive(Clone, Debug, Default)]
pub(crate) struct UiFrame {
    /// The play surface in CSS px, as the host reports it.
    pub css: Viewport,
    /// The og-ui unit it lays out in, CSS px (docs/ui.md §6.1).
    pub unit: f32,
    /// The tick's input, in units.
    pub input: InputFrame,
    /// Frames built: the tick this frame belongs to, counting from 1.
    pub tick: u64,
    /// Ticks per second.
    pub rate: u32,
    /// The active locale writes right to left.
    pub rtl: bool,
}

impl UiFrame {
    /// The play surface in units, as the frame lays out on it.
    pub(crate) fn viewport(&self) -> Viewport {
        self.css.zoomed(self.unit)
    }

    /// The UI's clock in seconds: frames over the tick rate.
    pub(crate) fn seconds(&self) -> f64 {
        self.tick as f64 / f64::from(self.rate.max(1))
    }
}

/// The unit engine `eng`'s UI lays out in on viewport `v`: the game's
/// `unitScale`, else the surface's.
pub(crate) fn unit_of(eng: &Engine, v: &Viewport) -> f32 {
    let u: wire::UiDesc = wire::value(&eng.state.borrow().ui_desc, "desc").unwrap_or_default();
    og_ui_model::surface_unit(u.unit_scale, v)
}

/// Records engine `engine_id`'s UI frame for this tick.
pub(crate) fn record(engine_id: u32, input: &InputFrame, css: Viewport, time: &og_core::Time) {
    let Ok(eng) = engine(engine_id) else { return };
    let unit = unit_of(&eng, &css);
    let frame = UiFrame {
        css,
        unit,
        input: input.clone().zoomed(unit),
        tick: time.tick + 1,
        rate: time.rate,
        rtl: crate::bind::i18n::rtl(engine_id),
    };
    eng.state.borrow_mut().ui_frame = frame;
}

/// Runs `f` on the frame of the engine whose UI frame is open; outside a
/// frame, logs a warning on the engine that opened one last and answers
/// `None`.
pub(crate) fn with_frame<T>(call: &str, f: impl FnOnce(&UiFrame) -> T) -> Option<T> {
    let eng = super::ui_desc::building(call)?;
    let st = eng.state.borrow();
    Some(f(&st.ui_frame))
}
