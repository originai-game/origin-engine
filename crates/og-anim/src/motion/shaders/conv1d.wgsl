// Conv1d, 'same' padding, x [n, cin, tin], w [cout, cin, k], y [n, cout, t],
// as a matmul: rows are (sequence, frame), the depth is (channel, tap), the
// weights are already [cout, cin·k]. Tiled as linear.wgsl. Token mask
// m(s, t) = mask[s·16 + t / fpt].
// flags: 1 relu input, 2 mask input, 4 upsample input ×2 (tin = t/2,
// nearest), 8 relu output, 16 accumulate into y (residual), 32 mask output.
struct Params { n: u32, cin: u32, cout: u32, t: u32, kernel: u32, dilation: u32, flags: u32, fpt: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read> w: array<f32>;
@group(0) @binding(3) var<storage, read> b: array<f32>;
@group(0) @binding(4) var<storage, read> mask: array<f32>;
@group(0) @binding(5) var<storage, read_write> y: array<f32>;

var<workgroup> xs: array<f32, 1024>;
var<workgroup> ws: array<f32, 1024>;

fn input(row: u32, kk: u32) -> f32 {
    let s = row / p.t;
    let t = row % p.t;
    let c = kk / p.kernel;
    let k = kk % p.kernel;
    let at = i32(t) + i32(k * p.dilation) - i32(p.dilation * (p.kernel / 2u));
    if (at < 0 || at >= i32(p.t)) { return 0.0; }
    var src = u32(at);
    var tin = p.t;
    if ((p.flags & 4u) != 0u) {
        src = src / 2u;
        tin = p.t / 2u;
    }
    var v = x[(s * p.cin + c) * tin + src];
    if ((p.flags & 1u) != 0u) { v = max(v, 0.0); }
    if ((p.flags & 2u) != 0u) { v *= mask[s * 16u + u32(at) / p.fpt]; }
    return v;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(local_invocation_id) l: vec3u, @builtin(local_invocation_index) li: u32) {
    let rows = p.n * p.t;
    let depth = p.cin * p.kernel;
    let row0 = wg.y * 64u;
    let out0 = wg.x * 64u;
    var acc: array<f32, 16>;
    let tiles = (depth + 15u) / 16u;
    for (var t = 0u; t < tiles; t++) {
        for (var e = 0u; e < 4u; e++) {
            let idx = li + e * 256u;
            let r = idx / 16u;
            let k = t * 16u + idx % 16u;
            var xv = 0.0;
            if (row0 + r < rows && k < depth) { xv = input(row0 + r, k); }
            xs[(idx % 16u) * 64u + r] = xv;
            var wv = 0.0;
            if (out0 + r < p.cout && k < depth) { wv = w[(out0 + r) * depth + k]; }
            ws[(idx % 16u) * 64u + r] = wv;
        }
        workgroupBarrier();
        for (var k = 0u; k < 16u; k++) {
            let a = vec4f(xs[k * 64u + l.y * 4u], xs[k * 64u + l.y * 4u + 1u], xs[k * 64u + l.y * 4u + 2u], xs[k * 64u + l.y * 4u + 3u]);
            let c = vec4f(ws[k * 64u + l.x * 4u], ws[k * 64u + l.x * 4u + 1u], ws[k * 64u + l.x * 4u + 2u], ws[k * 64u + l.x * 4u + 3u]);
            for (var i = 0u; i < 4u; i++) {
                acc[i * 4u] += a[i] * c.x;
                acc[i * 4u + 1u] += a[i] * c.y;
                acc[i * 4u + 2u] += a[i] * c.z;
                acc[i * 4u + 3u] += a[i] * c.w;
            }
        }
        workgroupBarrier();
    }
    for (var i = 0u; i < 4u; i++) {
        let row = row0 + l.y * 4u + i;
        if (row >= rows) { continue; }
        let s = row / p.t;
        let t = row % p.t;
        for (var j = 0u; j < 4u; j++) {
            let o = out0 + l.x * 4u + j;
            if (o >= p.cout) { continue; }
            var v = acc[i * 4u + j] + b[o];
            if ((p.flags & 8u) != 0u) { v = max(v, 0.0); }
            let at = (s * p.cout + o) * p.t + t;
            if ((p.flags & 16u) != 0u) { v += y[at]; }
            if ((p.flags & 32u) != 0u) { v *= mask[s * 16u + t / p.fpt]; }
            y[at] = v;
        }
    }
}
