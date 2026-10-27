//! og-nav through its public API: grids (A*, jump point search, smoothing),
//! flow fields, baked navmeshes (paths, funnel, obstacles that rebuild their
//! tiles, walking the surface) and the og.nav agent system in a world
//! (crowds, separation, determinism and snapshots).
//!
//! Ways it could fail, each asserted below: JPS answering a longer path than
//! A* or a move that cuts a blocked corner; smoothing through a wall; a flow
//! direction that does not lead downhill to a target; a navmesh path through
//! a wall, off the surface or missing a ramp to a platform; an obstacle that
//! rebuilds the whole mesh, or leaves it changed after removal; an agent
//! that never arrives, walks into a wall, piles onto another, or replays to
//! a different world after a snapshot; an agent that has never planned (or
//! was teleported) walked from a polygon it does not stand on; bounds that
//! leave polygons out.

use og_core::glam::{Vec2, Vec3};
use og_ecs::{World, WorldDesc};
use og_nav::bake::{bake, box_triangles};
use og_nav::{
    Algorithm, BLOCKED, FlowField, Grid, Nav, NavAgent, NavDesc, NavMesh, NavParams, Navs,
    Obstacle, Plane, STATUS_ARRIVED, Scratch, Steering, install,
};
use std::sync::{Arc, Mutex};

fn xorshift(s: &mut u32) -> u32 {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    *s
}

fn maze(w: u32, h: u32, seed: u32, density: u32) -> Grid {
    let mut g = Grid::new(w, h, 1.0, Vec2::ZERO);
    let mut s = seed.max(1);
    for y in 0..h {
        for x in 0..w {
            if xorshift(&mut s) % 100 < density {
                g.set(x, y, BLOCKED);
            }
        }
    }
    g.set(0, 0, 1);
    g.set(w - 1, h - 1, 1);
    g
}

#[test]
fn jump_point_search_finds_paths_as_short_as_astar_and_never_cuts_a_corner() {
    let mut scratch = Scratch::default();
    let mut found = 0;
    for seed in 1..80u32 {
        let g = maze(48, 40, seed * 7919, 28);
        let a = g.find_path((0, 0), (47, 39), Algorithm::AStar, &mut scratch);
        let j = g.find_path((0, 0), (47, 39), Algorithm::Jps, &mut scratch);
        assert_eq!(
            a.as_ref().map(|p| p.cost),
            j.as_ref().map(|p| p.cost),
            "seed {seed}: A* and JPS disagree on the cost"
        );
        let (Some(a), Some(j)) = (a, j) else { continue };
        found += 1;
        assert_eq!(j.algorithm, Algorithm::Jps);
        assert!(
            j.expanded < a.expanded,
            "seed {seed}: JPS expanded {} nodes, A* {}",
            j.expanded,
            a.expanded
        );
        for p in [&a, &j] {
            assert_eq!(p.cells.first(), Some(&(0, 0)));
            assert_eq!(p.cells.last(), Some(&(47, 39)));
            for w in p.cells.windows(2) {
                let (dx, dy) = (w[1].0 as i32 - w[0].0 as i32, w[1].1 as i32 - w[0].1 as i32);
                assert!(
                    dx.abs() <= 1 && dy.abs() <= 1 && (dx, dy) != (0, 0),
                    "seed {seed}: a jump of {dx},{dy}"
                );
                assert!(
                    g.can_step(w[0].0 as i32, w[0].1 as i32, dx, dy),
                    "seed {seed}: the step {w:?} cuts a blocked corner or enters a wall"
                );
            }
        }
    }
    assert!(
        found > 30,
        "only {found} of the random mazes were solvable; the test needs more"
    );
    // Two diagonal walls touching at a corner: no squeezing between them.
    let mut g = Grid::new(2, 2, 1.0, Vec2::ZERO);
    g.set(1, 0, BLOCKED);
    g.set(0, 1, BLOCKED);
    assert!(
        g.find_path((0, 0), (1, 1), Algorithm::AStar, &mut scratch)
            .is_none()
    );
    assert!(
        g.find_path((0, 0), (1, 1), Algorithm::Jps, &mut scratch)
            .is_none()
    );
}

