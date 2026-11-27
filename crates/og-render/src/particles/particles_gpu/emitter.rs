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

fn color_curve_gpu(c: &crate::particles::ColorCurve) -> ([[f32; 4]; 2], [[f32; 4]; 8]) {
    let (mut ts, mut vs) = ([[0.0; 4]; 2], [[0.0; 4]; 8]);
    for (i, (t, col)) in c.keys.iter().take(MAX_CURVE_KEYS).enumerate() {
        ts[i / 4][i % 4] = *t;
        vs[i] = *col;
    }
    (ts, vs)
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

/// `struct SimDispatch` in particles_sim.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SimDispatchGpu {
    pub tile_offset: u32,
    pub tile_count: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

/// `struct CompactTile` in particles_compact.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct CompactTileGpu {
    pub src_base: u32,
    pub dst_base: u32,
    pub trail_src_base: u32,
    pub trail_dst_base: u32,
    pub slot_offset: u32,
    pub count: u32,
    pub compact_idx: u32,
    pub compact_idx2: u32,
}

/// `struct EmitterDraw` in particle_draw_common.wgsl (dynamic uniform buffer).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct EmitterDrawGpu {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub size_t: [[f32; 4]; 2],
    pub size_v: [[f32; 4]; 2],
    pub height_t: [[f32; 4]; 2],
    pub height_v: [[f32; 4]; 2],
    pub spin_t: [[f32; 4]; 2],
    pub spin_v: [[f32; 4]; 2],
    pub color_t: [[f32; 4]; 2],
    pub color_v: [[f32; 4]; 8],
    pub color2_t: [[f32; 4]; 2],
    pub color2_v: [[f32; 4]; 8],
    pub sheet: [f32; 4],
    pub sheet2: [f32; 4],
    pub render: [f32; 4],
    pub counts: [f32; 4],
    pub counts2: [f32; 4],
    pub look: [f32; 4],
    pub axis: [f32; 4],
    pub trail: [f32; 4],
    pub crop: [f32; 4],
    pub info: [u32; 4],
}

/// `struct EmitterSim` in particles_sim.wgsl (densely packed compute storage).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct EmitterSimGpu {
    pub transform: [[f32; 4]; 4],
    pub prev_transform: [[f32; 4]; 4],
    pub shape: [f32; 4],
    pub shape2: [f32; 4],
    pub ranges0: [f32; 4],
    pub ranges1: [f32; 4],
    pub ranges2: [f32; 4],
    pub forces: [f32; 4],
    pub noise: [f32; 4],
    pub collision: [f32; 4],
    pub noise_origin: [[f32; 4]; 2],
    pub speed_t: [[f32; 4]; 2],
    pub speed_v: [[f32; 4]; 2],
    pub attract: [[f32; 4]; 4],
    pub attract_r: [f32; 4],
    pub field: [f32; 4],
    pub field_origin: [[f32; 4]; 2],
    pub motion: [f32; 4],
    pub air: [f32; 4],
    pub counts: [f32; 4],
    pub trail: [f32; 4],
    pub spawn: [u32; 4],
    pub info: [u32; 4],
    pub pool: [u32; 4],
}

