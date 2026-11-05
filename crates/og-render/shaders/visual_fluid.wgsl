// Visual fluids (og.Fluid3D's SIMULATION_VISUAL): MLS-MPM on the GPU for
// looks only; nothing reads this back.
//
// Ported from matsuoka-601/WebGPU-Ocean (MIT, Copyright (c) 2025
// matsuoka-601; revision 3bd932778650, `mls-mpm/*.wgsl`): the
// moving-least-squares material point method of Hu et al. (SIGGRAPH 2018)
// for a weakly compressible fluid, as nialltl's MPM guide lays it out (the
// volume recomputed from the grid every step), with fixed-point atomicAdd
// scatter. See third_party/webgpu-ocean/ for the licence and the changes.
//
// Units are upstream's: positions in grid cells, time in steps of 0.2
// (visual_fluid.rs maps metres and seconds onto them), particle mass 1.
// The grid lies in the world: cell (0, 0, 0) at ORIGIN, H metres a cell.

// Shaders/scene_colliders.wgsl comes before this file and reads `colliders`.
@group(0) @binding(0) var<storage, read> head: array<u32>;
@group(0) @binding(1) var<storage, read> colliders: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(3) var<storage, read_write> cells: array<atomic<i32>>;
@group(0) @binding(4) var<storage, read_write> cursor: array<u32>;
@group(0) @binding(5) var<storage, read_write> instances: array<vec4<f32>>;

struct Particle {
    // xyz in cells; w 1 alive, 0 not.
    position: vec4<f32>,
    // xyz in cells a time unit.
    v: vec4<f32>,
    C: mat3x3<f32>,
}

// Header words (visual_fluid.rs `h`).
const N: u32 = 0u;
const COUNT: u32 = 1u;
const RESET: u32 = 2u;
const SEED: u32 = 3u;
const DIMS: u32 = 4u;
const H: u32 = 7u;
const ORIGIN: u32 = 8u;
const GRAVITY: u32 = 11u;
const STIFFNESS: u32 = 14u;
const VISCOSITY: u32 = 15u;
const REST_DENSITY: u32 = 16u;
const FIXED: u32 = 17u;
const WALL_MIN: u32 = 18u;
const WALL_MAX: u32 = 21u;
const SPAWN_AT: u32 = 24u;
const SPAWN_STEP: u32 = 27u;
const LATTICE: u32 = 30u;
const EMIT_AT: u32 = 32u;
const EMIT_RADIUS: u32 = 35u;
const EMIT_V: u32 = 36u;
const EMITTING: u32 = 39u;
const SINK_MIN: u32 = 40u;
const SINK_MAX: u32 = 43u;
const SINKING: u32 = 46u;
const COLLIDER_LIST: u32 = 47u;
const RADIUS: u32 = 48u;
const DRAW_RADIUS: u32 = 49u;
const COLOR: u32 = 50u;
const DENSITY_K: u32 = 53u;
const OUT: u32 = 54u;
// The alive counts: before the frame, then after each of its ticks.
const ALIVE: u32 = 56u;

const DT: f32 = 0.2;
// The fastest a particle moves, cells a time unit: under a cell a step,
// and its momentum within the fixed-point range.
const MAX_SPEED: f32 = 3.0;

fn word(i: u32) -> f32 { return bitcast<f32>(head[i]); }
fn vec(i: u32) -> vec3<f32> { return vec3<f32>(word(i), word(i + 1u), word(i + 2u)); }
fn dims() -> vec3<i32> { return vec3<i32>(i32(head[DIMS]), i32(head[DIMS + 1u]), i32(head[DIMS + 2u])); }
fn cell_count() -> u32 { return head[DIMS] * head[DIMS + 1u] * head[DIMS + 2u]; }

fn encode(f: f32) -> i32 { return i32(f * word(FIXED)); }
fn decode(i: i32) -> f32 { return f32(i) / word(FIXED); }

fn cell_index(c: vec3<f32>) -> u32 {
    let d = dims();
    return u32((i32(c.x) * d.y + i32(c.y)) * d.z + i32(c.z)) * 4u;
}

fn to_cells(world: vec3<f32>) -> vec3<f32> { return (world - vec(ORIGIN)) / word(H); }
fn to_world(c: vec3<f32>) -> vec3<f32> { return vec(ORIGIN) + c * word(H); }

fn hash(a: u32, b: u32, c: u32) -> f32 {
    var h = a * 0x27d4eb2du + b * 0x9e3779b1u + c * 0x85ebca77u + 0x3c6ef372u;
    h ^= h >> 15u;
    h *= 0x2c1b3c6du;
    h ^= h >> 12u;
    h *= 0x297a2d39u;
    h ^= h >> 15u;
    return f32(h >> 8u) / f32(1u << 23u);
}

fn tick_index() -> u32 { return cursor[0]; }
fn alive_before() -> u32 { return head[ALIVE + tick_index()]; }
fn alive_now() -> u32 { return head[ALIVE + tick_index() + 1u]; }

