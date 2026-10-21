//! Prints the host's numbers in the format `load.mjs` prints the wasm's,
//! then what lanes did (`run.sh` runs it with `OG_LANES=off` and on).

fn main() {
    println!("math_digest {}", wasm_determinism::math_digest());
    println!(
        "bouncing_boxes_hash {}",
        wasm_determinism::bouncing_boxes_hash()
    );
    #[cfg(feature = "physics3d")]
    println!("physics3d_hash {}", wasm_determinism::physics3d_hash());
    #[cfg(feature = "physics2d")]
    println!("physics2d_hash {}", wasm_determinism::physics2d_hash());
    println!("anim_hash {}", wasm_determinism::anim_hash());
    let lanes = og_lanes::report();
    println!("lanes_threads {}", lanes.threads);
    println!("lanes_parallel_calls {}", lanes.parallel_calls);
}
