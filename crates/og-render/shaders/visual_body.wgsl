// Visual bodies (SIMULATION_VISUAL): cloth, soft volumes and garments
// simulated on the GPU for looks only; nothing reads this back.
//
// Extended position-based dynamics (Macklin, Müller and Chentanez, "XPBD:
// Position-Based Simulation of Compliant Constrained Dynamics", MIG 2016)
// in small steps (Macklin, Storey, Lu, Terdiman, Chentanez, Jeschke and
// Müller, "Small Steps in Physics Simulation", SCA 2019: one constraint
// pass per substep, no Lagrange multiplier carried), graph-coloured
// Gauss-Seidel distance constraints, long-range attachments (Kim,
// Chentanez and Müller, "Long Range Attachments", SCA 2012), the volume
// constraint of Müller, Heidelberger, Hennix and Ratcliff ("Position Based
// Dynamics", 2007) and hashed self-collision (Teschner et al., "Optimized
// Spatial Hashing for Collision Detection of Deformable Objects", VMV
// 2003). Independent implementation from the papers; see
// third_party/visual-bodies.md.
//
// One workgroup per body: the whole frame's ticks run in this dispatch,
// each phase separated by workgroup and storage barriers.

@group(0) @binding(0) var<storage, read> body: array<u32>;
@group(0) @binding(1) var<storage, read> colliders: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> state: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> grid: array<atomic<u32>>;
// Binding 4, the drawn points, and `emit` come after this file
// (visual_sim.rs: a surface's coarse vertices or a garment's points).

const WG: u32 = 256u;
// Header words (visual_sim.rs `Header`).
const N: u32 = 0u;
const KIND: u32 = 1u;
const COLUMNS: u32 = 2u;
const ROWS: u32 = 3u;
const STEPS: u32 = 4u;
const RESET: u32 = 5u;
const SUB: u32 = 6u;
const TICK: u32 = 7u;
const DT: u32 = 8u;
const KEEP: u32 = 9u;
const GRAVITY: u32 = 10u;
const PLANAR: u32 = 13u;
const PLANE_Z: u32 = 14u;
const WIND: u32 = 15u;
const GUST: u32 = 18u;
const RADIUS: u32 = 19u;
const FRICTION: u32 = 20u;
const THICKNESS: u32 = 21u;
const ALPHA_STRETCH: u32 = 22u;
const ALPHA_BEND: u32 = 23u;
const VOLUME_K: u32 = 24u;
const PRESSURE: u32 = 25u;
const REST_VOLUME: u32 = 26u;
const COLOURS: u32 = 27u;
const COLOUR_TABLE: u32 = 28u;
const INV_MASS: u32 = 29u;
const TETHER: u32 = 30u;
const TRIANGLES: u32 = 31u;
const TRIANGLE_COUNT: u32 = 32u;
const CSR: u32 = 33u;
const CELLS: u32 = 34u;
const NEIGHBOURS: u32 = 35u;
const COLLIDER_LIST: u32 = 36u;
const REACH: u32 = 37u;
const BACKSTOP: u32 = 38u;
const ALPHA_SHEAR: u32 = 39u;
const WORLD: u32 = 40u;   // garment: model to world, 12 words (3x4 rows)
const MODEL: u32 = 52u;   // garment: world to model, 12 words

// Kinds.
const CLOTH: u32 = 0u;
const SOFT: u32 = 1u;
const RING: u32 = 2u;
const GARMENT: u32 = 3u;

// State regions, each N vec4s: position, previous, the pins' targets at
// the frame before and at this frame, a scratch delta.
fn at_pos(i: u32) -> u32 { return i; }
fn at_prev(i: u32) -> u32 { return body[N] + i; }
fn at_old(i: u32) -> u32 { return 2u * body[N] + i; }
fn at_new(i: u32) -> u32 { return 3u * body[N] + i; }
fn at_delta(i: u32) -> u32 { return 4u * body[N] + i; }
// Garment: the targets' skin normals (the frame before, this frame).
fn at_old_normal(i: u32) -> u32 { return 5u * body[N] + i; }
fn at_new_normal(i: u32) -> u32 { return 6u * body[N] + i; }

fn word(i: u32) -> f32 { return bitcast<f32>(body[i]); }
fn vec(i: u32) -> vec3<f32> { return vec3<f32>(word(i), word(i + 1u), word(i + 2u)); }

var<workgroup> partial: array<f32, 256>;
var<workgroup> partial_b: array<f32, 256>;

