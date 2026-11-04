//! The FFT sea's persistent foam (`shaders/ocean_foam.wgsl`), ported from
//! web-ocean's world foam clipmaps (`js/main.js` `writeFoamUniforms`,
//! `js/wgsl/sim.js` `FOAM_UPDATE`, `FOAM_TEX`; MIT, third_party/ocean/).
//!
//! Two camera-centred levels of [`RES`]² texels ([`LEVEL_SIZE`] metres),
//! addressed toroidally in the absolute world, held in the fields array's
//! layers after the cascades (so the water pass reads them through its one
//! fields binding), with the foam's micro-structure in the layer after them.
//! Each step injects foam where the drawn surface folds, decays it and
//! drifts it downwind.
//!
//! Driven by the tick's clock, never the wall's: a step is one
//! [`STEP`] of tick time, its waves the FFT's and og_core's at that step's
//! time. The history restarts (empty) and is warmed up over [`WARM`] of tick
//! time before the frame on a fresh renderer, a new sea, a jump back in
//! time (a restore) or a jump forward of more than [`GAP`] steps, so a frame
//! is a function of the tick, the camera and the sea whatever the wall
//! clock: [`warm_steps`] is that schedule.

use glam::DVec2;

/// Texels along a level's edge.
pub const RES: u32 = 256;
/// Each level's edge (m): its texel is 0.25 m and 2 m.
pub const LEVEL_SIZE: [f64; 2] = [64.0, 512.0];
/// One step of the tick's time (s).
pub const STEP: f64 = 1.0 / 30.0;
/// The foam's decay (1/s; web-ocean's 0.22): τ about 4.5 s.
pub const DECAY: f32 = 0.22;
/// The warm-up after a restart: five τ, where what is left of the oldest
/// foam is under 1 %.
pub const WARM: f64 = 5.0 / DECAY as f64;
/// The warm-up's last seconds run every step; before them every
/// [`COARSE`]th step (the older foam is a faint fraction of the newer).
const FINE: i64 = 60;
const COARSE: i64 = 8;
/// A jump forward of more steps than this restarts the history.
pub const GAP: i64 = FINE;
/// A step back of up to this many steps (a frame between ticks after one
/// at the tick, under a tick of a 4 Hz game) keeps the history; further
/// back (a restore) restarts it.
pub const HOLD: i64 = 8;

/// The steps of a warm-up ending at step `now`: each the index of the
/// [`STEP`] it arrives at and the steps it spans since the one before.
/// Every [`COARSE`]th step from [`WARM`] before `now`, then every step of
/// the last [`FINE`]. Aligned to absolute step indices, so the same `now`
/// gives the same schedule.
pub fn warm_steps(now: i64) -> Vec<(i64, i64)> {
    let warm = (WARM / STEP).round() as i64;
    let fine_from = now - FINE + 1;
    let first = (now - warm).div_euclid(COARSE) * COARSE;
    let coarse = (first..fine_from).step_by(COARSE as usize);
    let mut out: Vec<(i64, i64)> = Vec::new();
    for k in coarse.chain(fine_from..=now) {
        let span = out.last().map_or(COARSE, |&(last, _)| k - last);
        out.push((k, span));
    }
    out
}

/// The steps that bring a history last stepped at `last` to step `now`,
/// and whether the history restarts first: each step since `last` when
/// `now` is at most [`GAP`] ahead, none when it is at most [`HOLD`]
/// behind (it holds), otherwise the warm-up of a restart.
pub fn schedule(last: Option<i64>, now: i64) -> (Vec<(i64, i64)>, bool) {
    match last {
        Some(last) if now >= last && now - last <= GAP => {
            (((last + 1)..=now).map(|k| (k, 1)).collect(), false)
        }
        Some(last) if now < last && last - now <= HOLD => (Vec::new(), false),
        _ => (warm_steps(now), true),
    }
}

/// What a step reads of the sea besides its waves (`FoamParams` in
/// ocean_foam.wgsl, frame coordinates).
#[derive(Clone, Copy, Debug)]
pub struct FoamSea {
    /// The surface's centre (frame xz) and the tick's grid's even cell (m).
    pub tick: [f32; 4],
    /// The surface's half size (xy).
    pub half: [f32; 4],
    /// As `WaterGpu::cascades` and `bands`.
    pub cascades: [[f32; 4]; 3],
    pub bands: [f32; 4],
    /// The wind (m/s) in the frame's xz.
    pub wind: glam::Vec2,
    /// Per level the Jacobian (as that level resolves it) under which foam
    /// is injected, and the injection's gain.
    pub thresholds: [f32; 2],
    pub gain: f32,
}

/// `FoamParams` in ocean_foam.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct FoamGpu {
    snap: [[i32; 4]; 2],
    base: [[f32; 4]; 2],
    p0: [f32; 4],
    p1: [f32; 4],
    tick: [f32; 4],
    half: [f32; 4],
    thresholds: [f32; 4],
    cascades: [[f32; 4]; 3],
    bands: [f32; 4],
    swell: [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES],
    shore: [[f32; 4]; 3],
}

/// The camera's texel on each level (absolute world xz over the texel).
pub(super) fn snaps(eye: DVec2) -> [[i32; 2]; 2] {
    std::array::from_fn(|l| {
        let ts = LEVEL_SIZE[l] / f64::from(RES);
        [(eye.x / ts).floor() as i32, (eye.y / ts).floor() as i32]
    })
}

