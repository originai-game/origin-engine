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
