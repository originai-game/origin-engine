# E1 animation: test run, break-and-restore, provenance

Working tree on main at 5968f7c (+ uncommitted og-anim/og-cook changes), Apple
M4 Pro, rustc 1.97.0, 2026-10-18. `scripts/evidence.sh` accepts only `M<n>`
milestones, so these files were copied here by hand; this file is their
provenance.

## Files

- `anim-walk-determinism.json`: `OG_EVIDENCE_DIR=<dir> CARGO_TARGET_DIR=target/agent-anim cargo test --release -p og-anim --test walk`, copied from `<dir>`.
- `anim-walk-determinism-debug.json`: the same in debug (`cargo test -p og-anim`). Hashes equal the release run.
- `anim-cook-resample.json`: `OG_EVIDENCE_DIR=<dir> cargo test --release -p og-cook --test animation` (per clip: source rate, rate, frames, samples checked, largest error against the independent glTF sampler; the walker at its source rates and at 60 Hz, trickster at its source rate).
- `anim-wasm-size.json`: the probe cdylib described in the file (source below).

## Results

```
cargo test --release -p og-anim -p og-cook        # all green
  og-anim: graph 3, ik 5, output 5, playback 7, walk 1
  og-cook: animation 4, scene_bool 1, template 7 (incl. the gfx reference with cooked models), og-blob 5
cargo test -p og-anim                             # debug: all green, walk 13.6 s
cargo clippy -p og-anim -p og-cook --all-targets -- -D warnings   # clean (og-render at HEAD; the working tree's og-render is mid-edit by the Render agent)
cargo fmt --check -p og-anim -p og-cook           # clean
cargo check -p og-anim --target wasm32-unknown-unknown            # ok
cargo check -p og-cook --no-default-features --features runtime --target wasm32-unknown-unknown   # ok
```

World hash after 3600 ticks (seed 7): `472dafa72c5124a8` on native debug,
native release, wasm32 scalar, wasm32 after `wasm-opt -Oz`, wasm32
`+simd128` (node v26.3.0). The restore at tick 1800 continues to the same
hashes at 2400, 3000, 3600.

## Broken once, then restored

Each change was applied, the named test run in release, the change reverted.

| Break | Failed |
|---|---|
| state block writes 0 for the player time | `walk`: the restored world continues differently |
| markers fire on `(a, b]` instead of `[a, b)` | `a_marker_at_zero_fires_on_the_first_tick_and_at_every_loop` |
| knee bends away from the pole | `two_bone_reaches_a_reachable_target_and_bends_to_the_pole` |
| wrong law of cosines for the knee | three `ik` tests |
| FABRIK straightens an unreachable chain along +y | `fabrik_reaches_and_clamps` |
| component blendshape weights not written through | `component_weights_write_through` |
| graph cross-fade skips the outgoing state | `transitions_fire_when_their_condition_holds_and_cross_fade` |
| cook: rotations lerped instead of slerped | `cooked_clips_resample_to_the_gltf_sampler_within_1e_5` (the walker's 120-degree `turn` at 60 Hz) |
| cook: cubic tangents not scaled by the key interval | the same |
| cook: STEP keys one sample late | the same |
| cook: skin streams in source order, not cooked order | `skins_and_morphs_follow_the_cooked_vertices` |
| loader: inverse bind matrices transposed | the same |
| cook: `extras."og.events"` ignored | `markers_come_from_extras_and_import_settings`, `a_cooked_walker_plays_in_og_anim` |

Two breaks did not fail, and why: removing the two-bone distance clamp (the
solver turns joints only, so bone lengths and the straight-at-the-target
result are kept anyway; the clamp only guards the square root), and forcing
the solver's internal `reachable` flag to true (`reached` also requires the
tip within `tolerance`, which an unreachable target never is).

## Size probe source

`Cargo.toml`: package `anim-probe`, `crate-type = ["cdylib", "rlib"]`, a bin
`src/main.rs`, feature `anim = ["dep:og-anim"]`, path deps on og-core, og-ecs,
og-anim, glam 0.34.1 with `scalar-math`, release opt-level 3, thin LTO,
codegen-units 1, an empty `[workspace]`, the workspace `Cargo.lock`.

```rust
#[cfg(feature = "anim")]
#[path = "<repo>/crates/og-anim/tests/support/mod.rs"]
mod support;

#[no_mangle]
pub extern "C" fn run(ticks: u32) -> u64 {
    #[cfg(feature = "anim")]
    {
        let (mut w, _anim) = support::walk::build(7);
        for _ in 0..ticks { w.step().unwrap(); }
        let s = w.snapshot();
        w.restore(&s).unwrap();
        w.world_hash()
    }
    #[cfg(not(feature = "anim"))]
    {
        let mut w = og_ecs::World::new(&og_ecs::WorldDesc { tick_rate: 60, seed: 7 }).unwrap();
        for _ in 0..ticks { w.step().unwrap(); }
        let s = w.snapshot();
        w.restore(&s).unwrap();
        w.world_hash()
    }
}
```

Run under node: `WebAssembly.instantiate(bytes, {})`, `exports.run(3600)`.
