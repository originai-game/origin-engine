//! og-ai's navigation through its public API: the og.nav agent system in a
//! world.
//!
//! Ways it could fail: two runs of a crowd differ, or a world restored from
//! a snapshot replays to a different world.

use og_ai::nav::{BLOCKED, Grid, Nav, NavAgent, NavDesc, Navs, Plane, STATUS_ARRIVED, install};
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
