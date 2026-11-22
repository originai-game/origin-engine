//! The golden hashes of `og model`'s commands: every case's
//! `<case> <sha256>`, which must equal `hashes.txt` byte for byte. The
//! kernel's own cases are og-shape-formats' (`tests/it/golden`), which
//! also run on wasm32.

mod place;
mod process;

use og_shape::math::{HashError, OutputHash};

/// One golden case.
pub struct Case {
    /// Unique name; the hash file is keyed by it.
    pub name: &'static str,
    /// Writes the command's output into the hash.
    pub run: fn(&mut OutputHash) -> Result<(), HashError>,
}

#[test]
fn every_case_hashes_as_recorded() {
    let mut cases = process::cases();
    cases.extend(place::cases());
    cases.sort_by(|a, b| a.name.cmp(b.name));
    let mut out = String::new();
    for case in cases {
        let mut hash = OutputHash::new();
        match (case.run)(&mut hash) {
            Ok(()) => out += &format!("{} {}\n", case.name, hash.finish_hex()),
            Err(err) => out += &format!("{} error: {err}\n", case.name),
        }
    }
    let recorded = include_str!("hashes.txt");
    let moved: Vec<&str> = out
        .lines()
        .filter(|l| !recorded.lines().any(|r| r == *l))
        .collect();
    let gone: Vec<&str> = recorded
        .lines()
        .filter(|r| !out.lines().any(|l| l == *r))
        .collect();
    assert!(
        moved.is_empty() && gone.is_empty(),
        "golden hashes moved (golden/hashes.txt).\nNow:\n{}\nRecorded:\n{}",
        moved.join("\n"),
        gone.join("\n"),
    );
}
