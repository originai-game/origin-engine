// LayerNorm over rows of `width`, eps 1e-5; one invocation per row.
struct Params { rows: u32, width: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32, pad4: u32, pad5: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> x: array<f32>;
@group(0) @binding(2) var<storage, read> w: array<f32>;
@group(0) @binding(3) var<storage, read> b: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let row = id.x;
    if (row >= p.rows) { return; }
    let base = row * p.width;
    var mean = 0.0;
    for (var i = 0u; i < p.width; i++) { mean += x[base + i]; }
    mean /= f32(p.width);
    var variance = 0.0;
    for (var i = 0u; i < p.width; i++) {
        let d = x[base + i] - mean;
        variance += d * d;
    }
    let inv = 1.0 / sqrt(variance / f32(p.width) + 1e-5);
    for (var i = 0u; i < p.width; i++) {
        y[base + i] = (x[base + i] - mean) * inv * w[i] + b[i];
    }
}
