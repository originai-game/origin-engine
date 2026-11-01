//! Greedy voxel surface meshing.

use super::*;

/// A greedy-meshed quad: its four corners (voxel units, MagicaVoxel's
/// frame), its outward normal and its colour index.
pub(crate) struct Quad {
    pub(crate) corners: [[f32; 3]; 4],
    pub(crate) normal: [f32; 3],
    pub(crate) color: u8,
}

/// Greedy meshing of one model: per axis and direction, per slice, maximal
/// rectangles of one colour whose face is exposed.
pub(crate) fn mesh_model(m: &VoxModel) -> Vec<Quad> {
    let [sx, sy, sz] = m.size.map(|v| v as usize);
    let mut grid = vec![0u8; sx * sy * sz];
    let idx = |x: usize, y: usize, z: usize| x + sx * (y + sy * z);
    for v in &m.voxels {
        grid[idx(v[0] as usize, v[1] as usize, v[2] as usize)] = v[3];
    }
    let at = |x: i64, y: i64, z: i64| -> u8 {
        if x < 0 || y < 0 || z < 0 || x >= sx as i64 || y >= sy as i64 || z >= sz as i64 {
            0
        } else {
            grid[idx(x as usize, y as usize, z as usize)]
        }
    };
    let dims = [sx, sy, sz];
    let mut quads = Vec::new();
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for dir in [-1i64, 1] {
            for slice in 0..dims[axis] {
                // The mask of this slice: the colour whose face along `dir`
                // is exposed, 0 where none.
                let mut mask = vec![0u8; dims[u] * dims[v]];
                for j in 0..dims[v] {
                    for i in 0..dims[u] {
                        let mut p = [0i64; 3];
                        p[axis] = slice as i64;
                        p[u] = i as i64;
                        p[v] = j as i64;
                        let c = at(p[0], p[1], p[2]);
                        if c == 0 {
                            continue;
                        }
                        let mut q = p;
                        q[axis] += dir;
                        if at(q[0], q[1], q[2]) == 0 {
                            mask[i + dims[u] * j] = c;
                        }
                    }
                }
                let mut j = 0;
                while j < dims[v] {
                    let mut i = 0;
                    while i < dims[u] {
                        let c = mask[i + dims[u] * j];
                        if c == 0 {
                            i += 1;
                            continue;
                        }
                        let mut w = 1;
                        while i + w < dims[u] && mask[i + w + dims[u] * j] == c {
                            w += 1;
                        }
                        let mut h = 1;
                        'rows: while j + h < dims[v] {
                            for k in 0..w {
                                if mask[i + k + dims[u] * (j + h)] != c {
                                    break 'rows;
                                }
                            }
                            h += 1;
                        }
                        for jj in 0..h {
                            for ii in 0..w {
                                mask[i + ii + dims[u] * (j + jj)] = 0;
                            }
                        }
                        let face = if dir > 0 {
                            slice as f32 + 1.0
                        } else {
                            slice as f32
                        };
                        let corner = |du: f32, dv: f32| {
                            let mut p = [0f32; 3];
                            p[axis] = face;
                            p[u] = i as f32 + du;
                            p[v] = j as f32 + dv;
                            p
                        };
                        let mut n = [0f32; 3];
                        n[axis] = dir as f32;
                        let (wf, hf) = (w as f32, h as f32);
                        // Counter-clockwise seen from outside.
                        let corners = if dir > 0 {
                            [
                                corner(0.0, 0.0),
                                corner(wf, 0.0),
                                corner(wf, hf),
                                corner(0.0, hf),
                            ]
                        } else {
                            [
                                corner(0.0, 0.0),
                                corner(0.0, hf),
                                corner(wf, hf),
                                corner(wf, 0.0),
                            ]
                        };
                        quads.push(Quad {
                            corners,
                            normal: n,
                            color: c,
                        });
                        i += w;
                    }
                    j += 1;
                }
            }
        }
    }
    quads
}
