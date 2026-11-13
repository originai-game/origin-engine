// Masked multi-head self-attention over sequences of `length` (≤ 32) rows of
// qkv [q | k | v] (each `heads·dh` wide); one workgroup of 64 per
// (sequence, head, query row): one invocation per key scores it, then one
// per output channel sums the values. mask[q·length + k] = 1 where the query
// may not read the key.
struct Params { n: u32, heads: u32, dh: u32, length: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> qkv: array<f32>;
@group(0) @binding(2) var<storage, read> mask: array<u32>;
@group(0) @binding(3) var<storage, read_write> out: array<f32>;

var<workgroup> scores: array<f32, 32>;

@compute @workgroup_size(64)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(num_workgroups) groups: vec3u, @builtin(local_invocation_index) l: u32) {
    let id = wg.y * groups.x + wg.x;
    if (id >= p.n * p.heads * p.length) { return; }
    let q = id % p.length;
    let head = (id / p.length) % p.heads;
    let s = id / (p.length * p.heads);
    let width = p.heads * p.dh;
    let stride = 3u * width;
    let qa = (s * p.length + q) * stride + head * p.dh;
    if (l < p.length) {
        var score = -3.0e38;
        if (mask[q * p.length + l] == 0u) {
            let ka = (s * p.length + l) * stride + width + head * p.dh;
            var dot = 0.0;
            for (var d = 0u; d < p.dh; d++) { dot += qkv[qa + d] * qkv[ka + d]; }
            score = dot / sqrt(f32(p.dh));
        }
        scores[l] = score;
    }
    workgroupBarrier();
    var best = -3.0e38;
    for (var k = 0u; k < p.length; k++) { best = max(best, scores[k]); }
    var total = 0.0;
    for (var k = 0u; k < p.length; k++) {
        if (mask[q * p.length + k] == 0u) { total += exp(scores[k] - best); }
    }
    let oa = (s * p.length + q) * width + head * p.dh;
    for (var d = l; d < p.dh; d += 64u) {
        var acc = 0.0;
        for (var k = 0u; k < p.length; k++) {
            if (mask[q * p.length + k] != 0u) { continue; }
            let va = (s * p.length + k) * stride + 2u * width + head * p.dh;
            acc += exp(scores[k] - best) * qkv[va + d];
        }
        out[oa + d] = acc / total;
    }
}
