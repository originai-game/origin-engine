//! Scratch bench: og-render's CPU frustum cull loop (lib.rs `cull`), serial
//! against og-lanes at several grains. The loop body is a copy of the
//! engine's: mark the included bounds, then `for_each_indexed` over the
//! marks with the frustum's sphere test.
use glam::{Mat4, Vec3, Vec4};
use std::time::Instant;

fn frustum_planes(m: Mat4) -> [Vec4; 6] {
    let (r0, r1, r2, r3) = (m.row(0), m.row(1), m.row(2), m.row(3));
    [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2].map(|p| p / p.truncate().length().max(1e-20))
}

struct Frustum(Vec<Vec4>);
impl Frustum {
    #[inline]
    fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.0.iter().all(|p| p.truncate().dot(center) + p.w >= -radius)
    }
}

fn cull(inside: &mut Vec<bool>, spheres: &[(Vec3, f32)], frustum: &Frustum, grain: usize) {
    inside.clear();
    inside.resize(spheres.len(), false);
    inside.fill(true);
    og_lanes::for_each_indexed(og_lanes::Site::Cull, inside, grain, |k, v| {
        if *v {
            let (c, r) = spheres[k];
            *v = frustum.intersects_sphere(c, r);
        }
    });
}

fn spin(us: u64) {
    let t = Instant::now();
    let mut x = 0u64;
    while t.elapsed().as_micros() < us as u128 {
        x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
    }
}

fn stats(v: &mut [f64]) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.total_cmp(b));
    let p = |q: f64| v[((v.len() - 1) as f64 * q) as usize];
    (p(0.5), p(0.9), v.iter().sum::<f64>() / v.len() as f64)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iters: usize = args.get(1).map_or(400, |s| s.parse().unwrap());
    let gap: u64 = args.get(2).map_or(300, |s| s.parse().unwrap());
    let ns: Vec<usize> = args
        .get(3)
        .map_or("1024,2048,8192,32768,100000".into(), |s| s.clone())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let threads = og_lanes::threads() as usize;
    let view = glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 30.0, -10.0), Vec3::new(0.0, 0.0, 80.0), Vec3::Y);
    let proj = glam::camera::rh::proj::directx::perspective(1.0, 16.0 / 9.0, 0.1, 400.0);
    let frustum = Frustum(frustum_planes(proj * view).to_vec());
    println!("threads={threads} iters={iters} gap_us={gap}");
    for &n in &ns {
        let side = (n as f32).sqrt().ceil() as usize;
        let spheres: Vec<(Vec3, f32)> = (0..n)
            .map(|i| {
                let (x, z) = ((i % side) as f32, (i / side) as f32);
                let s = 400.0 / side as f32;
                (Vec3::new((x - side as f32 / 2.0) * s, 2.0, z * s - 100.0), 1.5)
            })
            .collect();
        let per = n.div_ceil(threads.max(1));
        let variants: Vec<(&str, Option<usize>)> = vec![
            ("serial", None),
            ("g1024", Some(1024)),
            ("g4096", Some(4096)),
            ("g16384", Some(16384)),
            ("g32768", Some(32768)),
            ("g65536", Some(65536)),
            ("perthread", Some(per.max(1))),
        ];
        let mut times: Vec<Vec<f64>> = vec![Vec::new(); variants.len()];
        let mut inside = Vec::new();
        let mut reference = Vec::new();
        for it in 0..iters + 20 {
            for (vi, (_, g)) in variants.iter().enumerate() {
                spin(gap);
                og_lanes::set_enabled(g.is_some());
                let t = Instant::now();
                cull(&mut inside, &spheres, &frustum, g.unwrap_or(1024));
                let us = t.elapsed().as_secs_f64() * 1e6;
                if vi == 0 {
                    reference = inside.clone();
                } else {
                    assert_eq!(inside, reference);
                }
                if it >= 20 {
                    times[vi].push(us);
                }
            }
        }
        let visible = reference.iter().filter(|v| **v).count();
        let mut line = format!("n={n} visible={visible}");
        for (vi, (name, _)) in variants.iter().enumerate() {
            let (p50, p90, mean) = stats(&mut times[vi]);
            line += &format!(" | {name} p50={p50:.1} p90={p90:.1} mean={mean:.1}");
        }
        println!("{line}");
    }
}
