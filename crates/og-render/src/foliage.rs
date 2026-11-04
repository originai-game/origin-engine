//! Cooked branch/leaf attributes and the wind contract shared by tree passes.
//!
//! Anchor-coherent detail motion follows SeedThree (MIT, SkyeShark 2026;
//! third_party/vegetation/SeedThree.LICENSE). Conservative deformation bounds
//! follow threejs-nanites' per-type wind-margin approach (MIT, Victor Zakharov
//! 2026; third_party/vegetation/threejs-nanites.LICENSE), derived here for
//! rotations rather than its translations. GPU Gems 3 chapter 16 and Pivot
//! Painter are technique references only; no Epic/NVIDIA source is included.

use crate::{WIND_WAVELENGTH, Wind};
use glam::Vec3;

/// Binding-free deformation include for meshlet and conventional vertex paths.
/// Prepend [`WIND_WGSL`] once: the gust sampler is shared with grass.
pub const WGSL: &str = wgsl!("meshlet_foliage.wgsl");
pub const WIND_WGSL: &str = wgsl!("wind_common.wgsl");
/// The simulated wind field's lean the trunk bends by (after [`WIND_WGSL`]).
pub const WIND_LEAN_WGSL: &str = wgsl!("wind_lean.wgsl");

/// Both drawn states, shared by meshlets and conventional instancing.
pub(crate) fn foliage_winds(
    scene: &crate::RenderScene,
    previous: Option<(Wind, u64, f32)>,
    grid: crate::wind_field::WindGrid,
) -> ([Wind; 2], [FoliageWind; 2]) {
    let current = scene.wind.unwrap_or(Wind {
        strength: 0.0,
        ..Wind::default()
    });
    let (old, tick, rate) = previous.unwrap_or((current, scene.tick, scene.tick_rate));
    (
        [current, old],
        [
            FoliageWind::new(
                current,
                scene.tick,
                scene.tick_rate,
                0.0,
                scene.origin,
                grid,
            ),
            FoliageWind::new(old, tick, rate, 0.0, scene.origin, grid),
        ],
    )
}

/// `_OG_PIVOT` and `_OG_ANCHOR`, in mesh-local space. The eight floats travel
/// together through cooking and vertex remapping, separately from the ordinary
/// vertex stream. A leaf and the wood at its anchor share the branch pivot.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FoliageVertex {
    /// xyz: branch pivot; w: stiffness (0 flexible, 1 rigid).
    pub branch: [f32; 4],
    /// xyz: leaf anchor; w: flutter (0 for bark, up to 1 for leaves).
    pub leaf: [f32; 4],
}

impl Default for FoliageVertex {
    fn default() -> Self {
        Self {
            branch: [0.0, 0.0, 0.0, 1.0],
            leaf: [0.0; 4],
        }
    }
}

/// `meshlet_foliage.wgsl`'s explicit input; no bindings in that include.
/// Build the previous value from the last *drawn* wind, tick and rate, with the
/// current render origin (previous transforms have already been rebased).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FoliageWind {
    pub dir_strength: [f32; 4],
    /// gust, model height, and the two reduced travelling-wave phases.
    pub gust_top: [f32; 4],
    /// xyz: render origin; w: travelling-wave wavenumber.
    pub origin: [f32; 4],
    /// Its own drawn state's gust scroll, not the current wind at an old tick.
    field: crate::wind::WindFieldGpu,
}

impl FoliageWind {
    /// og.Wind at `tick`, read with the simulated field of grid `grid`
    /// (`wind_field.rs`; [`crate::wind_field::NO_GRID`]: analytic alone).
    pub fn new(
        wind: Wind,
        tick: u64,
        tick_rate: f32,
        top: f32,
        origin: Vec3,
        grid: crate::wind_field::WindGrid,
    ) -> Self {
        let phase = crate::wind_phases(&wind, tick, tick_rate);
        let direction = Vec3::new(wind.direction.x, 0.0, wind.direction.z).normalize_or(Vec3::X);
        Self {
            dir_strength: direction.extend(wind.strength.max(0.0)).to_array(),
            gust_top: [wind.gust.clamp(0.0, 1.0), top, phase[0], phase[1]],
            origin: origin
                .extend(std::f32::consts::TAU / WIND_WAVELENGTH)
                .to_array(),
            field: crate::wind::field_gpu(Some(wind), tick, tick, tick_rate, origin, grid),
        }
    }
}

/// Computed once over the cooked vertices, in model units. An LOD can use its
/// own bounds; a whole model takes the maxima over all levels. These are
/// deformation bounds only: neither the rest-pose sphere nor LOD error changes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FoliageBounds {
    pub root_radius: f32,
    pub branch_radius: f32,
    pub leaf_radius: f32,
    pub flexibility: f32,
    pub flutter: f32,
}