/// A step's uniforms: the camera's texels now and before (`prev`: None
/// after a restart, a window far away, so nothing carries over).
#[allow(clippy::too_many_arguments)]
pub(super) fn step_gpu(
    now: [[i32; 2]; 2],
    prev: Option<[[i32; 2]; 2]>,
    origin: DVec2,
    dt: f32,
    cascades: u32,
    sea: &FoamSea,
    swell: [[f32; 4]; 2 * og_core::ocean::OCEAN_WAVES],
    shore: [[f32; 4]; 3],
) -> FoamGpu {
    let far = 1 << 24;
    let snap = std::array::from_fn(|l| {
        let p = prev.map_or([now[l][0] + far, now[l][1] + far], |p| p[l]);
        [now[l][0], now[l][1], p[0], p[1]]
    });
    let base = std::array::from_fn(|l| {
        let ts = LEVEL_SIZE[l] / f64::from(RES);
        [
            (f64::from(now[l][0]) * ts - origin.x) as f32,
            (f64::from(now[l][1]) * ts - origin.y) as f32,
            ts as f32,
            0.0,
        ]
    });
    // web-ocean: foam drifts at 3 % of the wind.
    let drift = sea.wind * 0.03;
    FoamGpu {
        snap,
        base,
        p0: [dt, DECAY, 0.0, sea.gain],
        p1: [1.0, drift.x, drift.y, cascades as f32],
        tick: sea.tick,
        half: sea.half,
        thresholds: [sea.thresholds[0], sea.thresholds[1], 0.0, 0.0],
        cascades: sea.cascades,
        bands: sea.bands,
        swell,
        shore,
    }
}

/// Per level, what water.wgsl reads it by: one over its edge (1/m), the uv
/// of the frame's origin on it (whole repeats dropped, f64), its edge (m).
pub fn levels(origin: DVec2) -> [[f32; 4]; 2] {
    std::array::from_fn(|l| {
        let size = LEVEL_SIZE[l];
        [
            (1.0 / size) as f32,
            (origin.x / size).rem_euclid(1.0) as f32,
            (origin.y / size).rem_euclid(1.0) as f32,
            size as f32,
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The warm-up spans five τ of the tick's time before the frame, ends on
    /// it with single steps, is the same for the same frame, and costs some
    /// 140 steps rather than the 680 of stepping it all.
    #[test]
    fn the_warm_up_covers_five_decays_and_ends_on_the_frame() {
        let now = 12_345;
        let s = warm_steps(now);
        assert_eq!(s, warm_steps(now));
        let (first, _) = s[0];
        assert!(((now - first) as f64 * STEP - WARM).abs() < COARSE as f64 * STEP);
        assert_eq!(*s.last().unwrap(), (now, 1));
        assert!(s.iter().rev().take(FINE as usize - 1).all(|&(_, n)| n == 1));
        // Consecutive: each step spans the time since the one before.
        for w in s.windows(2) {
            assert_eq!(w[0].0 + w[1].1, w[1].0);
            assert!(w[1].1 >= 1 && w[1].1 <= COARSE);
        }
        assert!(s.len() < 150, "{} steps", s.len());
        // Negative times (a scene's first frame) too.
        let s0 = warm_steps(0);
        assert_eq!(*s0.last().unwrap(), (0, 1));
        assert!(s0[0].0 < -600);
    }

    /// Frames between ticks (the sea's clock at the display's rate) step
    /// the history on; a capture at the tick between two of them steps it
    /// back under a tick, which holds it (the code before restarted the
    /// warm-up, some 140 submits, on any step back); a restore further
    /// back restarts it.
    #[test]
    fn a_step_back_under_a_tick_holds_the_history() {
        assert_eq!(schedule(Some(100), 102), (vec![(101, 1), (102, 1)], false));
        assert_eq!(schedule(Some(100), 100), (vec![], false));
        assert_eq!(schedule(Some(100), 99), (vec![], false));
        assert_eq!(schedule(Some(100), 100 - HOLD), (vec![], false));
        assert_eq!(
            schedule(Some(100), 99 - HOLD),
            (warm_steps(99 - HOLD), true)
        );
        assert_eq!(
            schedule(Some(100), 101 + GAP),
            (warm_steps(101 + GAP), true)
        );
        assert_eq!(schedule(None, 7), (warm_steps(7), true));
    }

    /// The water pass's uv of a world point on a level is the update's
    /// toroidal index over the edge: the frame's point plus the origin.
    #[test]
    fn a_level_is_addressed_by_the_absolute_world() {
        let origin = DVec2::new(12_345.6, -7_890.1);
        let lv = levels(origin);
        let p = glam::Vec2::new(3.0, -40.0);
        for l in 0..2 {
            let uv = (p.x * lv[l][0] + lv[l][1]).rem_euclid(1.0);
            let ts = LEVEL_SIZE[l] / f64::from(RES);
            let g = ((f64::from(p.x) + origin.x) / ts).floor();
            let index = g.rem_euclid(f64::from(RES));
            assert!(
                ((uv as f64) * f64::from(RES) - index).abs() < 1.0,
                "level {l}"
            );
        }
    }
}