/// Builds the lean draw-only uniform for a system or sub-emitter child.
#[allow(clippy::too_many_arguments)]
pub(super) fn emitter_draw_gpu(
    e: &ParticleEmitter,
    transform: Mat4,
    tint: Vec3,
    hue: f32,
    seed: u32,
    textured: bool,
    soft: f32,
    crop: [f32; 4],
    world_override: bool,
) -> EmitterDrawGpu {
    let (size_t, size_v, size_n) = curve_gpu(Some(&e.size_over_life));
    let (height_t, height_v, height_n) = curve_gpu(e.height_over_life.as_ref());
    let (spin_t, spin_v, spin_n) = curve_gpu(e.spin_over_life.as_ref());
    let (color_t, color_v) = color_curve_gpu(&e.color_over_life);
    let (color2_t, color2_v) = e
        .random_color_over_life
        .as_ref()
        .map_or(([[0.0; 4]; 2], [[0.0; 4]; 8]), color_curve_gpu);
    let color2_n = e
        .random_color_over_life
        .as_ref()
        .map_or(0, |c| c.keys.len().min(MAX_CURVE_KEYS)) as f32;
    let (mode, param) = match e.sheet.mode {
        SheetMode::Off => (0.0, 0.0),
        SheetMode::OverLife { cycles } => (1.0, cycles),
        SheetMode::Fps(fps) => (2.0, fps),
        SheetMode::Random => (3.0, 0.0),
    };
    let (align, stretch, axis) = match e.align {
        ParticleAlign::Camera => (0.0, 0.0, Vec3::Y),
        ParticleAlign::Velocity { stretch } => (1.0, stretch, Vec3::Y),
        ParticleAlign::Upright { axis } => (2.0, 0.0, axis.normalize_or(Vec3::Y)),
        ParticleAlign::Facing { normal } => (3.0, 0.0, normal.normalize_or(Vec3::Y)),
    };
    let local = !world_override && e.space == SimulationSpace::Local;
    EmitterDrawGpu {
        transform: transform.to_cols_array_2d(),
        color: [
            e.color[0] * tint.x,
            e.color[1] * tint.y,
            e.color[2] * tint.z,
            e.color[3],
        ],
        size_t,
        size_v,
        height_t,
        height_v,
        spin_t,
        spin_v,
        color_t,
        color_v,
        color2_t,
        color2_v,
        sheet: [
            e.sheet.columns.max(1) as f32,
            e.sheet.rows.max(1) as f32,
            sheet_frames(&e.sheet) as f32,
            mode,
        ],
        sheet2: [
            param,
            textured as u32 as f32,
            e.sheet.blend as u32 as f32,
            local as u32 as f32,
        ],
        render: [
            align,
            stretch,
            soft,
            match e.blend() {
                ParticleBlend::Additive => 0.0,
                ParticleBlend::Alpha => 1.0,
                ParticleBlend::Subtractive => 2.0,
                ParticleBlend::Multiply => 3.0,
            },
        ],
        counts: [
            size_n,
            e.color_over_life.keys.len().min(MAX_CURVE_KEYS) as f32,
            height_n,
            color2_n,
        ],
        counts2: [spin_n, hue, 0.0, e.translucency],
        look: [
            e.lit as u32 as f32,
            e.distortion,
            matches!(e.render, ParticleRender::Mesh { .. }) as u32 as f32,
            e.roughness,
        ],
        axis: axis.extend(0.0).to_array(),
        trail: e
            .trail
            .map_or([0.0, 0.0, 0.0, -1.0], |t| [t.lifetime, t.width, 0.0, -1.0]),
        crop,
        info: [e.max_particles, seed, 0, 0],
    }
}

