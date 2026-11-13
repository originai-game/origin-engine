// out[s, j, i] = x[s, i, j] for x [n, a, b].
struct Params { n: u32, a: u32, b: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32, pad4: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(num_workgroups) groups: vec3u, @builtin(local_invocation_index) l: u32) {
    let id = (wg.y * groups.x + wg.x) * 256u + l;
    if (id >= p.n * p.a * p.b) { return; }
    let j = id % p.b;
    let i = (id / p.b) % p.a;
    let s = id / (p.a * p.b);
    out[(s * p.b + j) * p.a + i] = x[id];
}