fn sync() {
    storageBarrier();
    workgroupBarrier();
}

fn unit(v: vec3<f32>) -> vec3<f32> {
    let d = dot(v, v);
    if d > 1e-20 { return v * inverseSqrt(d); }
    return vec3<f32>(0.0);
}

fn planar(p: vec3<f32>) -> vec3<f32> {
    if body[PLANAR] != 0u { return vec3<f32>(p.xy, word(PLANE_Z)); }
    return p;
}

fn inv_mass(i: u32) -> f32 { return word(body[INV_MASS] + i); }

// ---- wind ---------------------------------------------------------------
// The air moves along `wind` at its magnitude in metres a second, gusting
// and eddying as it goes (`air`), so the pressure differs across a cloth
// and its folds run down it. Each
// triangle meets it with its own velocity: drag along the relative flow
// and lift across it, by the flat-plate split of the normal pressure that
// NVIDIA NvCloth documents for its cloth wind (drag ∝ |cos θ| |v|², lift
// ∝ cos θ sin θ |v|²); a point takes a third of each of its triangles'
// force over a third of their area. Scaled so a still face square to the
// wind feels `wind` (the og.Cloth field's acceleration).
const DRAG: f32 = 1.0;
const LIFT: f32 = 0.7;

// Smooth value noise in [-1, 1] on a unit lattice.
fn lattice(c: vec3<i32>, salt: u32) -> f32 {
    var h = (bitcast<u32>(c.x) * 0x8da6b343u) ^ (bitcast<u32>(c.y) * 0xd8163841u)
        ^ (bitcast<u32>(c.z) * 0xcb1ab31fu) ^ (salt * 0x9e3779b9u);
    h ^= h >> 15u;
    h *= 0x2c1b3c6du;
    h ^= h >> 12u;
    h *= 0x297a2d39u;
    h ^= h >> 15u;
    return f32(h >> 8u) / f32(1u << 23u) - 1.0;
}

fn value_noise(p: vec3<f32>, salt: u32) -> f32 {
    let c = floor(p);
    let f = p - c;
    let w = f * f * (3.0 - 2.0 * f);
    let i = vec3<i32>(c);
    var out = 0.0;
    for (var k = 0u; k < 8u; k++) {
        let o = vec3<i32>(i32(k & 1u), i32((k >> 1u) & 1u), i32(k >> 2u));
        let wx = select(1.0 - w.x, w.x, o.x == 1);
        let wy = select(1.0 - w.y, w.y, o.y == 1);
        let wz = select(1.0 - w.z, w.z, o.z == 1);
        out += wx * wy * wz * lattice(i + o, salt);
    }
    return out;
}

// The air at `p`, `t` seconds in: `wind` with gusts as frozen turbulence
// carried along at the wind's speed (Taylor's hypothesis): a swell 2.5 m
// across in its strength, and eddies 0.9 m and 0.35 m across in all three
// directions, which are what make a banner ripple rather than lean.
fn air(p: vec3<f32>, t: f32) -> vec3<f32> {
    let w = vec(WIND);
    let speed = length(w);
    if speed <= 0.0 { return w; }
    let along = w / speed;
    let g = word(GUST);
    let q = p - w * t;
    let swell = value_noise(q / 2.5 + vec3<f32>(0.0, 0.0, 0.3 * t), 1u);
    var eddy = vec3<f32>(0.0);
    for (var k = 0u; k < 3u; k++) {
        eddy[k] = 0.65 * value_noise(q / 0.9 + vec3<f32>(0.7 * t), 2u + k)
            + 0.35 * value_noise(q / 0.35 + vec3<f32>(1.6 * t), 5u + k);
    }
    return w * (1.0 + g * swell) + eddy * (0.6 * g * speed);
}