#[test]
fn weighted_cells_are_avoided_when_a_detour_is_cheaper_and_jps_falls_back_to_astar() {
    let mut g = Grid::new(21, 9, 1.0, Vec2::ZERO);
    // A swamp across the middle row band, open at the top row only.
    for x in 10..11 {
        for y in 0..8 {
            g.set(x, y, 50);
        }
    }
    let mut scratch = Scratch::default();
    let p = g
        .find_path((0, 4), (20, 4), Algorithm::Jps, &mut scratch)
        .expect("a path");
    assert_eq!(p.algorithm, Algorithm::AStar, "weighted grids run A*");
    assert!(
        p.cells.iter().all(|&(x, y)| x != 10 || y == 8),
        "the path wades the swamp: {:?}",
        p.cells
    );
    // When the detour is longer than the swamp's cost, it wades through.
    for x in 10..11 {
        g.set(x, 8, 50);
    }
    let p = g
        .find_path((0, 4), (20, 4), Algorithm::AStar, &mut scratch)
        .expect("a path");
    assert!(
        p.cells.contains(&(10, 4)),
        "with no way round, the cheapest path crosses: {:?}",
        p.cells
    );
}

#[test]
fn smoothed_grid_paths_keep_line_of_sight_and_open_rooms_are_one_segment() {
    let mut g = Grid::new(30, 30, 0.5, Vec2::new(-5.0, -5.0));
    let mut scratch = Scratch::default();
    let (a, b) = (Vec2::new(-4.6, -4.4), Vec2::new(9.6, 9.3));
    let p = g
        .find_path(
            g.cell_at(a).unwrap(),
            g.cell_at(b).unwrap(),
            Algorithm::Jps,
            &mut scratch,
        )
        .unwrap();
    let pts = g.waypoints(&p, a, b, true, 0.0);
    assert_eq!(pts, vec![a, b], "an empty room needs no corners");
    // A wall with a gap at its top end.
    for y in 0..24 {
        g.set(15, y, BLOCKED);
    }
    let p = g
        .find_path(
            g.cell_at(a).unwrap(),
            g.cell_at(b).unwrap(),
            Algorithm::Jps,
            &mut scratch,
        )
        .unwrap();
    let pts = g.waypoints(&p, a, b, true, 0.3);
    assert!(pts.len() >= 3, "the wall needs a corner: {pts:?}");
    for w in pts.windows(2) {
        assert!(
            g.line_of_sight(w[0], w[1], 0.0),
            "the segment {w:?} crosses the wall"
        );
        // Sample the segment: never inside a blocked cell.
        for k in 0..=50 {
            let q = w[0].lerp(w[1], k as f32 / 50.0);
            let (x, y) = g.cell_at(q).unwrap();
            assert!(g.walkable(x as i32, y as i32), "{q} is in a wall");
        }
    }
    let unsmoothed = g.waypoints(&p, a, b, false, 0.0);
    assert!(unsmoothed.len() > pts.len(), "smoothing removes corners");
}

#[test]
fn a_flow_field_leads_every_reachable_cell_downhill_to_a_target() {
    let g = maze(64, 64, 4242, 22);
    let f = FlowField::build(&g, &[(63, 63), (0, 0)]);
    let mut reachable = 0;
    for y in 0..64u32 {
        for x in 0..64u32 {
            let i = (y * 64 + x) as usize;
            if f.cost[i] == u64::MAX {
                continue;
            }
            reachable += 1;
            // Follow the directions: costs fall every step and end at 0.
            let (mut cx, mut cy) = (x as i32, y as i32);
            let mut steps = 0;
            loop {
                let k = (cy * 64 + cx) as usize;
                if f.cost[k] == 0 {
                    break;
                }
                let d = f.dir[k];
                assert_ne!(
                    d,
                    og_nav::flow::NONE,
                    "cell {cx},{cy} has a cost but no direction"
                );
                let (dx, dy) = og_nav::grid::DIRS[d as usize];
                assert!(
                    g.can_step(cx, cy, dx, dy),
                    "the direction at {cx},{cy} cuts a corner"
                );
                let n = ((cy + dy) * 64 + cx + dx) as usize;
                assert!(
                    f.cost[n] < f.cost[k],
                    "the direction at {cx},{cy} goes uphill"
                );
                cx += dx;
                cy += dy;
                steps += 1;
                assert!(steps < 64 * 64, "a loop at {x},{y}");
            }
        }
    }
    assert_eq!(f.reached, reachable);
    assert!(reachable > 64 * 64 / 2);
    // Sampling between cells blends, and is a unit vector or zero.
    let s = f.sample(Vec2::new(30.3, 30.7));
    assert!(s == Vec2::ZERO || (s.length() - 1.0).abs() < 1e-4);
}

