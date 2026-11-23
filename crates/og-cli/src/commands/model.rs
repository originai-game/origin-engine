//! `og model <command>`: og-shape's model and mesh commands. Every
//! result is the kernel's; these modules only read inputs, call the
//! kernel and write its outputs.
//!
//! Entry: [`source::load`] reads a [`Source`]: a model program
//! (`og_shape::model::program`) or a mesh file (glb, gltf, obj, fbx), through
//! a [`Files`] that does the I/O (plain files for the command line, a
//! confined workspace for the server). Each command is one module:
//!
//! - [`build`]: the built model's parts and joints.
//! - [`measure`]: measures as JSON, with no thresholds.
//! - [`preview`]: the CPU turntable as one PNG.
//! - [`export`]: GLB, OBJ or STL.
//! - [`import`]: what a mesh file holds, and what was skipped.
//! - [`repair`], [`decimate`], [`unwrap`]: the mesh tools on a mesh file.
//! - [`place`]: a mesh file turned, sized and moved to a base pivot.
//! - [`process`]: the chain import → repair → decimate → unwrap →
//!   measure → export under a triangle budget, or a prop folder.
//! - [`bake`]: base colour projected from concept images onto the UVs.
//!
//! Reports are JSON with keys in a fixed order ([`report`]); errors are
//! [`Failure`] values. [`args`] reads the command line.

use std::process::ExitCode;

pub mod args;
pub mod axes;
pub mod bake;
pub mod build;
pub mod collide;
pub mod compare;
pub mod compress;
pub mod decimate;
pub mod export;
pub mod import;
pub mod issues;
pub mod measure;
pub mod place;
pub mod preview;
pub mod process;
pub mod repair;
pub mod report;
pub mod rig;
pub mod source;
pub mod thin;
pub mod transfer;
pub mod unwrap;
pub mod views;

#[cfg(test)]
mod golden;

pub use source::{Files, Input};

/// Why a command did not run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The command was asked wrongly (a missing or unknown option).
    Usage(String),
    /// The command ran and failed: the kernel's error, or an I/O error.
    Failed(String),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Usage(w) | Failure::Failed(w) => f.write_str(w),
        }
    }
}

impl std::error::Error for Failure {}

/// A kernel error as a failure.
pub fn failed(e: impl std::fmt::Display) -> Failure {
    Failure::Failed(e.to_string())
}

