//! Point clouds (three.js `Points` with a `PointsMaterial`): a square per
//! point facing the camera, `size` CSS pixels wide, or world units wide
//! and shrinking with distance (`size_attenuation`), coloured per point or
//! all alike. Drawn by the lines pass (`lines.rs`, one instance a point).

use crate::lines::{FLAG_POINT, FLAG_SCREEN, FLAG_WORLD, LineInstance, LineLayer};
use glam::{Mat4, Vec3};

/// One set of points, drawn this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Points {
    /// The positions, in the set's space (`transform`), or CSS pixels from
    /// the top left with y down when `screen`.
    pub positions: Vec<Vec3>,
    /// Per point linear RGBA; empty: `color` for all.
    pub colors: Vec<[f32; 4]>,
    pub color: [f32; 4],
    /// The side of each square: CSS pixels, or world units with `size_attenuation`.
    pub size: f32,
    pub size_attenuation: bool,
    /// Hidden behind nearer geometry; false draws it over the scene.
    pub depth_test: bool,
    pub screen: bool,
    /// Where it is drawn: in the scene (tone mapped with it) or over the
    /// finished frame.
    pub layer: LineLayer,
    pub transform: Mat4,
}

impl Default for Points {
    fn default() -> Self {
        Points {
            positions: Vec::new(),
            colors: Vec::new(),
            color: [1.0; 4],
            size: 1.0,
            size_attenuation: true,
            depth_test: true,
            screen: false,
            layer: LineLayer::Scene,
            transform: Mat4::IDENTITY,
        }
    }
}

impl Points {
    /// The pass this set draws in (screen points always over the frame).
    pub(crate) fn layer(&self) -> LineLayer {
        if self.screen {
            LineLayer::Overlay
        } else {
            self.layer
        }
    }

    pub(crate) fn instances(&self, out: &mut Vec<LineInstance>) {
        let mut flags = FLAG_POINT;
        if self.screen {
            flags |= FLAG_SCREEN;
        } else if self.size_attenuation {
            flags |= FLAG_WORLD;
        }
        for (i, &p) in self.positions.iter().enumerate() {
            let p = if self.screen {
                p
            } else {
                self.transform.transform_point3(p)
            };
            if !p.is_finite() {
                continue;
            }
            let color = self.colors.get(i).copied().unwrap_or(self.color);
            let at = p.extend(0.0).to_array();
            out.push(LineInstance {
                a: at,
                b: at,
                color_a: color,
                color_b: color,
                params: [self.size.max(0.0), flags as f32, 0.0, 0.0],
            });
        }
    }
}