fn params() -> NavParams {
    NavParams {
        cell_size: 0.25,
        cell_height: 0.1,
        agent_radius: 0.3,
        agent_height: 1.8,
        agent_climb: 0.4,
        max_slope: 45.0,
        tile_size: 16,
        max_error: 1.3,
    }
}

/// A 20 × 20 m floor with a wall across the middle (a gap at +z) and a 1 m
/// platform in a corner with a ramp up to it.
fn level() -> Vec<[Vec3; 3]> {
    let mut t = box_triangles(Vec3::new(0.0, -0.1, 0.0), Vec3::new(10.0, 0.1, 10.0));
    t.extend(box_triangles(
        Vec3::new(0.0, 1.0, -2.0),
        Vec3::new(0.25, 1.0, 8.0),
    ));
    t.extend(box_triangles(
        Vec3::new(7.0, 0.5, -7.0),
        Vec3::new(2.0, 0.5, 2.0),
    ));
    // The ramp: from (3, 0, -7) up to (5, 1, -7), 2 m wide.
    let (a, b, c, d) = (
        Vec3::new(3.0, 0.0, -8.0),
        Vec3::new(3.0, 0.0, -6.0),
        Vec3::new(5.0, 1.0, -6.0),
        Vec3::new(5.0, 1.0, -8.0),
    );
    t.push([a, b, c]);
    t.push([a, c, d]);
    t
}

fn on_mesh(m: &NavMesh, p: Vec3) -> bool {
    m.nearest(p, Vec3::new(0.05, 0.3, 0.05)).is_some()
}

#[test]
fn a_baked_navmesh_routes_around_walls_up_ramps_and_stays_on_the_surface() {
    let field = bake(&level(), &params(), None).expect("bake");
    let mesh = NavMesh::build(field.clone());
    assert!(mesh.poly_count() > 4, "{} polygons", mesh.poly_count());
    // Every polygon is convex and counter-clockwise seen from +y.
    for tile in &mesh.tiles {
        for p in &tile.polys {
            let n = p.verts.len();
            assert!(n >= 3);
            for i in 0..n {
                let (a, b, c) = (
                    tile.verts[p.verts[i] as usize],
                    tile.verts[p.verts[(i + 1) % n] as usize],
                    tile.verts[p.verts[(i + 2) % n] as usize],
                );
                let cross = (b.x - a.x) * (c.z - b.z) - (b.z - a.z) * (c.x - b.x);
                assert!(cross >= -1e-4, "a reflex vertex");
            }
        }
    }
    // Across the wall: the path goes through the gap at +z.
    let (from, to) = (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.0, 0.0));
    let p = mesh
        .find_path(from, to, Vec3::new(1.0, 2.0, 1.0))
        .expect("a path");
    assert!(p.complete, "the far side is reachable");
    assert!(
        p.points.len() >= 3,
        "the wall needs a corner: {:?}",
        p.points
    );
    let length: f32 = p.points.windows(2).map(|w| w[0].distance(w[1])).sum();
    let straight = from.distance(to);
    assert!(
        length > straight * 1.05 && length < straight * 2.2,
        "length {length} against {straight}"
    );
    for w in p.points.windows(2) {
        for k in 0..=40 {
            let q = w[0].lerp(w[1], k as f32 / 40.0);
            assert!(
                !(q.x.abs() < 0.25 + 0.2 && q.z < 6.0 - 0.2),
                "{q} is inside the wall"
            );
            assert!(on_mesh(&mesh, q), "{q} is off the navmesh");
        }
    }
    assert!(
        p.points.iter().any(|q| q.z > 5.5),
        "the path uses the gap: {:?}",
        p.points
    );
    // Onto the platform: only by the ramp.
    let top = Vec3::new(7.5, 1.0, -7.5);
    let p = mesh
        .find_path(Vec3::new(-6.0, 0.0, -9.0), top, Vec3::new(1.0, 2.0, 1.0))
        .expect("a path");
    assert!(p.complete);
    let end = *p.points.last().unwrap();
    assert!(
        (end.y - 1.0).abs() < 0.15,
        "the path ends on the platform, at y {}",
        end.y
    );
    let up = p.points.windows(2).any(|w| {
        w[0].x <= 5.2
            && w[1].x >= 4.8
            && (w[0].z + w[1].z) * 0.5 > -8.5
            && (w[0].z + w[1].z) * 0.5 < -5.5
    });
    assert!(up, "the path climbs the ramp: {:?}", p.points);
    // The same heightfield builds the same tiles.
    assert_eq!(NavMesh::build(field).tiles, mesh.tiles);
}

