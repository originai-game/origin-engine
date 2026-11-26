//! The engine side of `og render` (docs/design.md §2.9): the frame the
//! host's loop drew last, read back every frame through the probe's
//! [`og_probe::Capture::frame`] hook (no second draw, no PNG, no JSON-RPC).
//! A development member: the hosts export it outside `publish` builds only.

use crate::engine::engine;

/// The last frame drawn as RGBA8 rows; refused without a renderer, before
/// the first frame, or inside a system.
pub fn render_frame(id: u32) -> Result<og_probe::RawFrame, String> {
    let eng = engine(id)?;
    if eng.stepping.get() {
        return Err("render_frame was called from inside a system".into());
    }
    let mut probe = eng
        .probe
        .try_borrow_mut()
        .map_err(|_| "the probe is busy; read the frame between frames".to_string())?;
    probe.frame_rgba().ok_or_else(|| {
        "no frame to read: the engine has no renderer, no frame was drawn yet, or (the web) the last frame's readback has not landed: ask again before the next frame"
            .to_string()
    })
}
