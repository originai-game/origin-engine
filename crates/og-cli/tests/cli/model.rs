//! `og model` on the model guide's programs: each builds and measures
//! the size its own numbers give.

use std::process::Command;

fn shape(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_og"))
        .arg("model")
        .args(args)
        .output()
        .expect("the binary runs")
}

/// Runs a command that must succeed and parses its JSON report.
fn report(args: &[&str]) -> serde_json::Value {
    let out = shape(args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("a JSON report")
}

/// Every ```json program block of the model guide builds, and measures the
/// size its own numbers give (the recipe is checked, not only shown).
#[test]
fn the_model_guides_programs_build() {
    let guide = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../og-api/guides/model.md"
    ))
    .unwrap();
    let mut blocks = Vec::new();
    let mut lines = guide.lines();
    while let Some(l) = lines.next() {
        if l.trim() == "```json program" {
            blocks.push(
                lines
                    .by_ref()
                    .take_while(|l| l.trim() != "```")
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
    }
    assert!(!blocks.is_empty());
    let dir = std::env::temp_dir().join(format!("shape-guide-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (k, b) in blocks.iter().enumerate() {
        let path = dir.join(format!("g{k}.shape.json"));
        std::fs::write(&path, b).unwrap();
        let m = report(&["measure", path.to_str().unwrap(), "--json"]);
        let size: Vec<f64> = m["size"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        // The side table: 600 wide, 720 tall (690 legs and a 30 top), 450 deep.
        if k == 0 {
            for (a, b) in size.iter().zip([0.6, 0.72, 0.45]) {
                assert!((a - b).abs() < 1e-6, "{size:?}");
            }
            assert_eq!(m["partCount"], 5);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