#[test]
fn geometry_wholly_above_the_bounds_is_left_out_of_the_bake() {
    // A floor, and a slab floating above the box the bake is bounded by
    // (a terrain's peaks above a navmesh's bounds): the slab is outside the
    // box, so the floor under it stays walkable and nothing else changes.
    let mut level = box_triangles(Vec3::new(0.0, -0.1, 0.0), Vec3::new(10.0, 0.1, 10.0));
    level.extend(box_triangles(
        Vec3::new(4.0, 6.0, 4.0),
        Vec3::new(2.0, 0.5, 2.0),
    ));
    let bounds = Some((Vec3::new(-10.0, -0.5, -10.0), Vec3::new(10.0, 1.0, 10.0)));
    let mesh = NavMesh::build(bake(&level, &params(), bounds).expect("bake"));
    assert!(
        on_mesh(&mesh, Vec3::new(4.0, 0.0, 4.0)),
        "the floor under the slab"
    );
    let floor = NavMesh::build(bake(&level[..12], &params(), bounds).expect("bake"));
    assert_eq!(mesh.tiles, floor.tiles);
}

#[test]
fn an_obstacle_rebuilds_only_its_tiles_and_removing_it_restores_them() {
    let field = bake(&level(), &params(), None).expect("bake");
    let mut mesh = NavMesh::build(field);
    let before = mesh.tiles.clone();
    let total = mesh.tiles.len();
    // Close the gap in the wall.
    let rebuilt = mesh.set_obstacle(
        7,
        Some(Obstacle {
            min: Vec3::new(-0.6, 0.0, 5.8),
            max: Vec3::new(0.6, 2.0, 10.2),
        }),
    );
    // The box lies in tile columns 2 (x -2..2) of rows 3–4 (z 2..10): those
    // and their four neighbours, nothing else.
    let under = |t: usize| -> bool {
        let (tx, tz) = (
            (t as u32 % mesh.tiles_x) as i32,
            (t as u32 / mesh.tiles_x) as i32,
        );
        (1..=3).contains(&tx) && (2..=4).contains(&tz)
    };
    assert!(
        !rebuilt.is_empty() && rebuilt.len() < total / 2,
        "rebuilt {rebuilt:?} of {total} tiles"
    );
    assert!(
        rebuilt.iter().all(|&t| under(t)),
        "rebuilt {rebuilt:?}: a tile far from the box"
    );
    let (from, to) = (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.0, 0.0));
    let p = mesh
        .find_path(from, to, Vec3::new(1.0, 2.0, 1.0))
        .expect("a partial path");
    assert!(
        !p.complete,
        "with the gap closed the far side is unreachable"
    );
    assert!(
        p.points.last().unwrap().x < 0.0,
        "the partial path stops on the near side"
    );
    // Tiles not rebuilt keep their polygons (their links into rebuilt tiles may move).
    let shape = |t: &og_nav::Tile| {
        (
            t.lattice.clone(),
            t.polys.iter().map(|p| p.verts.clone()).collect::<Vec<_>>(),
        )
    };
    for (t, tile) in mesh.tiles.iter().enumerate() {
        if !rebuilt.contains(&t) {
            assert_eq!(
                shape(tile),
                shape(&before[t]),
                "tile {t} changed though it was not rebuilt"
            );
        }
    }
    mesh.set_obstacle(7, None);
    assert_eq!(
        mesh.tiles, before,
        "removing the obstacle restores the mesh"
    );
    assert!(
        mesh.find_path(from, to, Vec3::new(1.0, 2.0, 1.0))
            .unwrap()
            .complete
    );
}

