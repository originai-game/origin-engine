// One decoder level's condition rows [n·positions, width + 2·per]: each of
// the position's `per` frames that has a pose condition replaces its part
// (width / per channels) of h [n, width, positions] with the condition's
// projection ht [n·64, part]; then the frames' ground velocity ext [n·64, 2].
struct Params { n: u32, width: u32, positions: u32, per: u32, pad0: u32, pad1: u32, pad2: u32, pad3: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> h: array<f32>;
@group(0) @binding(2) var<storage, read> ht: array<f32>;
@group(0) @binding(3) var<storage, read> has: array<f32>;
@group(0) @binding(4) var<storage, read> ext: array<f32>;
@group(0) @binding(5) var<storage, read_write> out: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) wg: vec3u, @builtin(num_workgroups) groups: vec3u, @builtin(local_invocation_index) l: u32) {
    let id = (wg.y * groups.x + wg.x) * 256u + l;
    let row_width = p.width + 2u * p.per;
    if (id >= p.n * p.positions * row_width) { return; }
    let j = id % row_width;
    let pos = (id / row_width) % p.positions;
    let s = id / (row_width * p.positions);
    let frames = p.positions * p.per;
    var v = 0.0;
    if (j < p.width) {
        let part = p.width / p.per;
        let frame = pos * p.per + j / part;
        if (has[s * frames + frame] > 0.0) {
            v = ht[(s * frames + frame) * part + j % part];
        } else {
            v = h[(s * p.width + j) * p.positions + pos];
        }
    } else {
        v = ext[(s * frames + pos * p.per) * 2u + (j - p.width)];
    }
    out[id] = v;
}
