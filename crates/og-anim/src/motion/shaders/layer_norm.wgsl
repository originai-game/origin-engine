// LayerNorm over rows of `width`, eps 1e-5; one workgroup of 64 per row,
// reducing the sums in workgroup memory.
struct Params { rows: u32, width: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32, pad4: u32, pad5: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read> w: array<f32>;
@group(0) @binding(3) var<storage, read> b: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;

var<workgroup> partial: array<f32, 64>;

fn reduce(l: u32, value: f32) -> f32 {
    partial[l] = value;
    workgroupBarrier();
    for (var stride = 32u; stride > 0u; stride = stride / 2u) {
        if (l < stride) { partial[l] += partial[l + stride]; }
        workgroupBarrier();
    }
    let total = partial[0];
    workgroupBarrier();
    return total;
}

@compute @workgroup_size(64)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(num_workgroups) groups: vec3u, @builtin(local_invocation_index) l: u32) {
    let row = wg.y * groups.x + wg.x;
    if (row >= p.rows) { return; }
    let base = row * p.width;
    var sum = 0.0;
    for (var i = l; i < p.width; i += 64u) { sum += x[base + i]; }
    let mean = reduce(l, sum) / f32(p.width);
    var squares = 0.0;
    for (var i = l; i < p.width; i += 64u) {
        let d = x[base + i] - mean;
        squares += d * d;
    }
    let inv = 1.0 / sqrt(reduce(l, squares) / f32(p.width) + 1e-5);
    for (var i = l; i < p.width; i += 64u) {
        y[base + i] = (x[base + i] - mean) * inv * w[i] + b[i];
    }
}
