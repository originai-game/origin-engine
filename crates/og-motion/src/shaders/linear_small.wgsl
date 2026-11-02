// linear.wgsl for few rows: 16×16 tiles, one output per invocation, so a
// short sequence still spreads over many workgroups.
// flags: 1 relu, 2 accumulate into y (residual).
struct Params { rows: u32, k: u32, out: u32, flags: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read> w: array<f32>;
@group(0) @binding(3) var<storage, read> b: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;

var<workgroup> xs: array<array<f32, 16>, 16>;
var<workgroup> ws: array<array<f32, 16>, 16>;

@compute @workgroup_size(16, 16)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(local_invocation_id) l: vec3u) {
    let row = wg.y * 16u + l.y;
    let o = wg.x * 16u + l.x;
    let wo = wg.x * 16u + l.y;
    var acc = 0.0;
    let tiles = (p.k + 15u) / 16u;
    for (var t = 0u; t < tiles; t++) {
        let kx = t * 16u + l.x;
        var xv = 0.0;
        if (row < p.rows && kx < p.k) { xv = x[row * p.k + kx]; }
        xs[l.y][l.x] = xv;
        var wv = 0.0;
        if (wo < p.out && kx < p.k) { wv = w[wo * p.k + kx]; }
        ws[l.y][l.x] = wv;
        workgroupBarrier();
        for (var i = 0u; i < 16u; i++) {
            acc += xs[l.y][i] * ws[l.x][i];
        }
        workgroupBarrier();
    }
    if (row < p.rows && o < p.out) {
        var v = acc + b[o];
        if ((p.flags & 1u) != 0u) { v = max(v, 0.0); }
        let at = row * p.out + o;
        if ((p.flags & 2u) != 0u) { v += y[at]; }
        y[at] = v;
    }
}