fn weights(diff: vec3<f32>) -> array<vec3<f32>, 3> {
    return array<vec3<f32>, 3>(
        0.5 * (0.5 - diff) * (0.5 - diff),
        0.75 - diff * diff,
        0.5 * (0.5 + diff) * (0.5 + diff),
    );
}

// A particle (re)born at the emitter, the n-th the emitter has made.
fn at_emitter(i: u32, n: u32) {
    let r = word(EMIT_RADIUS);
    let s = head[SEED] ^ 0x5bd1e995u;
    let k = vec3<f32>(hash(s, n, 0u), hash(s, n, 1u), hash(s, n, 2u)) - 0.5;
    particles[i].position = vec4<f32>(to_cells(vec(EMIT_AT) + k * 2.0 * r), 1.0);
    particles[i].v = vec4<f32>(vec(EMIT_V), 0.0);
    particles[i].C = mat3x3<f32>();
}

// Wakes the particles that come alive this tick (all of them on the
// frame's first after a reset): the first COUNT on a jittered lattice
// filling the spawn box from the bottom, the rest at the emitter.
@compute @workgroup_size(64)
fn spawn(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= head[N] { return; }
    var first = alive_before();
    if head[RESET] != 0u && tick_index() == 0u {
        first = 0u;
        particles[i].position.w = 0.0;
    }
    if i < first || i >= alive_now() { return; }
    if i < head[COUNT] {
        let cols = head[LATTICE];
        let deep = head[LATTICE + 1u];
        let g = vec3<f32>(f32(i % cols), f32(i / (cols * deep)), f32((i / cols) % deep));
        let j = vec3<f32>(hash(head[SEED], i, 0u), hash(head[SEED], i, 1u), hash(head[SEED], i, 2u)) - 0.5;
        let step = vec(SPAWN_STEP);
        particles[i].position = vec4<f32>(to_cells(vec(SPAWN_AT) + (g + j * 0.3) * step), 1.0);
        particles[i].v = vec4<f32>(0.0);
        particles[i].C = mat3x3<f32>();
    } else {
        at_emitter(i, i - head[COUNT]);
    }
}

@compute @workgroup_size(64)
fn clear_grid(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= cell_count() { return; }
    for (var k = 0u; k < 4u; k++) { atomicStore(&cells[id.x * 4u + k], 0); }
}

// Particle to grid, first half: mass and momentum (with the affine C).
@compute @workgroup_size(64)
fn p2g_1(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= head[N] { return; }
    let p = particles[id.x];
    if p.position.w == 0.0 { return; }
    let cell_idx = floor(p.position.xyz);
    let w = weights(p.position.xyz - (cell_idx + 0.5));
    for (var gx = 0; gx < 3; gx++) {
        for (var gy = 0; gy < 3; gy++) {
            for (var gz = 0; gz < 3; gz++) {
                let weight = w[gx].x * w[gy].y * w[gz].z;
                let cell_x = cell_idx + vec3<f32>(f32(gx), f32(gy), f32(gz)) - 1.0;
                let q = p.C * ((cell_x + 0.5) - p.position.xyz);
                let momentum = weight * (p.v.xyz + q);
                let c = cell_index(cell_x);
                atomicAdd(&cells[c], encode(momentum.x));
                atomicAdd(&cells[c + 1u], encode(momentum.y));
                atomicAdd(&cells[c + 2u], encode(momentum.z));
                atomicAdd(&cells[c + 3u], encode(weight));
            }
        }
    }
}

// Particle to grid, second half: each particle's density from the grid's
// mass, its pressure (Tait, gamma 5, no tension) and viscous stress, as
// momentum.
@compute @workgroup_size(64)
fn p2g_2(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= head[N] { return; }
    let p = particles[id.x];
    if p.position.w == 0.0 { return; }
    let cell_idx = floor(p.position.xyz);
    let w = weights(p.position.xyz - (cell_idx + 0.5));
    var density = 0.0;
    for (var gx = 0; gx < 3; gx++) {
        for (var gy = 0; gy < 3; gy++) {
            for (var gz = 0; gz < 3; gz++) {
                let weight = w[gx].x * w[gy].y * w[gz].z;
                let cell_x = cell_idx + vec3<f32>(f32(gx), f32(gy), f32(gz)) - 1.0;
                density += decode(atomicLoad(&cells[cell_index(cell_x) + 3u])) * weight;
            }
        }
    }
    let volume = 1.0 / max(density, 1e-4);
    let pressure = max(0.0, word(STIFFNESS) * (pow(density / word(REST_DENSITY), 5.0) - 1.0));
    var stress = mat3x3<f32>(-pressure, 0.0, 0.0, 0.0, -pressure, 0.0, 0.0, 0.0, -pressure);
    stress += word(VISCOSITY) * (p.C + transpose(p.C));
    let term = -volume * 4.0 * stress * DT;
    for (var gx = 0; gx < 3; gx++) {
        for (var gy = 0; gy < 3; gy++) {
            for (var gz = 0; gz < 3; gz++) {
                let weight = w[gx].x * w[gy].y * w[gz].z;
                let cell_x = cell_idx + vec3<f32>(f32(gx), f32(gy), f32(gz)) - 1.0;
                let momentum = term * weight * ((cell_x + 0.5) - p.position.xyz);
                let c = cell_index(cell_x);
                atomicAdd(&cells[c], encode(momentum.x));
                atomicAdd(&cells[c + 1u], encode(momentum.y));
                atomicAdd(&cells[c + 2u], encode(momentum.z));
            }
        }
    }
}