/// Builds the compute-only simulation entry for one tick step of a system (or
/// a sub-emitter child in world space).
#[allow(clippy::too_many_arguments)]
pub(super) fn emitter_sim_gpu(
    e: &ParticleEmitter,
    transform: Mat4,
    prev_transform: Option<Mat4>,
    step: Option<&TickStep>,
    dt: f32,
    time: f32,
    seed: u32,
    emission_scale: f32,
    points_count: u32,
    points_base: u32,
    grid_base: u32,
    particle_base: u32,
    trail_base: u32,
    collide_depth: bool,
    world_child: bool,
) -> EmitterSimGpu {
    let shape = if world_child {
        match e.shape {
            EmitterShape::Mesh { .. } | EmitterShape::Spline { .. } => EmitterShape::Point,
            s => s,
        }
    } else {
        e.shape
    };
    let (kind, a, b) = match shape {
        EmitterShape::Point => (0.0, 0.0, 0.0),
        EmitterShape::Sphere { radius, surface } => (1.0, radius, surface as u32 as f32),
        EmitterShape::Cone {
            angle_degrees,
            radius,
        } => (2.0, angle_degrees.to_radians(), radius),
        EmitterShape::Box { .. } => (3.0, 0.0, 0.0),
        EmitterShape::Mesh { .. } => (4.0, points_count as f32, 0.0),
        EmitterShape::Spline { .. } => (5.0, points_count as f32, 0.0),
    };
    let half = match shape {
        EmitterShape::Box { half_extents } => half_extents,
        _ => Vec3::ZERO,
    };
    let (bounce, friction) = e.collision.map_or((0.0, 0.0), |c| (c.bounce, c.friction));
    let max = e.max_particles;
    let (first, count, bursts) = step.map_or((0, 0, 0), |s| (s.first, s.count, s.bursts));
    let (speed_t, speed_v, speed_n) = curve_gpu(e.speed_over_life.as_ref());
    let local = !world_child && e.space == SimulationSpace::Local;
    let (mut attract, mut attract_r) = ([[0.0; 4]; 4], [0.0; 4]);
    let placed = (!world_child).then(|| crate::particles::placed_attractors(e, transform));
    let pulls: &[crate::particles::Attractor] = match &placed {
        Some(p) => p,
        None => &e.attractors[..e.attractors.len().min(crate::particles::MAX_ATTRACTORS)],
    };
    for (i, att) in pulls.iter().enumerate() {
        attract[i] = att.position.extend(att.strength).to_array();
        attract_r[i] = att.radius;
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
    let scale = if world_child {
        1.0
    } else {
        crate::particles::transform_scale(transform)
    };
    let force = if world_child {
        1.0
    } else {
        crate::particles::force_scale(e, transform)
    };
    let (prev_m, has_prev) = match prev_transform {
        Some(pm) if !local && !world_child => (pm, 1.0),
        _ => (transform, 0.0),
    };
    let push = if !local { e.force_fields } else { 0.0 };
    EmitterSimGpu {
        transform: transform.to_cols_array_2d(),
        prev_transform: prev_m.to_cols_array_2d(),
        shape: [kind, a, b, has_prev],
        shape2: [half.x, half.y, half.z, e.spread_degrees.to_radians()],
        ranges0: [e.lifetime[0], e.lifetime[1], e.speed[0], e.speed[1]],
        ranges1: [
            e.size[0] * scale,
            e.size[1] * scale,
            e.rotation_degrees[0].to_radians(),
            e.rotation_degrees[1].to_radians(),
        ],
        ranges2: [
            e.angular_velocity_degrees[0].to_radians(),
            e.angular_velocity_degrees[1].to_radians(),
            local as u32 as f32,
            dt,
        ],
        forces: (e.gravity * force).extend(e.drag).to_array(),
        noise: [
            e.noise.strength * force,
            e.noise.frequency,
            e.noise.scroll,
            time,
        ],
        collision: [collide_depth as u32 as f32, bounce, friction, 0.0],
        noise_origin: [[0.0; 4]; 2],
        speed_t,
        speed_v,
        attract,
        attract_r,
        field,
        field_origin: [[0.0; 4]; 2],
        motion: velocity.extend(e.inherit_velocity).to_array(),
        air: [0.0, 0.0, 0.0, push],
        counts: [
            speed_n,
            pulls.len() as f32,
            e.emission * emission_scale,
            e.collision.is_some_and(|c| c.die) as u32 as f32,
        ],
        trail: e
            .trail
            .map_or([0.0, 0.0, 0.0, -1.0], |t| [t.lifetime, t.width, 0.0, -1.0]),
        spawn: [first as u32, count, (first % max as u64) as u32, bursts],
        info: [max, seed, u32::MAX, points_base],
        pool: [particle_base, trail_base, grid_base, 0],
    }
}

/// Packs one GPU sub-emitter descriptor as 4 `vec4<u32>` entries appended to
/// `sim_tiles`.
pub(super) fn pack_sub_desc(
    sub: &crate::particles::SubEmitter,
    child: &ParticleEmitter,
    child_entry_idx: u32,
    counter_idx: u32,
) -> [[u32; 4]; 4] {
    use crate::particles::{ParticleEventKind, SubTrigger};
    let (trigger, count) = match sub.trigger {
        SubTrigger::Event(ParticleEventKind::Birth) => (0u32, sub.count),
        SubTrigger::Event(ParticleEventKind::Death) => (1u32, sub.count),
        SubTrigger::Event(ParticleEventKind::Collision) => (2u32, sub.count),
        SubTrigger::Life => (3u32, child.bursts.len().min(2) as u32),
    };
    let pack_burst = |i: usize| -> [u32; 4] {
        match child.bursts.get(i) {
            Some(b) => [
                b.time.to_bits(),
                (b.count as f32).to_bits(),
                (b.cycles.max(1) as f32).to_bits(),
                b.interval.to_bits(),
            ],
            None => [0; 4],
        }
    };
    [
        [child_entry_idx, counter_idx, trigger, count],
        [
            sub.inherit_velocity.to_bits(),
            child.rate.to_bits(),
            child.duration.to_bits(),
            (child.looping as u32 as f32).to_bits(),
        ],
        pack_burst(0),
        pack_burst(1),
    ]
}
