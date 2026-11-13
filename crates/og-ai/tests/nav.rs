//! og-ai's navigation through its public API: grids (A*, jump point search, smoothing),
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

use og_ai::nav::bake::{bake, box_triangles};
use og_ai::nav::{
    BLOCKED, Grid, Nav, NavAgent, NavDesc, NavMesh, NavParams, Navs, Obstacle, Plane,
    STATUS_ARRIVED, Steering, install,
};
use og_core::glam::{Vec2, Vec3};
use og_ecs::{World, WorldDesc};
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
    let shape = |t: &og_ai::nav::Tile| {
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
            plans_per_tick: og_ai::nav::PLANS_PER_TICK,
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
                og_ai::nav::system::Transform {
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
        let p = a.get::<og_ai::nav::system::Transform>(e).unwrap().position;
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
                    og_ai::nav::system::Transform {
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
            .map(|&e| w.get::<og_ai::nav::system::Transform>(e).unwrap().position)
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

// ── G2-4: areas, filters and shapes ─────────────────────────────────────────

/// A flat 20 × 10 m floor from the origin, 0.25 m cells.
fn floor() -> og_ai::nav::Heightfield {
    let p = NavParams {
        cell_size: 0.25,
        tile_size: 16,
        ..Default::default()
    };
    og_ai::nav::Heightfield::flat(p, Vec3::ZERO, 80, 40)
}

/// A swamp across the floor at x 9–11, from the near edge to z 8 (a gap from z 8 to 10).
fn swamp() -> og_ai::nav::Shape {
    og_ai::nav::Shape::Box {
        min: Vec3::new(9.0, -1.0, -1.0),
        max: Vec3::new(11.0, 1.0, 8.0),
    }
}

#[test]
fn filtered_agents_keep_out_of_an_area_and_shapes_and_areas_come_back_with_a_save() {
    use og_ai::nav::{NavFilter, Shape};
    let make = || {
        let mut navs = Navs::default();
        let h = navs.insert(Nav::Mesh(Box::new(NavMesh::build_with_areas(
            floor(),
            vec![(swamp(), 3)],
        ))));
        let navs = Arc::new(Mutex::new(navs));
        let mut w = world_with(navs.clone(), Plane::Xz);
        og_ai::nav::enable_filters(&mut w).unwrap();
        let agent = |w: &mut World, z: f32, filter: Option<NavFilter>| {
            let e = w
                .spawn((
                    og_ai::nav::system::Transform {
                        position: Vec3::new(2.0, 0.0, z),
                        ..Default::default()
                    },
                    NavAgent {
                        nav: h,
                        target: Vec3::new(18.0, 0.0, z),
                        seeking: true,
                        speed: 4.0,
                        ..Default::default()
                    },
                ))
                .unwrap();
            if let Some(f) = filter {
                w.insert(e, f).unwrap();
            }
            e
        };
        let mut keep_out = NavFilter::default();
        keep_out.excluded.push(3).unwrap();
        let careful = agent(&mut w, 4.0, Some(keep_out));
        let bold = agent(&mut w, 2.0, None);
        (w, navs, h, careful, bold)
    };
    let pos = |w: &World, e| w.get::<og_ai::nav::system::Transform>(e).unwrap().position;
    let (mut w, navs, h, careful, bold) = make();
    let (mut careful_z, mut bold_z) = (f32::MIN, f32::MIN);
    let mut snap = None;
    for t in 0..400 {
        if t == 100
            && let Some(Nav::Mesh(m)) = navs.lock().unwrap().get_mut(h)
        {
            m.carve(
                9,
                Some(Shape::Cylinder {
                    base: Vec3::new(15.0, 0.0, 2.0),
                    radius: 0.6,
                    height: 1.0,
                }),
            );
            m.set_area(
                4,
                Some((
                    Shape::Box {
                        min: Vec3::new(13.0, -1.0, 6.0),
                        max: Vec3::new(14.0, 1.0, 10.0),
                    },
                    5,
                )),
            );
        }
        if t == 150 {
            snap = Some(w.snapshot());
        }
        w.step().unwrap();
        let (c, b) = (pos(&w, careful), pos(&w, bold));
        assert!(
            !(c.x > 9.3 && c.x < 10.7 && c.z < 7.7),
            "the filtered agent waded the swamp at {c}"
        );
        careful_z = careful_z.max(c.z);
        bold_z = bold_z.max(b.z);
    }
    assert!(careful_z > 7.9, "it went round by the gap");
    assert!(bold_z < 5.0, "without a filter it went straight");
    assert!(pos(&w, careful).distance(Vec3::new(18.0, 0.0, 4.0)) < 0.5);
    // A fresh world restored at tick 150: its mesh takes the pillar and
    // the area back, and it goes on to the same world.
    let (mut fresh, fresh_navs, ..) = make();
    fresh.restore(&snap.unwrap()).unwrap();
    {
        let n = fresh_navs.lock().unwrap();
        let Some(Nav::Mesh(m)) = n.get(h) else {
            panic!("the mesh")
        };
        let Some(Nav::Mesh(orig)) = navs.lock().unwrap().get(h).cloned() else {
            panic!("the mesh")
        };
        assert_eq!(m.obstacles().count(), 1);
        assert_eq!(m.areas().count(), 1);
        assert_eq!(m.tiles, orig.tiles);
    }
    for _ in 150..400 {
        fresh.step().unwrap();
    }
    assert_eq!(fresh.world_hash(), w.world_hash());
}

/// A mesh edited more often than a restore edits it (a crate set, moved
/// and moved again) restores at the edit count its agents planned on, and
/// one edited after they planned restores as stale: both worlds go on tick
/// for tick (before, the restore's own count made every agent plan again).
#[test]
fn a_restored_mesh_keeps_the_edit_count_its_agents_planned_on() {
    use og_ai::nav::Shape;
    let crate_at = |x: f32| Shape::Box {
        min: Vec3::new(x, -1.0, 3.0),
        max: Vec3::new(x + 1.0, 1.0, 5.0),
    };
    let make = || {
        let mut navs = Navs::default();
        let h = navs.insert(Nav::Mesh(Box::new(NavMesh::build_with_areas(
            floor(),
            vec![],
        ))));
        let navs = Arc::new(Mutex::new(navs));
        let mut w = world_with(navs.clone(), Plane::Xz);
        w.spawn((
            og_ai::nav::system::Transform {
                position: Vec3::new(2.0, 0.0, 4.0),
                ..Default::default()
            },
            NavAgent {
                nav: h,
                target: Vec3::new(18.0, 0.0, 4.0),
                seeking: true,
                speed: 2.0,
                ..Default::default()
            },
        ))
        .unwrap();
        (w, navs, h)
    };
    let edit = |navs: &Arc<Mutex<Navs>>, h, x| {
        if let Some(Nav::Mesh(m)) = navs.lock().unwrap().get_mut(h) {
            m.carve(1, Some(crate_at(x)));
        }
    };
    for stale in [false, true] {
        let (mut w, navs, h) = make();
        for x in [6.0, 8.0, 12.0] {
            edit(&navs, h, x);
            w.step().unwrap();
        }
        for _ in 0..20 {
            w.step().unwrap();
        }
        if stale {
            edit(&navs, h, 13.0);
        }
        let snap = w.snapshot();
        let (mut fresh, ..) = make();
        fresh.restore(&snap).unwrap();
        for t in 0..200 {
            w.step().unwrap();
            fresh.step().unwrap();
            assert_eq!(
                fresh.world_hash(),
                w.world_hash(),
                "stale {stale}: diverged {t} ticks after the restore"
            );
        }
    }
}

// ── G2-4: off-mesh links ────────────────────────────────────────────────────

/// Two floors of 18 × 10 m split by a gap at x 8–10 from z 0 to 8: the way
/// round is a corridor across the gap's far end (z 8–10).
fn gap() -> NavMesh {
    let p = NavParams {
        cell_size: 0.25,
        tile_size: 16,
        ..Default::default()
    };
    NavMesh::build(
        og_ai::nav::Heightfield::flat(p, Vec3::ZERO, 72, 40).without(|x, z| {
            let (wx, wz) = (x as f32 * 0.25, z as f32 * 0.25);
            (8.0..10.0).contains(&wx) && wz < 8.0
        }),
    )
}

fn jump(cost: f32) -> og_ai::nav::Link {
    og_ai::nav::Link {
        start: Vec3::new(7.3, 0.0, 2.0),
        end: Vec3::new(10.7, 0.0, 2.0),
        kind: og_ai::nav::LinkKind::Jump,
        bidirectional: true,
        cost,
        open: true,
        duration: 0.0,
        game: false,
    }
}

fn link_world(link: og_ai::nav::Link) -> (World, Arc<Mutex<Navs>>, u64, og_ecs::Entity) {
    let mut navs = Navs::default();
    let mut m = gap();
    m.set_link(1, Some(link));
    let h = navs.insert(Nav::Mesh(Box::new(m)));
    let navs = Arc::new(Mutex::new(navs));
    let mut w = world_with(navs.clone(), Plane::Xz);
    og_ai::nav::enable_links(&mut w).unwrap();
    let e = w
        .spawn((
            og_ai::nav::system::Transform {
                position: Vec3::new(4.0, 0.0, 2.0),
                ..Default::default()
            },
            NavAgent {
                nav: h,
                target: Vec3::new(14.0, 0.0, 2.0),
                seeking: true,
                speed: 4.0,
                ..Default::default()
            },
        ))
        .unwrap();
    (w, navs, h, e)
}

fn reached(w: &World) -> Vec<og_ai::nav::NavLinkReached> {
    w.pending_events::<og_ai::nav::NavLinkReached>().unwrap()
}

#[test]
fn an_agent_jumps_a_link_on_an_arc_and_a_restore_mid_jump_lands_alike() {
    let (mut w, _, h, e) = link_world(jump(0.0));
    let pos = |w: &World| w.get::<og_ai::nav::system::Transform>(e).unwrap().position;
    let (mut top, mut on_link, mut events, mut snap) = (0.0f32, 0, Vec::new(), None);
    for _ in 0..240 {
        w.step().unwrap();
        events.extend(reached(&w));
        let p = pos(&w);
        top = top.max(p.y);
        if w.get::<NavAgent>(e).unwrap().status == og_ai::nav::STATUS_ON_LINK {
            on_link += 1;
            if on_link == 5 {
                snap = Some(w.snapshot());
            }
            assert!(p.x > 7.0 && p.x < 11.0, "mid-jump over the gap at {p}");
        } else {
            assert!(p.y.abs() < 1e-3, "off the link, on the floor at {p}");
            assert!(p.z < 3.5, "it jumped rather than going round at {p}");
        }
    }
    assert_eq!(events.len(), 1, "one og.NavLinkReached");
    let ev = events[0];
    assert_eq!(
        (ev.agent, ev.mesh, ev.link, ev.kind),
        (e.to_bits(), h, 1, 0)
    );
    assert!(
        on_link >= 10,
        "a jump of 3.4 m at 4 m/s lasts some ticks: {on_link}"
    );
    assert!(top > 0.8, "on an arc: {top}");
    assert_eq!(w.get::<NavAgent>(e).unwrap().status, STATUS_ARRIVED);
    assert!(pos(&w).distance(Vec3::new(14.0, 0.0, 2.0)) < 0.3);
    let hash = w.world_hash();
    // A fresh world restored mid-jump lands and arrives the same.
    let (mut z, ..) = link_world(jump(0.0));
    z.restore(&snap.unwrap()).unwrap();
    assert_eq!(
        z.get::<NavAgent>(e).unwrap().status,
        og_ai::nav::STATUS_ON_LINK
    );
    while z.time().tick < w.time().tick {
        z.step().unwrap();
    }
    assert_eq!(z.world_hash(), hash);
}

#[test]
fn a_game_crossing_waits_for_finish_a_teleport_is_instant_and_a_shut_door_reroutes() {
    // The game moves the agent across: it waits on the link until told.
    let mut game = jump(0.0);
    game.kind = og_ai::nav::LinkKind::Ladder;
    game.game = true;
    let mut navs = Navs::default();
    let mut m = gap();
    m.set_link(1, Some(game));
    let h = navs.insert(Nav::Mesh(Box::new(m)));
    let navs = Arc::new(Mutex::new(navs));
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 3,
    })
    .unwrap();
    let sys = install(
        &mut w,
        navs.clone(),
        NavDesc {
            plane: Plane::Xz,
            raycast: None,
            plans_per_tick: og_ai::nav::PLANS_PER_TICK,
        },
    )
    .unwrap();
    og_ai::nav::enable_links(&mut w).unwrap();
    let e = w
        .spawn((
            og_ai::nav::system::Transform {
                position: Vec3::new(4.0, 0.0, 2.0),
                ..Default::default()
            },
            NavAgent {
                nav: h,
                target: Vec3::new(14.0, 0.0, 2.0),
                seeking: true,
                speed: 4.0,
                ..Default::default()
            },
        ))
        .unwrap();
    let mut events = Vec::new();
    for _ in 0..120 {
        w.step().unwrap();
        events.extend(reached(&w));
    }
    assert_eq!(events.len(), 1);
    assert!(events[0].game && events[0].kind == 1);
    assert_eq!(
        w.get::<NavAgent>(e).unwrap().status,
        og_ai::nav::STATUS_ON_LINK
    );
    let held = w.get::<og_ai::nav::system::Transform>(e).unwrap().position;
    assert!(held.x < 8.0, "it waits at the ladder's foot: {held}");
    assert!(sys.finish_link(e));
    assert!(!sys.finish_link(og_ecs::Entity::from_bits(0xdead)));
    w.step().unwrap();
    let landed = w.get::<og_ai::nav::system::Transform>(e).unwrap().position;
    assert!(
        landed.distance(Vec3::new(10.7, 0.0, 2.0)) < 1e-3,
        "landed at {landed}"
    );
    for _ in 0..120 {
        w.step().unwrap();
    }
    assert_eq!(w.get::<NavAgent>(e).unwrap().status, STATUS_ARRIVED);

    // A teleport: on the far side the tick after it is reached.
    let mut tp = jump(0.0);
    tp.kind = og_ai::nav::LinkKind::Teleport;
    let (mut w, _, _, e) = link_world(tp);
    let mut last_x = 4.0f32;
    for _ in 0..200 {
        w.step().unwrap();
        let x = w
            .get::<og_ai::nav::system::Transform>(e)
            .unwrap()
            .position
            .x;
        assert!(
            !(x > 7.6 && x < 10.4),
            "a teleport passes no point over the gap: {x}"
        );
        if last_x < 8.0 && x > 10.0 {
            assert!(x - last_x > 3.0, "one tick from start to end");
        }
        last_x = x;
    }
    assert!(last_x > 13.5);

    // A door shut as the agent walks to it: it goes round by the corridor.
    let mut door = jump(0.0);
    door.kind = og_ai::nav::LinkKind::Door;
    let (mut w, navs, h, e) = link_world(door);
    let mut top = 0.0f32;
    for t in 0..600 {
        if t == 20
            && let Some(Nav::Mesh(m)) = navs.lock().unwrap().get_mut(h)
        {
            assert!(m.set_link_open(1, false));
        }
        w.step().unwrap();
        assert!(reached(&w).is_empty(), "never crosses a shut door");
        top = top.max(
            w.get::<og_ai::nav::system::Transform>(e)
                .unwrap()
                .position
                .z,
        );
    }
    assert!(top > 7.9, "round by the corridor: {top}");
    assert_eq!(w.get::<NavAgent>(e).unwrap().status, STATUS_ARRIVED);
}
