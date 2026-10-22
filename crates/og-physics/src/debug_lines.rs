//! The physics overlay's lines (og.debug's `physics` view, the command
//! channel's `debug:physics`): rapier's debug-render pipeline over the
//! simulation as the last step left it: collider shapes, joints and
//! contacts, each segment with an RGBA colour. Included by both profiles
//! (`dim3.rs`, `dim2.rs`), as `sim.rs` is; the 2D profile's lines lie in
//! the xy plane at z 0.

use super::{Sim, rp, unv};
use og_core::glam::Vec3;

/// A segment and its linear RGBA colour.
pub(crate) type DebugLine = (Vec3, Vec3, [f32; 4]);

/// An HSLA colour (rapier's debug colours) as RGBA.
fn rgba([h, s, l, a]: [f32; 4]) -> [f32; 4] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    [r + m, g + m, b + m, a]
}

struct Collect(Vec<DebugLine>);

impl rp::DebugRenderBackend for Collect {
    fn draw_line(
        &mut self,
        _object: rp::DebugRenderObject,
        a: rp::Vector,
        b: rp::Vector,
        color: rp::DebugColor,
    ) {
        self.0.push((unv(a), unv(b), rgba(color)));
    }
}

impl Sim {
    /// The overlay's segments: collider shapes, joints and contacts.
    pub(crate) fn debug_lines(&self) -> Vec<DebugLine> {
        lines(self)
    }
}

fn lines(sim: &Sim) -> Vec<DebugLine> {
    let mode = rp::DebugRenderMode::COLLIDER_SHAPES
        | rp::DebugRenderMode::JOINTS
        | rp::DebugRenderMode::CONTACTS;
    let mut pipeline = rp::DebugRenderPipeline::new(Default::default(), mode);
    let mut out = Collect(Vec::new());
    let w = &sim.world;
    pipeline.render(
        &mut out,
        &w.bodies,
        &w.colliders,
        &w.impulse_joints,
        &w.multibody_joints,
        &w.narrow_phase,
        &w.soft_bodies,
    );
    out.0
}

#[cfg(test)]
mod tests {
    #[test]
    fn hsla_primaries() {
        assert_eq!(super::rgba([0.0, 1.0, 0.5, 1.0]), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(super::rgba([120.0, 1.0, 0.5, 0.5]), [0.0, 1.0, 0.0, 0.5]);
    }
}