fn aero(i: u32, h: f32, t: f32) -> vec3<f32> {
    let speed = length(vec(WIND));
    if speed <= 0.0 { return vec3<f32>(0.0); }
    let tris = body[TRIANGLES];
    let csr = body[CSR];
    let first = body[csr + i];
    let last = body[csr + i + 1u];
    let rows = csr + body[N] + 1u;
    var force = vec3<f32>(0.0);
    var area = 0.0;
    for (var k = first; k < last; k++) {
        let f = tris + body[rows + k] * 3u;
        let a = body[f]; let b = body[f + 1u]; let c = body[f + 2u];
        let pa = state[at_pos(a)].xyz; let pb = state[at_pos(b)].xyz; let pc = state[at_pos(c)].xyz;
        let e = cross(pb - pa, pc - pa);
        let twice = length(e);
        if twice < 1e-10 { continue; }
        let n = e / twice;
        let share = twice / 6.0;
        area += share;
        let v = (pa + pb + pc - state[at_prev(a)].xyz - state[at_prev(b)].xyz - state[at_prev(c)].xyz) / (3.0 * h);
        let rel = air((pa + pb + pc) / 3.0, t) - v;
        let len = length(rel);
        if len < 1e-5 { continue; }
        let flow = rel / len;
        let cos_t = dot(n, flow);
        let q = len * len / (speed * speed);
        force += share * q * (DRAG * abs(cos_t) * flow + LIFT * cos_t * (n - flow * cos_t));
    }
    if area <= 0.0 { return vec3<f32>(0.0); }
    return force * (speed / area);
}

// ---- scene colliders -------------------------------------------------------
// shaders/scene_colliders.wgsl (included before this file) pushes a point
// out of the body's list (header COLLIDER_LIST) of `colliders`.

// ---- self-collision ---------------------------------------------------------
// grid: counts (CELLS), starts (CELLS), the points sorted by cell (N), then
// each point's neighbour count and NEIGHBOURS indices.

fn cell_of(p: vec3<f32>, reach: f32) -> vec3<i32> {
    return vec3<i32>(floor(p / reach));
}

fn hash_cell(c: vec3<i32>) -> u32 {
    let h = (u32(c.x) * 73856093u) ^ (u32(c.y) * 19349663u) ^ (u32(c.z) * 83492791u);
    return h & (body[CELLS] - 1u);
}

fn build_neighbours(lid: u32) {
    let n = body[N];
    let cells = body[CELLS];
    let k_max = body[NEIGHBOURS];
    let thickness = word(THICKNESS);
    let reach = thickness * 1.5;
    let starts = cells;
    let sorted = 2u * cells;
    let lists = 2u * cells + n;
    for (var c = lid; c < cells; c += WG) { atomicStore(&grid[c], 0u); }
    sync();
    for (var i = lid; i < n; i += WG) {
        let h = hash_cell(cell_of(state[at_pos(i)].xyz, reach));
        state[at_delta(i)].w = bitcast<f32>(h);
        atomicAdd(&grid[h], 1u);
    }
    sync();
    // Exclusive prefix sum of the counts: each thread its run, then the runs.
    let run = (cells + WG - 1u) / WG;
    var sum = 0u;
    for (var c = lid * run; c < min(cells, (lid + 1u) * run); c++) { sum += atomicLoad(&grid[c]); }
    partial[lid] = bitcast<f32>(sum);
    sync();
    if lid == 0u {
        var total = 0u;
        for (var t = 0u; t < WG; t++) {
            let s = bitcast<u32>(partial[t]);
            partial[t] = bitcast<f32>(total);
            total += s;
        }
    }
    sync();
    var at = bitcast<u32>(partial[lid]);
    for (var c = lid * run; c < min(cells, (lid + 1u) * run); c++) {
        let count = atomicLoad(&grid[c]);
        atomicStore(&grid[starts + c], at);
        atomicStore(&grid[c], at);
        at += count;
    }
    sync();
    for (var i = lid; i < n; i += WG) {
        let h = bitcast<u32>(state[at_delta(i)].w);
        let slot = atomicAdd(&grid[h], 1u);
        atomicStore(&grid[sorted + slot], i);
    }
    sync();
    // The scatter's order within a cell is the atomics'; sort each run by
    // index so every frame is the same.
    for (var c = lid; c < cells; c += WG) {
        let lo = atomicLoad(&grid[starts + c]);
        let hi = atomicLoad(&grid[c]);
        for (var a = lo + 1u; a < hi; a++) {
            let v = atomicLoad(&grid[sorted + a]);
            var b = a;
            while b > lo && atomicLoad(&grid[sorted + b - 1u]) > v {
                atomicStore(&grid[sorted + b], atomicLoad(&grid[sorted + b - 1u]));
                b -= 1u;
            }
            atomicStore(&grid[sorted + b], v);
        }
    }
    sync();
    for (var i = lid; i < n; i += WG) {
        let p = state[at_pos(i)].xyz;
        let rest = state[at_new(i)].xyz;
        let base = cell_of(p, reach);
        var seen: array<u32, 27>;
        var found = 0u;
        var count = 0u;
        for (var dz = -1; dz <= 1; dz++) {
            for (var dy = -1; dy <= 1; dy++) {
                for (var dx = -1; dx <= 1; dx++) {
                    let h = hash_cell(base + vec3<i32>(dx, dy, dz));
                    var twice = false;
                    for (var s = 0u; s < found; s++) { if seen[s] == h { twice = true; } }
                    if twice { continue; }
                    seen[found] = h;
                    found += 1u;
                    let lo = atomicLoad(&grid[starts + h]);
                    let hi = atomicLoad(&grid[h]);
                    for (var a = lo; a < hi && count < k_max; a++) {
                        let j = atomicLoad(&grid[sorted + a]);
                        if j == i { continue; }
                        let q = state[at_pos(j)].xyz;
                        // Near neighbours are held by the links.
                        if dot(q - p, q - p) < reach * reach && length(state[at_new(j)].xyz - rest) >= thickness {
                            atomicStore(&grid[lists + n + i * k_max + count], j);
                            count += 1u;
                        }
                    }
                }
            }
        }
        atomicStore(&grid[lists + i], count);
    }
    sync();
}

