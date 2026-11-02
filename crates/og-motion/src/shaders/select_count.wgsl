// The chosen count per sequence: the first maximum of CLS's count logits
// over the allowed counts (index i is 6 + i tokens; out of reach never).
struct Params { n: u32, length: u32, logits: u32, counts: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> logits: array<f32>;
@group(0) @binding(2) var<storage, read> allowed: array<f32>;
@group(0) @binding(3) var<storage, read_write> count: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let s = id.x;
    if (s >= p.n) { return; }
    var best = 0u;
    var found = false;
    var value = 0.0;
    for (var c = 0u; c < p.counts; c++) {
        if (allowed[s * p.counts + c] <= 0.0) { continue; }
        let v = logits[s * p.length * p.logits + c];
        if (!found || v > value) {
            best = c;
            value = v;
            found = true;
        }
    }
    count[s] = best;
}
