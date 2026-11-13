// y[r,o] = b[o] + Σk x[r,k]·w[o,k] (PyTorch Linear layout). 64×64 output
// tiles, 16 deep; each of the 256 invocations computes 4×4 outputs.
// flags: 1 relu, 2 accumulate into y (residual).
struct Params { rows: u32, k: u32, out: u32, flags: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read> w: array<f32>;
@group(0) @binding(3) var<storage, read> b: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;

// [k][row] and [k][out], so the inner loop reads four neighbours.
var<workgroup> xs: array<f32, 1024>;
var<workgroup> ws: array<f32, 1024>;

@compute @workgroup_size(16, 16)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(local_invocation_id) l: vec3u, @builtin(local_invocation_index) li: u32) {
    let row0 = wg.y * 64u;
    let out0 = wg.x * 64u;
    var acc: array<f32, 16>;
    let tiles = (p.k + 15u) / 16u;
    for (var t = 0u; t < tiles; t++) {
        for (var e = 0u; e < 4u; e++) {
            let idx = li + e * 256u;
            let r = idx / 16u;
            let k = t * 16u + idx % 16u;
            var xv = 0.0;
            if (row0 + r < p.rows && k < p.k) { xv = x[(row0 + r) * p.k + k]; }
            xs[(idx % 16u) * 64u + r] = xv;
            var wv = 0.0;
            if (out0 + r < p.out && k < p.k) { wv = w[(out0 + r) * p.k + k]; }
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
        for (var j = 0u; j < 4u; j++) {
            let o = out0 + l.x * 4u + j;
            if (row < p.rows && o < p.out) {
                var v = acc[i * 4u + j] + b[o];
                if ((p.flags & 1u) != 0u) { v = max(v, 0.0); }
                let at = row * p.out + o;
                if ((p.flags & 2u) != 0u) { v += y[at]; }
                y[at] = v;
            }
        }
    }
}