fn self_collide(lid: u32) {
    let n = body[N];
    let cells = body[CELLS];
    let k_max = body[NEIGHBOURS];
    let lists = 2u * cells + n;
    let thickness = word(THICKNESS);
    for (var i = lid; i < n; i += WG) {
        let wi = inv_mass(i);
        var d = vec3<f32>(0.0);
        if wi > 0.0 {
            let p = state[at_pos(i)].xyz;
            let count = atomicLoad(&grid[lists + i]);
            for (var k = 0u; k < count; k++) {
                let j = atomicLoad(&grid[lists + n + i * k_max + k]);
                let wj = inv_mass(j);
                let e = p - state[at_pos(j)].xyz;
                let len = length(e);
                if len < thickness && len > 1e-9 {
                    d += e / len * ((thickness - len) * wi / (wi + wj));
                }
            }
        }
        state[at_delta(i)] = vec4<f32>(d, 0.0);
    }
    sync();
    for (var i = lid; i < n; i += WG) {
        state[at_pos(i)] = vec4<f32>(planar(state[at_pos(i)].xyz + state[at_delta(i)].xyz), 0.0);
    }
    sync();
}

// ---- constraints ------------------------------------------------------------

fn distance_constraints(lid: u32, h2: f32) {
    let colours = body[COLOURS];
    let table = body[COLOUR_TABLE];
    for (var c = 0u; c < colours; c++) {
        let first = body[table + c * 2u];
        let count = body[table + c * 2u + 1u];
        for (var e = lid; e < count; e += WG) {
            let at = first + e * 4u;
            let a = body[at];
            let b = body[at + 1u];
            let rest = bitcast<f32>(body[at + 2u]);
            var slot = ALPHA_STRETCH;
            if body[at + 3u] == 1u { slot = ALPHA_SHEAR; } else if body[at + 3u] == 2u { slot = ALPHA_BEND; }
            let alpha = word(slot) / h2;
            let wa = inv_mass(a);
            let wb = inv_mass(b);
            let w = wa + wb;
            if w <= 0.0 { continue; }
            let pa = state[at_pos(a)].xyz;
            let pb = state[at_pos(b)].xyz;
            let d = pb - pa;
            let len = length(d);
            if len <= 1e-9 { continue; }
            let lambda = (len - rest) / (w + alpha);
            let n = d / len;
            state[at_pos(a)] = vec4<f32>(pa + n * (lambda * wa), 0.0);
            state[at_pos(b)] = vec4<f32>(pb - n * (lambda * wb), 0.0);
        }
        sync();
    }
}

// Long-range attachments: no point further from its nearest pin than at rest.
fn tethers(lid: u32, t: f32) {
    let n = body[N];
    let base = body[TETHER];
    if base == 0u { return; }
    for (var i = lid; i < n; i += WG) {
        let pin = body[base + i * 2u];
        if pin == 0xffffffffu { continue; }
        let limit = word(base + i * 2u + 1u);
        let anchor = mix(state[at_old(pin)].xyz, state[at_new(pin)].xyz, t);
        let p = state[at_pos(i)].xyz;
        let d = p - anchor;
        let len = length(d);
        if len > limit {
            state[at_pos(i)] = vec4<f32>(planar(anchor + d * (limit / len)), 0.0);
        }
    }
    sync();
}

