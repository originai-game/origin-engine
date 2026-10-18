//! The cost numbers of tests/evidence/E6/nav-costs.md: 200 agents on a
//! 256 × 256 grid and on a baked navmesh, path queries, flow fields, the bake.
//!
//!     cargo run --release -p og-nav --example nav_costs > tests/evidence/E6/nav-costs.json

use og_core::glam::{Vec2, Vec3};
use og_ecs::{World, WorldDesc};
use og_nav::bake::{bake, box_triangles};
use og_nav::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn xorshift(s: &mut u32) -> u32 {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    *s
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    let mut out = serde_like::Obj::default();
    // ── grid ────────────────────────────────────────────────────────────
    let mut grid = Grid::new(256, 256, 1.0, Vec2::ZERO);
    let mut s = 99u32;
    for y in 0..256 {
        for x in 0..256 {
            if xorshift(&mut s) % 100 < 12 {
                grid.set(x, y, BLOCKED);
            }
        }
    }
    let mut pairs = Vec::new();
    let mut s = 7u32;
    while pairs.len() < 200 {
        let a = (xorshift(&mut s) % 256, xorshift(&mut s) % 256);
        let b = (xorshift(&mut s) % 256, xorshift(&mut s) % 256);
        if grid.walkable(a.0 as i32, a.1 as i32) && grid.walkable(b.0 as i32, b.1 as i32) {
            pairs.push((a, b));
        }
    }
    let mut scratch = Scratch::default();
    for (name, algo) in [("astar", Algorithm::AStar), ("jps", Algorithm::Jps)] {
        let t = Instant::now();
        let mut expanded = 0u64;
        let mut found = 0;
        for &(a, b) in &pairs {
            if let Some(p) = grid.find_path(a, b, algo, &mut scratch) {
                expanded += p.expanded as u64;
                found += 1;
            }
        }
        let total = ms(t);
        out.num(&format!("grid256.{name}.queries"), pairs.len() as f64);
        out.num(&format!("grid256.{name}.found"), found as f64);
        out.num(
            &format!("grid256.{name}.msPerQuery"),
            total / pairs.len() as f64,
        );
        out.num(
            &format!("grid256.{name}.expandedPerQuery"),
            expanded as f64 / pairs.len() as f64,
        );
    }
    let t = Instant::now();
    let field = FlowField::build(&grid, &[(250, 250)]);
    out.num("grid256.flowField.buildMs", ms(t));
    out.num("grid256.flowField.cellsReached", field.reached as f64);

    // 200 agents on the grid: planning tick, then the steady state.
    let agents_on = |nav: Nav,
                     plane: Plane,
                     spawn: &dyn Fn(&mut u32) -> (Vec3, Vec3),
                     flow: bool|
     -> (f64, f64, f64, usize) {
        let mut navs = Navs::default();
        let h = navs.insert(nav);
        let navs = Arc::new(Mutex::new(navs));
        let field_h = if flow {
            let g = match navs.lock().unwrap().get(h) {
                Some(Nav::Grid(g)) => g.clone(),
                _ => unreachable!(),
            };
            let f = FlowField::build(&g, &[(250, 250)]);
            navs.lock().unwrap().insert(Nav::Flow {
                grid: h,
                targets: vec![(250, 250)],
                field: f,
            })
        } else {
            h
        };
        let mut w = World::new(&WorldDesc {
            tick_rate: 60,
            seed: 1,
        })
        .unwrap();
        install(
            &mut w,
            navs,
            NavDesc {
                plane,
                raycast: None,
                plans_per_tick: og_nav::system::PLANS_PER_TICK,
            },
        )
        .unwrap();
        let mut s = 11u32;
        for _ in 0..200 {
            let (p, t) = spawn(&mut s);
            w.spawn((
                og_nav::system::Transform {
                    position: p,
                    ..Default::default()
                },
                NavAgent {
                    nav: field_h,
                    target: t,
                    seeking: true,
                    speed: 4.0,
                    acceleration: 40.0,
                    repath_ticks: 0,
                    ..Default::default()
                },
                Steering {
                    separation: 1.0,
                    ..Default::default()
                },
            ))
            .unwrap();
        }
        let t0 = Instant::now();
        w.step().unwrap();
        let first = ms(t0);
        let t1 = Instant::now();
        let n = 300;
        for _ in 0..n {
            w.step().unwrap();
        }
        let steady = ms(t1) / n as f64;
        let tq = Instant::now();
        let _ = w.world_hash();
        (first, steady, ms(tq), 200)
    };
    let g2 = grid.clone();
    let spawn_grid = move |s: &mut u32| loop {
        let (x, y) = (xorshift(s) % 256, xorshift(s) % 256);
        let (tx, ty) = (xorshift(s) % 256, xorshift(s) % 256);
        if g2.walkable(x as i32, y as i32) && g2.walkable(tx as i32, ty as i32) {
            return (
                Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0),
                Vec3::new(tx as f32 + 0.5, ty as f32 + 0.5, 0.0),
            );
        }
    };
    let (first, steady, _, _) = agents_on(Nav::Grid(grid.clone()), Plane::Xy, &spawn_grid, false);
    out.num("grid256.agents200.firstTickMs", first);
    out.num("grid256.agents200.tickMs", steady);
    let (first, steady, _, _) = agents_on(Nav::Grid(grid.clone()), Plane::Xy, &spawn_grid, true);
    out.num("grid256.flowAgents200.firstTickMs", first);
    out.num("grid256.flowAgents200.tickMs", steady);

    // ── navmesh ─────────────────────────────────────────────────────────
    // A 64 × 64 m arena: a floor, 120 pillars, four long walls, a platform with a ramp.
    let mut tris = box_triangles(Vec3::new(0.0, -0.1, 0.0), Vec3::new(32.0, 0.1, 32.0));
    let mut s = 3u32;
    for _ in 0..120 {
        let x = (xorshift(&mut s) % 6000) as f32 / 100.0 - 30.0;
        let z = (xorshift(&mut s) % 6000) as f32 / 100.0 - 30.0;
        tris.extend(box_triangles(
            Vec3::new(x, 1.0, z),
            Vec3::new(0.4, 1.0, 0.4),
        ));
    }
    for (x, z, hx, hz) in [
        (-10.0, 0.0, 0.3, 14.0),
        (10.0, 5.0, 0.3, 12.0),
        (0.0, -16.0, 12.0, 0.3),
        (0.0, 18.0, 9.0, 0.3),
    ] {
        tris.extend(box_triangles(Vec3::new(x, 1.5, z), Vec3::new(hx, 1.5, hz)));
    }
    tris.extend(box_triangles(
        Vec3::new(24.0, 0.5, 24.0),
        Vec3::new(4.0, 0.5, 4.0),
    ));
    let params = NavParams {
        cell_size: 0.25,
        cell_height: 0.1,
        agent_radius: 0.4,
        tile_size: 32,
        ..NavParams::default()
    };
    let t = Instant::now();
    let hf = bake(&tris, &params, None).unwrap();
    out.num("navmesh64.bake.ms", ms(t));
    out.num("navmesh64.bake.triangles", tris.len() as f64);
    out.num("navmesh64.bake.columns", (hf.width * hf.depth) as f64);
    let t = Instant::now();
    let mesh = NavMesh::build(hf.clone());
    out.num("navmesh64.build.ms", ms(t));
    out.num("navmesh64.build.tiles", mesh.tiles.len() as f64);
    out.num("navmesh64.build.polygons", mesh.poly_count() as f64);
    let mut s = 5u32;
    let mut queries = Vec::new();
    while queries.len() < 200 {
        let a = Vec3::new(
            (xorshift(&mut s) % 6000) as f32 / 100.0 - 30.0,
            0.0,
            (xorshift(&mut s) % 6000) as f32 / 100.0 - 30.0,
        );
        let b = Vec3::new(
            (xorshift(&mut s) % 6000) as f32 / 100.0 - 30.0,
            0.0,
            (xorshift(&mut s) % 6000) as f32 / 100.0 - 30.0,
        );
        if mesh.nearest(a, Vec3::new(0.5, 1.0, 0.5)).is_some()
            && mesh.nearest(b, Vec3::new(0.5, 1.0, 0.5)).is_some()
        {
            queries.push((a, b));
        }
    }
    let t = Instant::now();
    let mut expanded = 0u64;
    let mut corners = 0u64;
    for &(a, b) in &queries {
        let p = mesh.find_path(a, b, Vec3::new(1.0, 2.0, 1.0)).unwrap();
        expanded += p.expanded as u64;
        corners += p.points.len() as u64;
    }
    out.num("navmesh64.query.msPerQuery", ms(t) / queries.len() as f64);
    out.num(
        "navmesh64.query.polygonsExpanded",
        expanded as f64 / queries.len() as f64,
    );
    out.num(
        "navmesh64.query.cornersPerPath",
        corners as f64 / queries.len() as f64,
    );
    let mut m2 = mesh.clone();
    let t = Instant::now();
    let rebuilt = m2.set_obstacle(
        1,
        Some(Obstacle {
            min: Vec3::new(-2.0, 0.0, -2.0),
            max: Vec3::new(2.0, 2.0, 2.0),
        }),
    );
    out.num("navmesh64.obstacle.rebuildMs", ms(t));
    out.num("navmesh64.obstacle.tilesRebuilt", rebuilt.len() as f64);
    let q2 = queries.clone();
    let k = std::cell::Cell::new(0usize);
    let spawn_mesh = move |_s: &mut u32| {
        let i = k.get();
        k.set(i + 1);
        q2[i % q2.len()]
    };
    let (first, steady, _, _) = agents_on(Nav::Mesh(Box::new(mesh)), Plane::Xz, &spawn_mesh, false);
    out.num("navmesh64.agents200.firstTickMs", first);
    out.num("navmesh64.agents200.tickMs", steady);
    println!("{}", out.finish());
}

/// A tiny ordered JSON object writer (the example keeps og-nav free of serde).
mod serde_like {
    #[derive(Default)]
    pub struct Obj(Vec<(String, f64)>);
    impl Obj {
        pub fn num(&mut self, k: &str, v: f64) {
            self.0.push((k.to_string(), v));
        }
        pub fn finish(&self) -> String {
            let body: Vec<String> = self
                .0
                .iter()
                .map(|(k, v)| format!("  \"{k}\": {}", (v * 1000.0).round() / 1000.0))
                .collect();
            format!("{{\n{}\n}}", body.join(",\n"))
        }
    }
}
