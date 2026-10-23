//! The engine side of `og render` (docs/design.md §2.9): the frame the
//! host's loop drew last, read back every frame through the probe's
//! [`og_probe::Capture::frame`] hook (no second draw, no PNG, no JSON-RPC),
//! and the render snapshots a job of `og render --jobs N` starts from
//! ([`og_probe::Probe::render_snapshot`]). Development members: the hosts
//! export them outside `publish` builds only.

use super::engine::engine;
use super::probe as rprobe;

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

/// The world as a render snapshot (the world exactly and the probe's
/// pointer), for another process of the same game and script.
pub fn render_snapshot(id: u32) -> Result<Vec<u8>, String> {
    let eng = engine(id)?;
    rprobe::with_probe(&eng, false, |p, w| p.render_snapshot(w))
}

/// Restores a render snapshot another process took; the steps of this
/// process's script before its tick are dropped. The tick.
pub fn render_restore(id: u32, blob: &[u8]) -> Result<u64, String> {
    let eng = engine(id)?;
    let tick = rprobe::with_probe(&eng, true, |p, w| p.render_restore(w, blob))?
        .map_err(|e| e.reason().to_string())?;
    rprobe::resync_names(&eng);
    Ok(tick)
}
