//! `shape import <mesh> [-o out]`: what a mesh file holds, as JSON: its
//! parts (name, material, triangles, UVs, bones), images, polygons
//! triangulated, and what the importer skipped. With `-o`, also writes
//! the parts placed as GLB (skins kept), OBJ or STL. Imported UVs and
//! textures are measured, not carried (`docs/research.md` §9.8).

use std::path::PathBuf;

use og_shape::tools::Import;

use super::args::{Args, IMPORT_OPTIONS};
use super::export::export;
use super::report::{imported, object};
use super::source::{Input, Local, Source, load_mesh};
use super::{Failure, Files};

/// What to import. The zero value has no input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImportDesc {
    /// The mesh file.
    pub input: Input,
    /// How to read the mesh file: its unit (OBJ) and axes.
    pub import: Import,
    /// Where to write the parts, if anywhere.
    pub out: Option<PathBuf>,
}

/// The import report; `"output"` is the file written, or null.
pub fn import(files: &dyn Files, d: &ImportDesc) -> Result<String, Failure> {
    let mesh = load_mesh(files, &d.input, &d.import)?;
    let report = imported(&mesh.format, &mesh.imported);
    let output = match &d.out {
        None => "null".to_string(),
        Some(out) => export(files, &Source::Mesh(mesh), out)?,
    };
    Ok(object(&[("import", report), ("output", output)]))
}

/// The command line.
pub fn run(a: &Args) -> Result<String, Failure> {
    let mut known = vec!["-o", "--out"];
    known.extend(IMPORT_OPTIONS);
    a.only(&known)?;
    import(
        &Local,
        &ImportDesc {
            input: a.input()?,
            import: a.import()?,
            out: a.out_opt()?,
        },
    )
}