// The sum of partial over the workgroup (every thread wrote its share).
fn reduce(lid: u32) -> vec2<f32> {
    for (var s = WG / 2u; s > 0u; s >>= 1u) {
        if lid < s {
            partial[lid] += partial[lid + s];
            partial_b[lid] += partial_b[lid + s];
        }
        workgroupBarrier();
    }
    let r = vec2<f32>(partial[0], partial_b[0]);
    workgroupBarrier();
    return r;
}

// The enclosed volume (a ring: its area) back to its target in one XPBD
// step, every point moved along the volume's gradient.
fn volume(lid: u32, h2: f32) {
    let n = body[N];
    let k = word(VOLUME_K);
    if k <= 0.0 || body[KIND] == CLOTH || body[KIND] == GARMENT { return; }
    let origin = state[at_new(0u)].xyz;
    let ring = body[KIND] == RING;
    var v = 0.0;
    if ring {
        for (var i = lid; i < n; i += WG) {
            let a = state[at_pos(i)].xyz - origin;
            let b = state[at_pos((i + 1u) % n)].xyz - origin;
            v += (a.x * b.y - b.x * a.y) * 0.5;
        }
    } else {
        let tris = body[TRIANGLES];
        for (var t = lid; t < body[TRIANGLE_COUNT]; t += WG) {
            let a = state[at_pos(body[tris + t * 3u])].xyz - origin;
            let b = state[at_pos(body[tris + t * 3u + 1u])].xyz - origin;
            let c = state[at_pos(body[tris + t * 3u + 2u])].xyz - origin;
            v += dot(a, cross(b, c)) / 6.0;
        }
    }
    partial[lid] = v;
    // The gradient at each point, and the sum of w |gradient|^2.
    var g2 = 0.0;
    for (var i = lid; i < n; i += WG) {
        var g = vec3<f32>(0.0);
        if ring {
            let prev = state[at_pos((i + n - 1u) % n)].xyz;
            let next = state[at_pos((i + 1u) % n)].xyz;
            g = vec3<f32>(next.y - prev.y, prev.x - next.x, 0.0) * 0.5;
        } else {
            let tris = body[TRIANGLES];
            let csr = body[CSR];
            let faces = csr + n + 1u;
            for (var f = body[csr + i]; f < body[csr + i + 1u]; f++) {
                let t = body[faces + f];
                let ia = body[tris + t * 3u];
                let ib = body[tris + t * 3u + 1u];
                let ic = body[tris + t * 3u + 2u];
                let a = state[at_pos(ia)].xyz - origin;
                let b = state[at_pos(ib)].xyz - origin;
                let c = state[at_pos(ic)].xyz - origin;
                if ia == i { g += cross(b, c) / 6.0; }
                if ib == i { g += cross(c, a) / 6.0; }
                if ic == i { g += cross(a, b) / 6.0; }
            }
        }
        state[at_delta(i)] = vec4<f32>(g, 0.0);
        g2 += inv_mass(i) * dot(g, g);
    }
    partial_b[lid] = g2;
    sync();
    let sums = reduce(lid);
    let c = sums.x - word(REST_VOLUME) * word(PRESSURE);
    // Stiffness 1 restores the volume in a step; lower is compliant.
    let alpha = (1.0 - k) / max(k, 1e-6) * 1e-3 / h2;
    let denom = sums.y + alpha;
    if denom > 1e-12 {
        let lambda = -c / denom;
        for (var i = lid; i < n; i += WG) {
            let p = state[at_pos(i)].xyz + state[at_delta(i)].xyz * (lambda * inv_mass(i));
            state[at_pos(i)] = vec4<f32>(planar(p), 0.0);
        }
    }
    sync();
}

// Garment: each point within its reach of its skinned target and no
// further than the backstop behind it (garment.rs's constraints).
fn garment_limits(lid: u32, t: f32) {
    let n = body[N];
    let reach = body[REACH];
    let backstop = word(BACKSTOP);
    for (var i = lid; i < n; i += WG) {
        let target_p = mix(state[at_old(i)].xyz, state[at_new(i)].xyz, t);
        let limit = word(reach + i);
        var p = state[at_pos(i)].xyz;
        if limit <= 0.0 {
            p = target_p;
        } else {
            let d = p - target_p;
            let len = length(d);
            if len > limit { p = target_p + d * (limit / len); }
            let nrm = unit(mix(state[at_old_normal(i)].xyz, state[at_new_normal(i)].xyz, t));
            let behind = dot(p - target_p, nrm) + backstop;
            if behind < 0.0 { p -= nrm * behind; }
        }
        state[at_pos(i)] = vec4<f32>(p, 0.0);
    }
    sync();
}

