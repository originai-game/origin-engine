use glam::Vec3;

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

/// How a leaf material follows the year (`og.Season`): a species' own
/// values, which a plant's `leaves` block, a model text's `material` block
/// and glTF `extras.ogFoliage` set (`season`, `autumnColor`,
/// `autumnColor2`, `seasonShift`) and the cooked material carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafSeason {
    /// How far the leaves follow the year: 0 evergreen, 1 deciduous.
    pub season: f32,
    /// Each leaf turns to a colour between these two (linear RGB) before
    /// it browns and falls.
    pub autumn_color: [f32; 3],
    pub autumn_color2: [f32; 3],
    /// Days the species runs early (negative) or late.
    pub shift: f32,
}

impl LeafSeason {
    /// An evergreen's, with the default autumn colours: orange and deep red.
    pub const EVERGREEN: Self = Self {
        season: 0.0,
        autumn_color: [0.42, 0.12, 0.015],
        autumn_color2: [0.25, 0.025, 0.015],
        shift: 0.0,
    };
    /// The most days a species runs early or late.
    pub const MAX_SHIFT: f32 = 120.0;
}

impl Default for LeafSeason {
    fn default() -> Self {
        Self::EVERGREEN
    }
}

/// The year a frame draws the seasonal leaves in (og.Season): its
/// climate's leaf year and the date as the place lives it
/// (`og_env::season::local`). Each leaf of a species with
/// [`LeafSeason::season`] above 0 then grows, turns and falls on its own
/// day (foliage.wgsl `season_leaf`); without a calendar every leaf
/// is drawn as modelled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafCalendar {
    pub year: og_env::season::LeafYear,
    pub day: f32,
}

impl LeafCalendar {
    /// foliage.wgsl's `LeafYear`: (the bud's day, the days growing,
    /// opening and turning), (the days after the turn starts that the
    /// first and the last leaf fall, the day, the kind: 0 every leaf as
    /// modelled, 1 a seasonal year, 2 bare all year).
    pub fn gpu(calendar: Option<LeafCalendar>) -> [[f32; 4]; 2] {
        use og_env::season::LeafYear;
        match calendar {
            None
            | Some(LeafCalendar {
                year: LeafYear::Evergreen,
                ..
            }) => [[0.0; 4]; 2],
            Some(LeafCalendar {
                year: LeafYear::Bare,
                day,
            }) => [[0.0; 4], [0.0, 0.0, day, 2.0]],
            Some(LeafCalendar {
                year:
                    LeafYear::Seasonal {
                        bud,
                        growing,
                        opening,
                        turning,
                        fall,
                    },
                day,
            }) => [
                [bud, growing, opening, turning],
                [fall[0], fall[1], day, 1.0],
            ],
        }
    }
}

/// A leaf's lot in its tree's year, from the point it hangs from
/// ([`FoliageVertex::leaf`], `_OG_ANCHOR`, in model space): `order`, when
/// it turns and falls among its tree's leaves (0 first), and `pick`, where
/// its autumn colour lies between [`LeafSeason::autumn_color`] and
/// `autumn_color2`; each even over 0 to 1 across a tree's leaves and
/// independent of the other. A hash of the anchor to the millimetre, in
/// `hash.wgsl`'s operations (its `lattice3`, then `pcg`), so every cook
/// and a shader's twin agree and the leaves of one anchor turn together. A
/// tree's own offset (`foliage_seed`) is added where it is drawn.
pub fn leaf_order_pick(anchor: Vec3) -> (f32, f32) {
    use og_core::hash::{pcg, unit24};
    let q = (anchor * 1000.0).round();
    let h = (q.x as i32 as u32).wrapping_mul(73_856_093)
        ^ (q.y as i32 as u32).wrapping_mul(19_349_663)
        ^ (q.z as i32 as u32).wrapping_mul(83_492_791);
    let order = pcg(h);
    (unit24(order), unit24(pcg(order)))
}
