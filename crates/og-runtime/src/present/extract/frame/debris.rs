//! The debris layer's chips and crumbs, thrown from a seed.

use super::*;

/// How far (m) a fine piece may fly from where it broke off before it is
/// no longer drawn (og_render::DebrisPiece::reach): its culling sphere.
pub(super) const DEBRIS_REACH: f32 = 12.0;

/// A stream of 0..1 numbers from a seed (splitmix64): a debris chip's
/// every number, the same each frame.
pub(super) struct Seeded(pub(super) u64);

impl Seeded {
    pub(super) fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A direction, evenly over the sphere.
    pub(super) fn unit(&mut self) -> Vec3 {
        let z = self.next() * 2.0 - 1.0;
        let a = self.next() * std::f32::consts::TAU;
        let r = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(r * og_core::math::cos(a), z, r * og_core::math::sin(a))
    }
}

/// One rubble chip (og_render::chip) as the debris layer throws it.
pub(super) struct Chip {
    pub(super) id: u64,
    pub(super) tick: u64,
    pub(super) position: Vec3,
    /// Its size along its own axes (m): a chip is about 1 across.
    pub(super) scale: Vec3,
    pub(super) velocity: Vec3,
    pub(super) angular: Vec3,
    pub(super) material: og_render::MaterialHandle,
    pub(super) reach: f32,
}

/// Draws `chip` as an instance the debris layer moves: one of the chip
/// shapes (made at the first) picked and turned by `seed`. False once the
/// layer is full.
pub(super) fn push_chip(
    rs: &mut RenderState,
    chip: Chip,
    turn: Quat,
    seed: &mut Seeded,
    look: &super::fx::ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    if rs.scene.objects.debris.len() + 1 >= og_render::MAX_DEBRIS {
        return false;
    }
    if rs.fx.chip_meshes.is_empty() {
        rs.fx.chip_meshes = (0..og_render::CHIP_VARIANTS)
            .map(|v| {
                let m = og_render::chip(v);
                // Its hull's points: the mesh's corners, each once.
                let mut points: Vec<Vec3> = Vec::new();
                for p in m.vertices.iter().map(|x| Vec3::from(x.position)) {
                    if !points.contains(&p) {
                        points.push(p);
                    }
                }
                (rs.scene.add_mesh(m), points)
            })
            .collect();
    }
    let variant = (seed.next() * og_render::CHIP_VARIANTS as f32) as u32 % og_render::CHIP_VARIANTS;
    let (mesh, points) = rs.fx.chip_meshes[variant as usize].clone();
    let axis = seed.unit();
    let rotation =
        og_core::math::quat_from_axis_angle(axis, seed.next() * std::f32::consts::TAU) * turn;
    let body = Mat4::from_scale_rotation_translation(chip.scale, rotation, chip.position);
    if rs.scene.meshlet_drawable(mesh, chip.material) {
        let at = rs.scene.objects.instances.len();
        rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
        rs.scene.objects.instance_levels[at] = 0;
    }
    let index = rs.scene.objects.instances.len() as u32;
    rs.scene.objects.instances.push(og_render::Instance {
        mesh,
        material: chip.material,
        transform: body,
        material_layer: None,
        fx: look.fx,
        cast_shadows: look.cast,
        receive_shadows: look.receive,
        sway: 0.0,
    });
    keys.push((chip.id, 0));
    rs.scene.objects.debris.push(og_render::DebrisPiece {
        id: chip.id,
        tick: chip.tick,
        position: chip.position,
        rotation,
        scale: chip.scale,
        velocity: chip.velocity,
        angular_velocity: chip.angular,
        points,
        friction: 0.7,
        restitution: 0.2,
        reach: chip.reach,
        instances: vec![(index, Mat4::IDENTITY)],
    });
    true
}

/// A chip's id: its source's, mixed with which of its chips it is.
pub(super) fn chip_id(source: u64, k: u32) -> u64 {
    source ^ (u64::from(k) + 1).wrapping_mul(0xd6e8_feb8_6659_fd93)
}