#[test]
fn moving_along_the_surface_slides_on_walls_and_never_leaves_the_mesh() {
    let mesh = NavMesh::build(bake(&level(), &params(), None).unwrap());
    let start = Vec3::new(-2.0, 0.0, -2.0);
    let (poly, _) = mesh.nearest(start, Vec3::new(1.0, 1.0, 1.0)).unwrap();
    // Straight into the wall at x = 0, diagonally: the slide keeps the z motion.
    let (_, at) = mesh.move_along(poly, start, Vec3::new(2.0, 0.0, -1.0));
    assert!(at.x < -0.25, "stopped before the wall, at {at}");
    assert!(at.z > -1.9, "slid along it, at {at}");
    assert!(on_mesh(&mesh, at));
}

fn world_with(navs: Arc<Mutex<Navs>>, plane: Plane) -> World {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 3,
    })
    .unwrap();
    install(
        &mut w,
        navs,
        NavDesc {
            plane,
            raycast: None,
            plans_per_tick: og_nav::PLANS_PER_TICK,
        },
    )
    .unwrap();
    w
}

/// 200 agents on a 256 × 256 grid with scattered walls, to opposite corners.
fn crowd(seed_agents: u32) -> (World, Vec<og_ecs::Entity>) {
    let mut navs = Navs::default();
    let grid = maze(256, 256, 99, 12);
    let h = navs.insert(Nav::Grid(grid.clone()));
    let navs = Arc::new(Mutex::new(navs));
    let mut w = world_with(navs, Plane::Xy);
    let mut s = seed_agents;
    let mut agents = Vec::new();
    while agents.len() < 200 {
        let (x, y) = (xorshift(&mut s) % 60, xorshift(&mut s) % 60);
        if !grid.walkable(x as i32, y as i32) {
            continue;
        }
        let (tx, ty) = (196 + xorshift(&mut s) % 58, 196 + xorshift(&mut s) % 58);
        if !grid.walkable(tx as i32, ty as i32) {
            continue;
        }
        let e = w
            .spawn((
                og_nav::system::Transform {
                    position: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0),
                    ..Default::default()
                },
                NavAgent {
                    nav: h,
                    target: Vec3::new(tx as f32 + 0.5, ty as f32 + 0.5, 0.0),
                    seeking: true,
                    speed: 12.0,
                    acceleration: 80.0,
                    repath_ticks: 0,
                    stop_distance: 0.3,
                    ..Default::default()
                },
            ))
            .unwrap();
        agents.push(e);
    }
    (w, agents)
}

