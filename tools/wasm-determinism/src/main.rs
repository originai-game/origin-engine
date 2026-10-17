//! Prints the host's numbers in the format `load.mjs` prints the wasm's.

fn main() {
    println!("math_digest {}", wasm_determinism::math_digest());
    println!(
        "bouncing_boxes_hash {}",
        wasm_determinism::bouncing_boxes_hash()
    );
}
