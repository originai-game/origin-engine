//! The golden hashes: every case's `<case> <sha256>`, sorted by name,
//! must equal `hashes.txt` byte for byte, natively and as wasm32-wasip1
//! under wasmtime (the gate's `shape-golden-wasm32` step). A change that
//! moves a hash says why in its commit and changelog, and replaces the
//! lines this test prints.

mod bake;
mod bevel;
mod cases;
mod font;
mod gen2;
mod geom;
mod interference;
mod look;
mod mcp;
mod meshrepair;
mod meshtools;
mod model;
mod painted;
mod place;
mod rest;
mod selfunion;
mod skin;
mod smallgaps;
mod solids;
mod surface;
mod tools;
mod touching;
mod uvs;

#[test]
fn every_case_hashes_as_recorded() {
    let mut cases = cases::all();
    cases.sort_by(|a, b| a.name.cmp(b.name));
    let mut out = String::new();
    for case in cases {
        let mut hash = og_shape::math::OutputHash::new();
        match (case.run)(&mut hash) {
            Ok(()) => out += &format!("{} {}\n", case.name, hash.finish_hex()),
            Err(err) => out += &format!("{} error: {err}\n", case.name),
        }
    }
    let recorded = include_str!("hashes.txt");
    let moved: Vec<String> = out
        .lines()
        .filter(|l| !recorded.lines().any(|r| r == *l))
        .map(String::from)
        .collect();
    let gone: Vec<&str> = recorded
        .lines()
        .filter(|r| !out.lines().any(|l| l == *r))
        .collect();
    assert!(
        moved.is_empty() && gone.is_empty(),
        "golden hashes moved (tests/it/golden/hashes.txt).\nNow:\n{}\nRecorded:\n{}",
        moved.join("\n"),
        gone.join("\n"),
    );
}