#[test]
fn two_hundred_agents_cross_a_256_grid_and_replay_identically_after_a_snapshot() {
    let (mut a, agents) = crowd(5);
    let (mut b, _) = crowd(5);
    let mut snap = None;
    for t in 0..2400 {
        a.step().unwrap();
        b.step().unwrap();
        if t == 300 {
            snap = Some(a.snapshot());
            assert_eq!(
                a.world_hash(),
                b.world_hash(),
                "two runs differ at tick 300"
            );
        }
    }
    assert_eq!(a.world_hash(), b.world_hash(), "two runs differ at the end");
    let arrived = agents
        .iter()
        .filter(|&&e| a.get::<NavAgent>(e).unwrap().status == STATUS_ARRIVED)
        .count();
    assert!(
        arrived >= 190,
        "only {arrived} of 200 agents arrived in 40 s"
    );
    for &e in &agents {
        let p = a.get::<og_nav::system::Transform>(e).unwrap().position;
        assert!(
            p.x >= 0.0 && p.y >= 0.0 && p.x < 256.0 && p.y < 256.0,
            "an agent left the grid: {p}"
        );
    }
    // A fresh world restored at tick 301 ends where the first one did.
    let (mut c, _) = crowd(5);
    c.restore(&snap.unwrap()).unwrap();
    for _ in 301..2400 {
        c.step().unwrap();
    }
    assert_eq!(
        c.world_hash(),
        a.world_hash(),
        "the restored world diverged"
    );
}

#[test]
fn separation_spreads_a_pile_of_agents_and_wander_is_deterministic() {
    let run = || {
        let navs = Arc::new(Mutex::new(Navs::default()));
        let mut w = world_with(navs, Plane::Xz);
        let mut es = Vec::new();
        for i in 0..20 {
            let e = w
                .spawn((
                    og_nav::system::Transform {
                        position: Vec3::new(0.001 * i as f32, 0.0, 0.0),
                        ..Default::default()
                    },
                    NavAgent {
                        speed: 2.0,
                        radius: 0.3,
                        ..Default::default()
                    },
                    Steering {
                        seek: 0.0,
                        separation: 2.0,
                        separation_radius: 1.2,
                        wander: if i == 0 { 1.0 } else { 0.0 },
                        ..Default::default()
                    },
                ))
                .unwrap();
            es.push(e);
        }
        for _ in 0..180 {
            w.step().unwrap();
        }
        let ps: Vec<Vec3> = es
            .iter()
            .map(|&e| w.get::<og_nav::system::Transform>(e).unwrap().position)
            .collect();
        (w.world_hash(), ps)
    };
    let (h1, ps) = run();
    let (h2, _) = run();
    assert_eq!(h1, h2, "two runs differ");
    let mut min = f32::MAX;
    for i in 0..ps.len() {
        for j in i + 1..ps.len() {
            min = min.min(ps[i].distance(ps[j]));
        }
        assert_eq!(ps[i].y, 0.0, "3D agents keep their height");
    }
    assert!(
        min > 0.3,
        "agents still overlap: the closest pair is {min} m apart"
    );
}

#[test]
fn an_agent_walks_a_navmesh_around_the_wall_and_arrives_on_the_surface() {
    let mut navs = Navs::default();
    let mesh = NavMesh::build(bake(&level(), &params(), None).unwrap());
    let h = navs.insert(Nav::Mesh(Box::new(mesh.clone())));
    let mut w = world_with(Arc::new(Mutex::new(navs)), Plane::Xz);
    let target = Vec3::new(6.0, 0.0, 0.0);
    let e = w
        .spawn((
            og_nav::system::Transform {
                position: Vec3::new(-6.0, 0.0, -6.0),
                ..Default::default()
            },
            NavAgent {
                nav: h,
                target,
                seeking: true,
                speed: 4.0,
                face_movement: true,
                ..Default::default()
            },
        ))
        .unwrap();
    let mut max_z: f32 = -10.0;
    for _ in 0..900 {
        w.step().unwrap();
        let p = w.get::<og_nav::system::Transform>(e).unwrap().position;
        max_z = max_z.max(p.z);
        assert!(
            !(p.x.abs() < 0.25 && p.z < 6.0),
            "the agent is inside the wall at {p}"
        );
        assert!(on_mesh(&mesh, p), "the agent left the surface at {p}");
    }
    let a = w.get::<NavAgent>(e).unwrap();
    let p = w.get::<og_nav::system::Transform>(e).unwrap();
    assert_eq!(
        a.status, STATUS_ARRIVED,
        "status {} at {}",
        a.status, p.position
    );
    assert!(p.position.distance(target) < 0.3);
    assert!(max_z > 5.0, "it went round through the gap (max z {max_z})");
    assert!(p.quaternion.length() > 0.99, "a unit rotation");
}