const USAGE: &str = "usage: og model <command> <input> [options]
  an input is a program (.json) or a mesh file (.glb .gltf .obj .fbx);
  OBJ takes --unit-mm MM (default metres); OBJ and glTF take --up AXIS and
  --front AXIS (+x -x +y -y +z -z: the file's axes that are the object's up and
  front; default up +y, front +z) or --z-up (up +z, front -y); a program takes --uvs
  (build, export, measure, preview): every part it makes gets UVs
  build    <program.json>                       report the built model as JSON
  export   <input> -o <out.glb|obj|stl|urdf|usda|usdz>
                                                write the posed model or the mesh;
                                                .urdf with --density KG_M3 (inertials),
                                                --effort and --velocity (joint limits);
                                                .usda and .usdz with skins and clips;
                                                OBJ and STL at the pose if one is set;
                                                --atlas PX: a program's parts' UVs in
                                                one atlas, alike materials shared
  measure  <input> [--intersections] [--posed] [--axes]
                                                measures as JSON; --posed: a program
                                                at its pose, its joints and interference;
                                                --axes: a mesh's extents, area facing
                                                each way and where its mass sits
  preview  <input> -o <out.png> [--views N] [--width PX] [--height PX]
           [--columns N] [--elevation RAD] [--azimuth RAD] [--no-edges] [--no-ground]
           [--fov RAD] [--unlit] [--each] [--uv [--uv-size PX]] [--env PICTURE] [--axes]
                                                draws base-colour maps and vertex colours;
                                                --each: a PNG a view with its camera;
                                                --uv: each material's UV layout instead
                                                --axes: the engine axes at the origin,
                                                labelled (+Z FRONT, +Y UP, +X)
  preview  <input> --views front,left,… --out <dir> [--width PX] [--unlit]
                                                a transparent PNG a named view, framed as
                                                compare frames it; --unlit: as compare draws it
  import   <mesh> [-o out.glb|obj|stl]          parts, images and what was skipped
  repair   <mesh> [-o out] [--weld MM] [--weld-all] [--floaters FRACTION]
           [--floater-size MM] [--non-manifold] [--holes N [--holes-through]] [--thicken MM] [--fuse]
           [--min-edge MM] [--intersections]
  decimate <mesh> (--ratio R | --target N | --lod-ratios A,B,…) [--max-error MM]
           [--lock-border]
           [--merge-charts UV] [--cross-color-borders] [--lods N] [-o out]
  unwrap   <mesh> [-o out.glb] [--angle DEG] [--resolution PX] [--padding TEXELS] [--shared] [--conformal] [--seams]
  place    <mesh> [-o out] [--up AXIS] [--front AXIS] [--turn DEG]
           [--longest L | --height L | --width L | --depth L | --scale F]
           [--pivot base|center|X,Y,Z]      turn, size (L with a unit: 1.2m, 90cm,
                                                900mm) and origin, before and after
  compress <file.glb> -o out.glb [--keep-order]
                                                meshopt, lossless (EXT_meshopt_compression);
                                                also tried in a vertex-cache order, the
                                                smaller written
           [--quantize [--position-bits N]]     first: attributes in fewer bytes (lossy)
           [--draco [--position-bits N] [--normal-bits N] [--uv-bits N] [--color-bits N]
            [--weight-bits N] [--sequential]]   Draco, quantized (KHR_draco_mesh_compression):
                                                an EdgeBreaker walk where the mesh can be
                                                walked, or with --sequential in the file's order
  collide  <input> [-o out] [--hulls N] [--concavity MM] [--max-vertices N]
                                                convex hulls over each part: colliders
  process  <mesh> [--budget N | --ratio R] [--merge-charts UV] [--cross-color-borders]
           [-o out.glb]
           [repair, unwrap and place options]
           [--intersections] [--unwrap] [--max-texture PX] [--ktx2 | --ktx2-rdo Q | --etc1s | --webp]
           [--bake PX [--bake-distance MM]]    --bake: fresh UVs packed at PX², the
                                                look baked into new maps
  bake     <input> --project VIEW=IMAGE.png … [--size PX] [--view-size PX] [--out DIR]
                                                base colour from concept images (views as
                                                preview's), per material beside the model
           <variant.ogt> --from <base.ogt>      a variant's maps carried from its baked base
  rig      <mesh> [-o out.glb] [--skeleton joints.json] [--check-clip]
                                                a skin by bone heat: for the joints given,
                                                or a humanoid's found from the mesh (up +Z,
                                                facing -Y, T or A pose)";

/// Runs `og model <command> …`: the report on stdout, a usage error on
/// stderr with exit status 2, a failure with 1.
pub fn run(argv: &[&str]) -> ExitCode {
    let argv: Vec<String> = argv.iter().map(|a| a.to_string()).collect();
    let result = match argv.split_first() {
        Some((cmd, rest)) => {
            let a = args::Args::new(rest);
            match cmd.as_str() {
                "build" => build::run(&a),
                "export" => export::run(&a),
                "measure" => measure::run(&a),
                "preview" => preview::run(&a),
                "import" => import::run(&a),
                "repair" => repair::run(&a),
                "decimate" => decimate::run(&a),
                "unwrap" => unwrap::run(&a),
                "place" => place::run(&a),
                "collide" => collide::run(&a),
                "compare" => compare::run(&a),
                "compress" => compress::run(&a),
                "process" => process::run(&a),
                "rig" => rig::run(&a),
                "bake" => bake::run(&a),
                "help" | "-h" | "--help" => {
                    println!("{USAGE}");
                    return ExitCode::SUCCESS;
                }
                other => Err(Failure::Usage(format!("no command {other:?}"))),
            }
        }
        None => Err(Failure::Usage("no command".into())),
    };
    let json = argv.iter().any(|a| a == "--json");
    match result {
        Err(Failure::Usage(why)) if json => {
            println!("{}", serde_json::json!({ "error": why }));
            ExitCode::from(2)
        }
        Err(Failure::Failed(why)) if json => {
            println!("{}", serde_json::json!({ "error": why }));
            ExitCode::from(1)
        }
        Ok(out) => {
            if !out.is_empty() {
                println!("{out}");
            }
            ExitCode::SUCCESS
        }
        Err(Failure::Usage(why)) => {
            eprintln!("og model: {why}\n{USAGE}");
            ExitCode::from(2)
        }
        Err(Failure::Failed(why)) => {
            eprintln!("og model: {why}");
            ExitCode::from(1)
        }
    }
}
