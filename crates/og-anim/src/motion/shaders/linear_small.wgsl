// linear.wgsl for few rows: 16×16 output tiles, one output per invocation,
// so a short sequence still spreads over many workgroups; 64 deep per
// barrier, since with few rows the chain of k tiles is the latency.
// flags: 1 relu, 2 accumulate into y (residual).
struct Params { rows: u32, k: u32, out: u32, flags: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read> w: array<f32>;
@group(0) @binding(3) var<storage, read> b: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;

// [row or out][k], 16 × 64 each.
var<workgroup> xs: array<f32, 1024>;
var<workgroup> ws: array<f32, 1024>;

@compute @workgroup_size(16, 16)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(local_invocation_id) l: vec3u, @builtin(local_invocation_index) li: u32) {
    let row0 = wg.y * 16u;
    let out0 = wg.x * 16u;
    var acc = 0.0;
    let tiles = (p.k + 63u) / 64u;
    for (var t = 0u; t < tiles; t++) {
        for (var e = 0u; e < 4u; e++) {
            let idx = li + e * 256u;
            let r = idx / 64u;
            let k = t * 64u + idx % 64u;
            var xv = 0.0;
            if (row0 + r < p.rows && k < p.k) { xv = x[(row0 + r) * p.k + k]; }
            xs[idx] = xv;
            var wv = 0.0;
            if (out0 + r < p.out && k < p.k) { wv = w[(out0 + r) * p.k + k]; }
            ws[idx] = wv;
        }
        workgroupBarrier();
        for (var i = 0u; i < 64u; i++) {
            acc += xs[l.y * 64u + i] * ws[l.x * 64u + i];
        }
        workgroupBarrier();
    }
    let row = row0 + l.y;
    let o = out0 + l.x;
    if (row < p.rows && o < p.out) {
        var v = acc + b[o];
        if ((p.flags & 1u) != 0u) { v = max(v, 0.0); }
        let at = row * p.out + o;
        if ((p.flags & 2u) != 0u) { v += y[at]; }
        y[at] = v;
    }
}