fn collide(lid: u32) {
    let radius = word(RADIUS);
    if radius <= 0.0 { return; }
    let n = body[N];
    let friction = clamp(word(FRICTION), 0.0, 1.0);
    for (var i = lid; i < n; i += WG) {
        if inv_mass(i) <= 0.0 { continue; }
        let p = state[at_pos(i)].xyz;
        let h = push_out(p, radius, body[COLLIDER_LIST]);
        if h.hit {
            let q = planar(h.p);
            state[at_pos(i)] = vec4<f32>(q, 0.0);
            if friction > 0.0 {
                // Friction: a point resting on a solid loses part of its slide.
                let prev = state[at_prev(i)].xyz;
                let v = q - prev;
                let slide = v - h.n * dot(h.n, v);
                state[at_prev(i)] = vec4<f32>(prev + slide * friction, 0.0);
            }
        }
    }
    sync();
}

@compute @workgroup_size(256)
fn solve(@builtin(local_invocation_index) lid: u32) {
    let n = body[N];
    let kind = body[KIND];
    if body[RESET] != 0u {
        for (var i = lid; i < n; i += WG) {
            let p = state[at_new(i)];
            state[at_pos(i)] = p;
            state[at_prev(i)] = p;
            state[at_old(i)] = p;
            if kind == GARMENT { state[at_old_normal(i)] = state[at_new_normal(i)]; }
        }
        sync();
    }
    let steps = body[STEPS];
    let sub = max(body[SUB], 1u);
    let total = f32(max(steps * sub, 1u));
    let h = word(DT) / f32(sub);
    let h2 = max(h * h, 1e-12);
    let keep = word(KEEP);
    let gravity = vec(GRAVITY);
    let wind = vec(WIND);
    let windy = (kind == CLOTH || kind == GARMENT) && dot(wind, wind) > 0.0 && body[TRIANGLE_COUNT] > 0u;
    for (var s = 0u; s < steps; s++) {
        let tick = body[TICK] + s;
        if word(THICKNESS) > 0.0 { build_neighbours(lid); }
        for (var k = 0u; k < sub; k++) {
            let t = f32(s * sub + k + 1u) / total;
            // Seconds since tick 0, wrapped every 2^16 ticks to keep it exact.
            let clock = (f32(tick & 0xffffu) + f32(k + 1u) / f32(sub)) * word(DT);
            if windy {
                for (var i = lid; i < n; i += WG) { state[at_delta(i)] = vec4<f32>(aero(i, h, clock), 0.0); }
                sync();
            }
            // Integrate: the pins and garment targets move to this
            // substep's place between the frame's two.
            for (var i = lid; i < n; i += WG) {
                let p = state[at_pos(i)].xyz;
                if inv_mass(i) <= 0.0 {
                    let pin = mix(state[at_old(i)].xyz, state[at_new(i)].xyz, t);
                    state[at_prev(i)] = vec4<f32>(pin, 0.0);
                    state[at_pos(i)] = vec4<f32>(pin, 0.0);
                    continue;
                }
                var a = gravity;
                if windy { a += state[at_delta(i)].xyz; }
                let v = (p - state[at_prev(i)].xyz) * keep;
                state[at_prev(i)] = vec4<f32>(p, 0.0);
                state[at_pos(i)] = vec4<f32>(planar(p + v + a * h2), 0.0);
            }
            sync();
            distance_constraints(lid, h2);
            volume(lid, h2);
            if kind == GARMENT {
                garment_limits(lid, t);
            } else {
                tethers(lid, t);
            }
            if word(THICKNESS) > 0.0 { self_collide(lid); }
            collide(lid);
        }
    }
    // The drawn points: world positions (a garment's in its model's space).
    for (var i = lid; i < n; i += WG) {
        var p = state[at_pos(i)].xyz;
        if kind == GARMENT {
            let p4 = vec4<f32>(p, 1.0);
            p = vec3<f32>(dot(row(MODEL), p4), dot(row(MODEL + 4u), p4), dot(row(MODEL + 8u), p4));
        }
        emit(i, p);
    }
}

fn row(at: u32) -> vec4<f32> { return vec4<f32>(word(at), word(at + 1u), word(at + 2u), word(at + 3u)); }