/// A court at y 0 (x from -3), a terrace at y 4 and, over the heightfield's
/// first tile (so navmesh polygon 0), a high terrace at y 8.
fn terraces() -> NavMesh {
    let mut t = box_triangles(Vec3::new(3.5, -0.1, 0.0), Vec3::new(6.5, 0.1, 10.0));
    t.extend(box_triangles(
        Vec3::new(-6.5, 4.0, -6.5),
        Vec3::new(3.5, 4.0, 3.5),
    ));
    t.extend(box_triangles(
        Vec3::new(6.0, 2.0, 6.0),
        Vec3::new(3.0, 2.0, 3.0),
    ));
    NavMesh::build(bake(&t, &params(), None).expect("bake"))
}

#[test]
fn an_idle_agent_stays_on_its_own_terrace_and_later_reaches_a_target() {
    // Kestrel Stair FINDINGS #7: an agent that had never planned was walked
    // from polygon 0 (the high terrace here) and lifted to its height.
    let mesh = terraces();
    let (tile, first) = mesh.poly(0).expect("polygon 0");
    assert!(
        first
            .verts
            .iter()
            .all(|&v| (tile.verts[v as usize].y - 8.0).abs() < 0.2),
        "polygon 0 is on the high terrace"
    );
    let mut navs = Navs::default();
    let h = navs.insert(Nav::Mesh(Box::new(mesh.clone())));
    let mut w = world_with(Arc::new(Mutex::new(navs)), Plane::Xz);
    let agent = |at: Vec3| {
        (
            og_nav::system::Transform {
                position: at,
                ..Default::default()
            },
            NavAgent {
                nav: h,
                target: at,
                speed: 4.0,
                ..Default::default()
            },
        )
    };
    let (court_at, terrace_at) = (Vec3::new(0.0, 0.0, 5.0), Vec3::new(5.0, 4.0, 5.0));
    let court = w.spawn(agent(court_at)).unwrap();
    let terrace = w.spawn(agent(terrace_at)).unwrap();
    let at = |w: &World, e| w.get::<og_nav::system::Transform>(e).unwrap().position;
    for _ in 0..30 {
        w.step().unwrap();
        assert_eq!(at(&w, court), court_at, "the idle agent on the court stays");
        assert_eq!(
            at(&w, terrace),
            terrace_at,
            "the idle agent on the terrace stays"
        );
    }
    assert_eq!(
        w.get::<NavAgent>(court).unwrap().status,
        og_nav::STATUS_IDLE
    );
    // Later, each walks to a point on its own level and arrives there.
    let goals = [
        (court, Vec3::new(5.0, 0.0, -5.0), 0.0),
        (terrace, Vec3::new(7.5, 4.0, 7.5), 4.0),
    ];
    for (e, target, _) in goals {
        let mut a = w.get::<NavAgent>(e).unwrap();
        a.target = target;
        a.seeking = true;
        w.insert(e, a).unwrap();
    }
    for _ in 0..600 {
        w.step().unwrap();
        for (e, _, y) in goals {
            let p = at(&w, e);
            assert!((p.y - y).abs() < 0.2, "the agent left its level: {p}");
        }
    }
    for (e, target, _) in goals {
        let a = w.get::<NavAgent>(e).unwrap();
        assert_eq!(a.status, STATUS_ARRIVED, "at {}", at(&w, e));
        assert!(at(&w, e).distance(target) < 0.3);
    }
    // Teleported while idle, then sent a step away: it walks from where it
    // now stands, not from the polygon it left.
    let mut a = w.get::<NavAgent>(court).unwrap();
    a.seeking = false;
    w.insert(court, a).unwrap();
    w.step().unwrap();
    let mut t = w.get::<og_nav::system::Transform>(court).unwrap();
    t.position = Vec3::new(-2.0, 0.0, 8.0);
    w.insert(court, t).unwrap();
    a = w.get::<NavAgent>(court).unwrap();
    a.target = Vec3::new(-1.0, 0.0, 8.0);
    a.seeking = true;
    w.insert(court, a).unwrap();
    for _ in 0..120 {
        w.step().unwrap();
        let p = at(&w, court);
        assert!(p.y.abs() < 0.2 && p.z > 7.0, "the agent jumped to {p}");
    }
    assert_eq!(w.get::<NavAgent>(court).unwrap().status, STATUS_ARRIVED);
}

