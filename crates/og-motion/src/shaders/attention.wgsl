// Masked multi-head self-attention over sequences of `length` (≤ 32) rows of
// qkv [q | k | v] (each `heads·dh` wide); one invocation per (sequence, head,
// query row). mask[q·length + k] = 1 where the query may not read the key.
struct Params { n: u32, heads: u32, dh: u32, length: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> qkv: array<f32>;
@group(0) @binding(2) var<storage, read> mask: array<u32>;
@group(0) @binding(3) var<storage, read_write> out: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let total = p.n * p.heads * p.length;
    if (id.x >= total) { return; }
    let q = id.x % p.length;
    let head = (id.x / p.length) % p.heads;
    let s = id.x / (p.length * p.heads);
    let width = p.heads * p.dh;
    let stride = 3u * width;
    let qa = (s * p.length + q) * stride + head * p.dh;
    let scale = 1.0 / sqrt(f32(p.dh));
    var scores: array<f32, 32>;
    var best = -3.0e38;
    for (var k = 0u; k < p.length; k++) {
        if (mask[q * p.length + k] != 0u) { continue; }
        let ka = (s * p.length + k) * stride + width + head * p.dh;
        var dot = 0.0;
        for (var d = 0u; d < p.dh; d++) { dot += qkv[qa + d] * qkv[ka + d]; }
        scores[k] = dot * scale;
        best = max(best, scores[k]);
    }
    var total_weight = 0.0;
    for (var k = 0u; k < p.length; k++) {
        if (mask[q * p.length + k] != 0u) { scores[k] = 0.0; continue; }
        scores[k] = exp(scores[k] - best);
        total_weight += scores[k];
    }
    let oa = (s * p.length + q) * width + head * p.dh;
    for (var d = 0u; d < p.dh; d++) {
        var acc = 0.0;
        for (var k = 0u; k < p.length; k++) {
            let va = (s * p.length + k) * stride + 2u * width + head * p.dh;
            acc += scores[k] * qkv[va + d];
        }
        out[oa + d] = acc / total_weight;
    }
}