impl FoliageBounds {
    pub fn include(&mut self, position: Vec3, vertex: FoliageVertex) {
        self.root_radius = self.root_radius.max(position.length());
        let [x, y, z, _] = vertex.branch;
        let branch = Vec3::new(x, y, z);
        let [x, y, z, _] = vertex.leaf;
        let leaf = Vec3::new(x, y, z);
        self.branch_radius = self.branch_radius.max(position.distance(branch));
        self.leaf_radius = self.leaf_radius.max(position.distance(leaf));
        self.flexibility = self.flexibility.max(1.0 - vertex.branch[3]);
        self.flutter = self.flutter.max(vertex.leaf[3]);
    }

    /// Maximum displacement for any phase, including a leaf carried by a
    /// bending branch and trunk. For a rotation through at most θ, the chord
    /// is 2r sin(θ/2). Outer rotations preserve the length of an inner
    /// displacement, so the sum of the three rest-pose chords is conservative.
    /// `lean` is the most the trunk leans ([`crate::wind_field::lean_cap`]).
    pub fn deform_margin(&self, lean: f32) -> f32 {
        let lean = lean.max(0.0);
        let chord = |radius: f32, amplitude: f32| 2.0 * radius * (0.5 * amplitude.atan()).sin();
        let leaf = chord(self.leaf_radius, lean * 0.35 * self.flutter);
        let branch = chord(self.branch_radius, lean * self.flexibility);
        leaf + branch + chord(self.root_radius, lean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_vec4_packing_matches_the_shader() {
        let module = naga::front::wgsl::parse_str(&format!("{WIND_WGSL}{WGSL}")).unwrap();
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).unwrap();
        let (handle, ty) = module
            .types
            .iter()
            .find(|(_, t)| t.name.as_deref() == Some("FoliageWind"))
            .unwrap();
        let naga::TypeInner::Struct { members, .. } = &ty.inner else {
            panic!("FoliageWind must be a struct")
        };
        for (name, offset) in [
            (
                "dir_strength",
                std::mem::offset_of!(FoliageWind, dir_strength),
            ),
            ("gust_top", std::mem::offset_of!(FoliageWind, gust_top)),
            ("origin", std::mem::offset_of!(FoliageWind, origin)),
            ("field", std::mem::offset_of!(FoliageWind, field)),
        ] {
            let member = members
                .iter()
                .find(|m| m.name.as_deref() == Some(name))
                .unwrap();
            assert_eq!(member.offset as usize, offset, "{name}");
        }
        assert_eq!(
            layouter[handle].size as usize,
            std::mem::size_of::<FoliageWind>()
        );
        assert_eq!(std::mem::size_of::<FoliageWind>(), 9 * 16);
    }

    #[test]
    fn margins_cover_leaf_branch_and_trunk_rotations() {
        let mut bounds = FoliageBounds::default();
        bounds.include(
            Vec3::new(2.0, 5.0, -1.0),
            FoliageVertex {
                branch: [0.0, 2.0, 0.0, 0.25],
                leaf: [1.0, 4.5, -1.0, 0.8],
            },
        );
        let wind = Wind {
            strength: 0.7,
            gust: 0.8,
            ..Wind::default()
        };
        let margin = bounds.deform_margin(crate::wind_field::lean_cap(Some(wind), false));
        // Independent geometric sampling: varying axes and signed angles,
        // at the maximum gust, can move the inner rotations outwards before
        // the outer one rotates them. No time or frequency formula is reused.
        let original = Vec3::new(2.0, 5.0, -1.0);
        let branch = Vec3::new(0.0, 2.0, 0.0);
        let leaf = Vec3::new(1.0, 4.5, -1.0);
        let lean: f32 = 1.26;
        let angles = [
            (lean * 0.35 * 0.8).atan(),
            (lean * 0.75).atan(),
            lean.atan(),
        ];
        for i in 0..64 {
            let axis = Vec3::new(1.0, (i as f32 * 0.7).sin(), (i as f32 * 0.3).cos()).normalize();
            for sign in [-1.0, 1.0] {
                let p =
                    leaf + glam::Quat::from_axis_angle(axis, sign * angles[0]) * (original - leaf);
                let p =
                    branch + glam::Quat::from_axis_angle(Vec3::Z, -sign * angles[1]) * (p - branch);
                let p = glam::Quat::from_axis_angle(Vec3::X, sign * angles[2]) * p;
                assert!(p.distance(original) <= margin);
            }
        }
        assert_eq!(bounds.deform_margin(0.0), 0.0);
        let rigid = FoliageBounds {
            flexibility: 0.0,
            flutter: 0.0,
            ..bounds
        };
        assert!(rigid.deform_margin(crate::wind_field::lean_cap(Some(wind), false)) < margin);
        // The field's turbulence and motors widen it.
        let gusty = Wind {
            turbulence: 1.0,
            ..wind
        };
        assert!(bounds.deform_margin(crate::wind_field::lean_cap(Some(gusty), false)) >= margin);
        assert!(bounds.deform_margin(crate::wind_field::lean_cap(None, true)) > 0.0);
    }
}
