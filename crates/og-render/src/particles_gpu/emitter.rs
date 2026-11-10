use super::*;

/// A curve's keys as the shader's two times and two values vec4s, and
/// its key count (0 for none).
fn curve_gpu(c: Option<&crate::particles::Curve>) -> ([[f32; 4]; 2], [[f32; 4]; 2], f32) {
    let (mut t, mut v) = ([[0.0; 4]; 2], [[0.0; 4]; 2]);
    let Some(c) = c else { return (t, v, 0.0) };
    for (i, (kt, kv)) in c.keys.iter().take(MAX_CURVE_KEYS).enumerate() {
        t[i / 4][i % 4] = *kt;
        v[i / 4][i % 4] = *kv;
    }
    (t, v, c.keys.len().min(MAX_CURVE_KEYS) as f32)
}

/// `struct ParticleView` in particle_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ParticleViewGpu {
    pub view_proj: [[f32; 4]; 4],
    pub cam_pos: [f32; 4],
    pub cam_forward: [f32; 4],
    pub cam_right: [f32; 4],
    pub cam_up: [f32; 4],
    pub screen: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emitter_gpu(
    e: &ParticleEmitter,
    transform: Mat4,
    step: Option<&TickStep>,
    dt: f32,
    seed: u32,
    points: u32,
    textured: bool,
    collide: bool,
    soft: f32,
) -> EmitterGpu {
    let (kind, a, b) = match e.shape {
        EmitterShape::Point => (0.0, 0.0, 0.0),
        EmitterShape::Sphere { radius, surface } => (1.0, radius, surface as u32 as f32),
        EmitterShape::Cone {
            angle_degrees,
            radius,
        } => (2.0, angle_degrees.to_radians(), radius),
        EmitterShape::Box { .. } => (3.0, 0.0, 0.0),
        EmitterShape::Mesh { .. } => (4.0, points as f32, 0.0),
        EmitterShape::Spline { .. } => (5.0, points as f32, 0.0),
    };
    let half = match e.shape {
        EmitterShape::Box { half_extents } => half_extents,
        _ => Vec3::ZERO,
    };
    let mut size_t = [[0.0; 4]; 2];
    let mut size_v = [[0.0; 4]; 2];
    for (i, (t, v)) in e
        .size_over_life
        .keys
        .iter()
        .take(MAX_CURVE_KEYS)
        .enumerate()
    {
        size_t[i / 4][i % 4] = *t;
        size_v[i / 4][i % 4] = *v;
    }
    let mut color_t = [[0.0; 4]; 2];
    let mut color_v = [[0.0; 4]; 8];
    for (i, (t, c)) in e
        .color_over_life
        .keys
        .iter()
        .take(MAX_CURVE_KEYS)
        .enumerate()
    {
        color_t[i / 4][i % 4] = *t;
        color_v[i] = *c;
    }
    let (mode, param) = match e.sheet.mode {
        SheetMode::Off => (0.0, 0.0),
        SheetMode::OverLife { cycles } => (1.0, cycles),
        SheetMode::Fps(fps) => (2.0, fps),
        SheetMode::Random => (3.0, 0.0),
    };
    let (align, stretch) = match e.align {
        ParticleAlign::Camera => (0.0, 0.0),
        ParticleAlign::Velocity { stretch } => (1.0, stretch),
    };
    let (bounce, friction) = e.collision.map_or((0.0, 0.0), |c| (c.bounce, c.friction));
    let max = e.max_particles;
    let (first, count, bursts, time) =
        step.map_or((0, 0, 0, 0.0), |s| (s.first, s.count, s.bursts, s.time));
    let (speed_t, speed_v, speed_n) = curve_gpu(e.speed_over_life.as_ref());
    let (spin_t, spin_v, spin_n) = curve_gpu(e.spin_over_life.as_ref());
    let pulls = crate::particles::placed_attractors(e, transform);
    let (mut attract, mut attract_r) = ([[0.0; 4]; 4], [0.0; 4]);
    for (i, a) in pulls.iter().enumerate() {
        attract[i] = a.position.extend(a.strength).to_array();
        attract_r[i] = a.radius;
    }
    let field = match &e.field {
        None => [0.0; 4],
        Some(crate::particles::VectorField::Curl {
            strength,
            frequency,
            scroll,
        }) => [1.0, *strength, *frequency, *scroll],
        Some(crate::particles::VectorField::Grid {
            strength,
            size,
            cells,
            ..
        }) => [2.0, *strength, *size, *cells as f32],
    };
    let velocity = step.map_or(Vec3::ZERO, |s| s.velocity);
    EmitterGpu {
        transform: transform.to_cols_array_2d(),
        shape: [kind, a, b, 0.0],
        shape2: [half.x, half.y, half.z, e.spread_degrees.to_radians()],
        ranges0: [e.lifetime[0], e.lifetime[1], e.speed[0], e.speed[1]],
        ranges1: [
            e.size[0],
            e.size[1],
            e.rotation_degrees[0].to_radians(),
            e.rotation_degrees[1].to_radians(),
        ],
        ranges2: [
            e.angular_velocity_degrees[0].to_radians(),
            e.angular_velocity_degrees[1].to_radians(),
            if e.space == SimulationSpace::Local {
                1.0
            } else {
                0.0
            },
            dt,
        ],
        forces: e.gravity.extend(e.drag).to_array(),
        noise: [e.noise.strength, e.noise.frequency, e.noise.scroll, time],
        collision: [collide as u32 as f32, bounce, friction, 0.0],
        color: e.color,
        size_t,
        size_v,
        color_t,
        color_v,
        sheet: [
            e.sheet.columns.max(1) as f32,
            e.sheet.rows.max(1) as f32,
            sheet_frames(&e.sheet) as f32,
            mode,
        ],
        sheet2: [param, textured as u32 as f32, 0.0, 0.0],
        render: [
            align,
            stretch,
            soft,
            match e.blend {
                ParticleBlend::Additive => 0.0,
                ParticleBlend::Alpha => 1.0,
            },
        ],
        counts: [
            e.size_over_life.keys.len().min(MAX_CURVE_KEYS) as f32,
            e.color_over_life.keys.len().min(MAX_CURVE_KEYS) as f32,
            0.0,
            0.0,
        ],
        spawn: [first as u32, count, (first % max as u64) as u32, bursts],
        info: [max, seed, 0, 0],
        noise_origin: [[0.0; 4]; 2],
        look: [
            e.lit as u32 as f32,
            e.distortion,
            matches!(e.render, ParticleRender::Mesh { .. }) as u32 as f32,
            e.roughness,
        ],
        speed_t,
        speed_v,
        spin_t,
        spin_v,
        attract,
        attract_r,
        field,
        field_origin: [[0.0; 4]; 2],
        motion: velocity.extend(e.inherit_velocity).to_array(),
        counts2: [speed_n, spin_n, pulls.len() as f32, points as f32 * 2.0],
    }
}