#[test]
fn a_navmesh_s_bounds_hold_every_polygon_up_to_the_high_terrace() {
    // Kestrel Stair FINDINGS #9: the bounds' top was the floor's origin plus
    // the agent's height, whatever the polygons' heights.
    let mesh = terraces();
    let (lo, hi) = mesh.bounds();
    assert!(
        (hi.y - 8.0).abs() < 0.2,
        "the top is the high terrace: {hi}"
    );
    assert!(lo.y.abs() < 0.2, "the bottom is the court: {lo}");
    for tile in &mesh.tiles {
        for p in &tile.polys {
            for &v in &p.verts {
                let v = tile.verts[v as usize];
                assert!(
                    v.cmpge(lo).all() && v.cmple(hi).all(),
                    "{v} is outside {lo}..{hi}"
                );
            }
        }
    }
    assert!(lo.x < -9.0 && hi.x > 9.0 && lo.z < -9.0 && hi.z > 9.0);
}

/// The heading of a rotation about +Z (the Xy plane's facing), in (-π, π].
fn heading_z(q: og_core::glam::Quat) -> f32 {
    let mut h = 2.0 * og_core::math::atan2(q.z, q.w);
    while h > std::f32::consts::PI {
        h -= std::f32::consts::TAU;
    }
    while h <= -std::f32::consts::PI {
        h += std::f32::consts::TAU;
    }
    h
}

/// A facing agent turns the short way, through ±π, by at most its turn
/// speed a tick, until it faces where it walks: from a heading of 3.0 rad
/// towards one of -3.0, the short turn is +0.28 rad across π, not -6.0.
#[test]
fn a_facing_agent_turns_the_short_way_at_its_turn_speed() {
    let mut navs = Navs::default();
    let h = navs.insert(Nav::Grid(Grid::new(32, 32, 1.0, Vec2::ZERO)));
    let mut w = world_with(Arc::new(Mutex::new(navs)), Plane::Xy);
    let start = Vec3::new(20.5, 16.5, 0.0);
    let target = start + 10.0 * Vec3::new(og_core::math::cos(-3.0), og_core::math::sin(-3.0), 0.0);
    let e = w
        .spawn((
            og_nav::system::Transform {
                position: start,
                quaternion: og_core::math::quat_from_axis_angle(Vec3::Z, 3.0),
                ..Default::default()
            },
            NavAgent {
                nav: h,
                target,
                seeking: true,
                face_movement: true,
                turn_speed: 8.0,
                ..Default::default()
            },
        ))
        .unwrap();
    let max = 8.0 / 60.0;
    let mut was = 3.0f32;
    let mut first = None;
    for _ in 0..30 {
        w.step().unwrap();
        let now = heading_z(w.get::<og_nav::system::Transform>(e).unwrap().quaternion);
        let mut turned = now - was;
        if turned > std::f32::consts::PI {
            turned -= std::f32::consts::TAU;
        } else if turned < -std::f32::consts::PI {
            turned += std::f32::consts::TAU;
        }
        assert!(
            turned.abs() <= max + 1e-4,
            "turned {turned} in a tick, more than {max}"
        );
        first.get_or_insert(turned);
        was = now;
    }
    assert!(
        first.unwrap() > 0.1,
        "the first tick turned {:?}: the long way",
        first
    );
    let v = w.get::<NavAgent>(e).unwrap().velocity;
    let facing = og_core::math::atan2(v.y, v.x);
    assert!(
        (heading_z(og_core::math::quat_from_axis_angle(Vec3::Z, facing)) - was).abs() < 1e-3,
        "faces {was}, walks {facing}"
    );
}