// The grid's velocities from its momentum, gravity, and the container's
// walls (no flow through the outer two cells).
@compute @workgroup_size(64)
fn update_grid(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= cell_count() { return; }
    let c = id.x * 4u;
    let mass = atomicLoad(&cells[c + 3u]);
    if mass <= 0 { return; }
    var v = vec3<f32>(
        decode(atomicLoad(&cells[c])),
        decode(atomicLoad(&cells[c + 1u])),
        decode(atomicLoad(&cells[c + 2u])),
    ) / decode(mass);
    v += vec(GRAVITY) * DT;
    let d = dims();
    let at = vec3<i32>(i32(id.x) / (d.z * d.y), (i32(id.x) / d.z) % d.y, i32(id.x) % d.z);
    let lo = vec3<i32>(vec(WALL_MIN)) - 1;
    let hi = vec3<i32>(vec(WALL_MAX)) + 1;
    for (var k = 0; k < 3; k++) {
        if at[k] < lo[k] && v[k] < 0.0 { v[k] = 0.0; }
        if at[k] > hi[k] && v[k] > 0.0 { v[k] = 0.0; }
    }
    atomicStore(&cells[c], encode(v.x));
    atomicStore(&cells[c + 1u], encode(v.y));
    atomicStore(&cells[c + 2u], encode(v.z));
}

// Grid to particle: the velocity and its affine part (APIC), the move,
// then the scene's colliders, the container's soft walls, and the sink.
@compute @workgroup_size(64)
fn g2p(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= head[N] { return; }
    var p = particles[i];
    if p.position.w == 0.0 { return; }
    let cell_idx = floor(p.position.xyz);
    let w = weights(p.position.xyz - (cell_idx + 0.5));
    var v = vec3<f32>(0.0);
    var B = mat3x3<f32>();
    for (var gx = 0; gx < 3; gx++) {
        for (var gy = 0; gy < 3; gy++) {
            for (var gz = 0; gz < 3; gz++) {
                let weight = w[gx].x * w[gy].y * w[gz].z;
                let cell_x = cell_idx + vec3<f32>(f32(gx), f32(gy), f32(gz)) - 1.0;
                let dist = (cell_x + 0.5) - p.position.xyz;
                let c = cell_index(cell_x);
                let wv = vec3<f32>(
                    decode(atomicLoad(&cells[c])),
                    decode(atomicLoad(&cells[c + 1u])),
                    decode(atomicLoad(&cells[c + 2u])),
                ) * weight;
                B += mat3x3<f32>(wv * dist.x, wv * dist.y, wv * dist.z);
                v += wv;
            }
        }
    }
    let speed = length(v);
    if speed > MAX_SPEED { v *= MAX_SPEED / speed; }
    let d = vec3<f32>(dims());
    var x = clamp(p.position.xyz + v * DT, vec3<f32>(1.0), d - 2.0);
    // The scene's colliders: pushed out, the velocity into them taken away.
    let hit = push_out(to_world(x), word(RADIUS), head[COLLIDER_LIST]);
    if hit.hit {
        x = clamp(to_cells(hit.p), vec3<f32>(1.0), d - 2.0);
        v -= hit.n * min(dot(v, hit.n), 0.0);
    }
    // Upstream's soft walls: a particle heading through one is turned back.
    let ahead = x + v * DT * 3.0;
    let lo = vec(WALL_MIN);
    let hi = vec(WALL_MAX);
    v += 0.3 * (max(lo - ahead, vec3<f32>(0.0)) - max(ahead - hi, vec3<f32>(0.0)));
    p.position = vec4<f32>(x, 1.0);
    p.v = vec4<f32>(v, 0.0);
    p.C = B * 4.0;
    particles[i] = p;
    if head[SINKING] != 0u {
        let at = to_world(x);
        if all(at >= vec(SINK_MIN)) && all(at <= vec(SINK_MAX)) {
            if head[EMITTING] != 0u {
                // Back at the emitter, as a new particle would be.
                at_emitter(i, i * 7919u + u32(p.v.x * 1e4));
            } else {
                particles[i].position.w = 0.0;
            }
        }
    }
}

@compute @workgroup_size(1)
fn next_tick() { cursor[0] += 1u; }

// The particles as the screen-space fluid's instances: world position and
// drawn radius, colour and optical density (radius 0: not drawn).
@compute @workgroup_size(64)
fn draw(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= head[N] { return; }
    let p = particles[i];
    let out = (head[OUT] + i) * 2u;
    let r = select(0.0, word(DRAW_RADIUS), p.position.w != 0.0);
    instances[out] = vec4<f32>(to_world(p.position.xyz), r);
    instances[out + 1u] = vec4<f32>(vec(COLOR), word(DENSITY_K));
}
