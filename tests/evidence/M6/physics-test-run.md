# og-physics M6 test run (49ed935)

Run from a `git archive 49ed935` export (the working tree had another agent's uncommitted edits in crates/og-physics/src at the time), `CARGO_TARGET_DIR=target/agent-qa`, Apple M4 Pro, 2026-10-18:

- `OG_PHYSICS_PILE_REPORT=… cargo test -p og-physics -- --nocapture`: every test file passes (bodies 7, edits 7, pile 1, profile_2d 3, queries 3).
- `OG_PHYSICS_PILE_REPORT=… cargo test --release -p og-physics --test pile -- --nocapture`: passes.

```
test a_broken_collider_stops_the_tick_with_the_entity_named ... ok
test locked_axes_and_gravity_scale_hold ... ok
test a_sensor_reports_entry_and_exit_and_does_not_stop_what_passes ... ok
test touching_starts_once_ends_once_and_a_removed_floor_stops_holding_the_box ... ok
test kinematic_bodies_follow_their_transform_and_push_dynamic_ones ... ok
test layers_filter_collisions ... ok
test a_dropped_box_comes_to_rest_on_a_static_floor_and_its_transform_is_written_back ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test a_velocity_set_by_game_code_launches_the_body ... ok
test a_despawned_body_stops_holding_up_what_rests_on_it ... ok
test a_static_collider_moved_by_game_code_moves_in_the_simulation ... ok
test a_teleport_moves_the_body_and_it_falls_from_there ... ok
test a_resized_collider_and_a_body_made_static_take_effect ... ok
test a_mesh_registered_again_changes_its_colliders_only_when_its_geometry_changes ... ok
test a_restore_into_a_world_that_ran_on_continues_from_the_snapshot ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test the_pile_is_deterministic_and_a_restore_at_tick_1800_ends_in_the_same_world ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.94s
test the_2d_profile_refuses_3d_only_shapes ... ok
test a_circle_rolls_down_a_slope ... ok
test a_polygon_from_a_mesh_lands_flat ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test a_shape_cast_stops_at_the_first_surface_and_overlap_lists_what_it_touches ... ok
test a_raycast_hits_the_nearest_body_and_honours_the_filter ... ok
test queries_see_where_the_step_moved_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## The pile (physics-pile-determinism.json, physics-pile-determinism-debug.json)

500 dynamic bodies, a kinematic paddle, a sensor and churn (180 despawn/respawns), seed 42, 3600 ticks. Both profiles give the same hashes: tick 0 `983bd3cd8dae9c2f`, tick 1800 `52d3656938239c47`, tick 3600 `1cc878a44e5a47f3`, and the run restored at tick 1800 into a fresh world (seed 99) ends at `1cc878a44e5a47f3`; 69,383 collisions started, 68,050 ended, 253 sensor entries, 211 exits. About 1.8–2.0 ms per tick for 500 bodies under load (dev builds rapier at opt-level 3).

## wasm size (physics-wasm-size.json)

og-host-web does not link og-physics at 49ed935, so the gate's size report does not see it. Measured with a probe cdylib outside the workspace, the gate's pipeline minus wasm-bindgen: physics adds **694 KB** brotli for the 3D profile, **541 KB** for 2D and **1.05 MB** for both, which would put og_host_web.wasm at 1.79 / 1.64 / 2.15 MB brotli against the 4 MB core budget (docs/phase-1.md §5's feature-gating fallback is not needed on these numbers). The probe's source:

```rust
//! QA probe: the wasm bytes og-physics adds to a world that steps it.
use og_ecs::{World, WorldDesc};

#[no_mangle]
pub extern "C" fn run(ticks: u32, profile: u32) -> u64 {
    let mut w = World::new(&WorldDesc { tick_rate: 60, seed: 7 }).unwrap();
    #[cfg(feature = "physics")]
    {
        use og_core::glam::Vec3;
        use og_physics::*;
        w.register::<Transform>().unwrap();
        let settings = if profile == 2 { Physics::profile_2d() } else { Physics::profile_3d() };
        let sys = install(&mut w, settings).unwrap();
        w.spawn((Transform::at(Vec3::new(0.0, -0.5, 0.0)), Collider::cuboid(Vec3::new(40.0, 1.0, 40.0)))).unwrap();
        for i in 0..ticks.min(64) {
            let c = match i % 4 { 0 => Collider::sphere(0.5), 1 => Collider::capsule(0.3, 1.0), 2 => Collider::cylinder(0.4, 0.8), _ => Collider::cuboid(Vec3::ONE) };
            w.spawn((Transform::at(Vec3::new(i as f32 * 0.1, 2.0 + i as f32, 0.0)), RigidBody::dynamic(), c)).unwrap();
        }
        sys.set_mesh(1, MeshDesc { positions: vec![Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z], indices: vec![[0, 1, 2], [0, 2, 3]] }).ok();
        w.spawn((Transform::at(Vec3::new(5.0, 1.0, 0.0)), Collider::trimesh(1))).unwrap();
        w.spawn((Transform::at(Vec3::new(-5.0, 1.0, 0.0)), RigidBody::dynamic(), Collider::convex(1))).unwrap();
        for _ in 0..ticks { w.step().unwrap(); }
        let q = sys.query(&w).unwrap();
        let hit = q.raycast(&RaycastDesc { origin: Vec3::Y * 50.0, direction: -Vec3::Y, ..Default::default() }).ok().flatten();
        let snap = w.snapshot();
        w.restore(&snap).unwrap();
        return w.world_hash() ^ (hit.is_some() as u64);
    }
    #[allow(unreachable_code)]
    {
        let _ = profile;
        for _ in 0..ticks { w.step().unwrap(); }
        let snap = w.snapshot();
        w.restore(&snap).unwrap();
        w.world_hash()
    }
}
```

```toml
[package]
name = "physics-size-probe"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ["cdylib"]

[features]
default = []
physics = ["dep:og-physics"]
p2d = ["physics", "og-physics/physics2d"]
p3d = ["physics", "og-physics/physics3d"]

[dependencies]
og-core = { path = "<49ed935 export>/crates/og-core" }
og-ecs = { path = "<49ed935 export>/crates/og-ecs" }
og-physics = { path = "<49ed935 export>/crates/og-physics", default-features = false, optional = true }

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1

[workspace]
```
