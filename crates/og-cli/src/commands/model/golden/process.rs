//! Golden cases for the front ends (`shape-cli`'s commands, which the
//! `shape` binary and the MCP server `shape-mcp` both call): their JSON
//! reports and files are the kernel's, so they are the same bytes on
//! every target.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::commands::model::process::{ProcessDesc, process};
use crate::commands::model::{Failure, Files, Input};
use og_shape::math::{HashError, OutputHash};
use og_shape::tools::{Floaters, Repair};

use super::Case;

#[path = "../../../../../og-shape-formats/tests/it/support/painted.rs"]
mod painted;

pub fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "mcp.process.ball_defects",
            run: process_case,
        },
        Case {
            name: "mcp.process.look",
            run: look_case,
        },
        Case {
            name: "mcp.process.bake",
            run: bake_case,
        },
    ]
}

fn fail(e: impl std::fmt::Display) -> HashError {
    eprintln!("mcp: {e}");
    HashError::NaN
}

/// One file in memory (wasmtime gives the golden run no directory), and
/// the files written.
struct Memory {
    input: (PathBuf, Vec<u8>),
    written: RefCell<Vec<(PathBuf, Vec<u8>)>>,
}

impl Files for Memory {
    fn read(&self, path: &Path) -> Result<Vec<u8>, Failure> {
        if path == self.input.0 {
            Ok(self.input.1.clone())
        } else {
            Err(Failure::Failed(format!("{}: no such file", path.display())))
        }
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), Failure> {
        self.written
            .borrow_mut()
            .push((path.to_path_buf(), bytes.to_vec()));
        Ok(())
    }
}

/// The `process` chain on the committed mesh with defects: its report
/// and the plain GLB it writes.
fn process_case(h: &mut OutputHash) -> Result<(), HashError> {
    let files = Memory {
        input: (
            PathBuf::from("ball_defects.obj"),
            include_bytes!("../../../../../og-shape/tests/data/meshes/ball_defects.obj").to_vec(),
        ),
        written: RefCell::new(Vec::new()),
    };
    let report = process(
        &files,
        &ProcessDesc {
            input: Input::Path(PathBuf::from("ball_defects.obj")),
            repair: Repair {
                weld: 0.001,
                floaters: Floaters {
                    fraction: 0.05,
                    ..Default::default()
                },
                max_hole_edges: 64,
                ..Default::default()
            },
            budget: 1000,
            intersections: true,
            out: Some(PathBuf::from("out.glb")),
            ..Default::default()
        },
    )
    .map_err(fail)?;
    h.str(&report);
    for (path, bytes) in files.written.borrow().iter() {
        h.str(&path.display().to_string());
        h.bytes(bytes);
    }
    Ok(())
}

/// The `process` chain on a coloured, textured sphere with a material in
/// a GLB: kept UVs, maps shrunk to 2 texels, the report and the GLB; then
/// again with fresh UVs (every map dropped, and said so) to OBJ (colours,
/// UVs and material not written, and said so).
fn look_case(h: &mut OutputHash) -> Result<(), HashError> {
    let s = painted::painted_sphere()?;
    let mut part = og_shape::tools::Part::new("ball", &s);
    part.material = painted::material();
    let glb = og_shape_formats::gltf::write::glb(&og_shape::tools::Scene {
        name: "look".into(),
        parts: vec![part],
    })
    .map_err(fail)?;
    for (fresh_uvs, out) in [(false, "out.glb"), (true, "out.obj")] {
        let files = Memory {
            input: (PathBuf::from("ball.glb"), glb.clone()),
            written: RefCell::new(Vec::new()),
        };
        let report = process(
            &files,
            &ProcessDesc {
                input: Input::Path(PathBuf::from("ball.glb")),
                repair: Repair {
                    weld: 0.001,
                    ..Default::default()
                },
                budget: 300,
                fresh_uvs,
                texture_max: 2,
                out: Some(PathBuf::from(out)),
                ..Default::default()
            },
        )
        .map_err(fail)?;
        h.str(&report);
        for (path, bytes) in files.written.borrow().iter() {
            h.str(&path.display().to_string());
            h.bytes(bytes);
        }
    }
    Ok(())
}

/// `process --bake` at 64 texels: the coloured sphere with every map (a
/// 4 × 2 normal map applied, the vertex colours multiplied in), and the
/// committed mesh with defects (normals only); each report and GLB (the
/// baked PNGs inside).
fn bake_case(h: &mut OutputHash) -> Result<(), HashError> {
    let s = painted::painted_sphere()?;
    let mut part = og_shape::tools::Part::new("ball", &s);
    part.material = painted::material();
    let glb = og_shape_formats::gltf::write::glb(&og_shape::tools::Scene {
        name: "look".into(),
        parts: vec![part],
    })
    .map_err(fail)?;
    let obj = include_bytes!("../../../../../og-shape/tests/data/meshes/ball_defects.obj").to_vec();
    for (name, bytes, budget) in [("ball.glb", glb, 300), ("ball_defects.obj", obj, 500)] {
        let files = Memory {
            input: (PathBuf::from(name), bytes),
            written: RefCell::new(Vec::new()),
        };
        let report = process(
            &files,
            &ProcessDesc {
                input: Input::Path(PathBuf::from(name)),
                repair: Repair {
                    weld: 0.001,
                    floaters: Floaters {
                        fraction: 0.05,
                        ..Default::default()
                    },
                    max_hole_edges: 64,
                    ..Default::default()
                },
                budget,
                bake: 64,
                out: Some(PathBuf::from("out.glb")),
                ..Default::default()
            },
        )
        .map_err(fail)?;
        h.str(&report);
        for (path, bytes) in files.written.borrow().iter() {
            h.str(&path.display().to_string());
            h.bytes(bytes);
        }
    }
    Ok(())
}
