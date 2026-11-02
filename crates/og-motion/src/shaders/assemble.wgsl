// The student's sequence [n, length, width]: CLS (allowed counts projected,
// CLS folded into the bias), the chosen count's embedding, the 8 slots with
// their positions, the 16 queries (length 26); or for the count's pass only
// CLS and the slots (length 9). Element-wise, flat over a 2D grid.
struct Params { n: u32, width: u32, length: u32, pad1: u32, pad2: u32, pad3: u32, pad4: u32, pad5: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> cls: array<f32>;
@group(0) @binding(2) var<storage, read> count_emb: array<f32>;
@group(0) @binding(3) var<storage, read> count: array<u32>;
@group(0) @binding(4) var<storage, read> slots: array<f32>;
@group(0) @binding(5) var<storage, read> slot_pos: array<f32>;
@group(0) @binding(6) var<storage, read> queries: array<f32>;
@group(0) @binding(7) var<storage, read_write> seq: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(num_workgroups) groups: vec3u, @builtin(local_invocation_index) l: u32) {
    let id = (wg.y * groups.x + wg.x) * 256u + l;
    if (id >= p.n * p.length * p.width) { return; }
    let c = id % p.width;
    var r = (id / p.width) % p.length;
    let s = id / (p.width * p.length);
    if (p.length == 9u && r > 0u) { r += 1u; }
    var v = 0.0;
    if (r == 0u) {
        v = cls[s * p.width + c];
    } else if (r == 1u) {
        v = count_emb[count[s] * p.width + c];
    } else if (r < 10u) {
        v = slots[(s * 8u + r - 2u) * p.width + c] + slot_pos[(r - 2u) * p.width + c];
    } else {
        v = queries[(r - 10u) * p.width + c];
    }
    seq[id] = v;
}