/// The rubble chips a fine piece throws as it breaks off: four to six
/// lumps (og_render::chip) of its own cut faces' material, one in four of
/// its outer face's, each a third to two thirds of the piece across with
/// most of them small, at least half as thick as wide, flung from its
/// volume and tumbling. Visual only and the same each frame: every
/// number is the piece's id hashed, so a restored world or a renderer
/// made late throws the same chips, and the debris layer catches them up
/// from the tick the piece broke off as it does the piece.
pub(super) fn throw_chips(
    rs: &mut RenderState,
    piece: &og_render::DebrisPiece,
    materials: &[og_render::MaterialHandle],
    look: &super::fx::ModelLook,
    keys: &mut Vec<(u64, u32)>,
) {
    let (Some(&inner), Some(&outer)) = (materials.last(), materials.first()) else {
        return;
    };
    let radius = piece
        .points
        .iter()
        .map(|p| (*p * piece.scale).length())
        .fold(0.0, f32::max);
    let mut r = Seeded(piece.id);
    let count = 4 + (r.next() * 3.0) as u32;
    for k in 0..count {
        let material = if materials.len() > 1 && r.next() < 0.25 {
            outer
        } else {
            inner
        };
        // Most chips small, a few near the piece's size.
        let u = r.next();
        let size = radius * (0.3 + 0.5 * u * u);
        let scale = Vec3::new(
            size,
            size * (0.65 + 0.35 * r.next()),
            size * (0.5 + 0.35 * r.next()),
        );
        let offset = r.unit() * radius * 0.5 * r.next().sqrt();
        let fling = r.unit() * (0.5 + 1.3 * r.next());
        let spin = r.unit() * (4.0 + 10.0 * r.next());
        let chip = Chip {
            id: chip_id(piece.id, k),
            tick: piece.tick,
            position: piece.position + offset,
            scale,
            velocity: piece.velocity + piece.angular_velocity.cross(offset) + fling,
            angular: piece.angular_velocity + spin,
            material,
            reach: piece.reach,
        };
        if !push_chip(rs, chip, piece.rotation, &mut r, look, keys) {
            return;
        }
    }
}

/// The grit a broken joint sheds: three or four small chips falling out of
/// the crack, one of the mortar where the prop has one
/// (`fracture.mortar`), the rest of its cut faces' clay (`inner`), from
/// one centimetre to five and a half across, knocked out along the joint's normal
/// either way and moving with the blow that broke it. Seeded by the
/// joint as [`throw_chips`] is by its piece.
pub(super) fn throw_crumb(
    rs: &mut RenderState,
    c: &crate::bind::destruction::DrawnCrumb,
    inner: og_render::MaterialHandle,
    mortar: Option<og_render::MaterialHandle>,
    look: &super::fx::ModelLook,
    keys: &mut Vec<(u64, u32)>,
) {
    let mut r = Seeded(c.id);
    let count = 3 + (r.next() * 2.0) as u32;
    let n = c.normal.normalize_or(Vec3::Y);
    for k in 0..count {
        let (material, size) = match mortar {
            Some(m) if k == 0 => (m, 0.012 + 0.016 * r.next()),
            _ => (inner, 0.02 + 0.035 * r.next() * r.next()),
        };
        let scale = Vec3::new(
            size,
            size * (0.6 + 0.4 * r.next()),
            size * (0.5 + 0.4 * r.next()),
        );
        let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
        // Anywhere on the joint's face, most near its rim (where the crack
        // shows): its centre is inside the prop, and a chip there would
        // ride hidden in the bricks.
        let along = r.unit();
        let flat = (along - n * along.dot(n)).normalize_or(n.any_orthonormal_vector());
        let offset = flat * c.half * (1.0 - 0.6 * r.next() * r.next()) + n * side * 0.01;
        let chip = Chip {
            id: chip_id(c.id, k),
            tick: c.tick,
            position: c.point + offset,
            scale,
            velocity: c.velocity
                + n * side * (0.3 + 0.9 * r.next())
                + flat * (0.4 + 0.8 * r.next()),
            angular: r.unit() * (6.0 + 14.0 * r.next()),
            material,
            reach: DEBRIS_REACH,
        };
        if !push_chip(rs, chip, Quat::IDENTITY, &mut r, look, keys) {
            return;
        }
    }
}
