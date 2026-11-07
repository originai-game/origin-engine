//! `og vfx import <file.efkproj|file.efkefc> <out-dir> [--root <assets-dir>]`:
//! an Effekseer effect converted to the engine's own effect as text to edit
//! (og-cook's `efk`): `<out-dir>/<name>.ogt` (`type vfx`) and `<name>/`
//! beside it (emitters, effect materials, meshes, the textures it names),
//! with what did not convert listed.

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str =
    "usage: og vfx import <file.efkproj|file.efkefc> <out-dir> [--root <assets-dir>]";

fn go(args: &[&str]) -> Result<(), String> {
    let (file, out, root) = match args {
        [file, out] => (*file, *out, None),
        [file, out, "--root", root] => (*file, *out, Some(Path::new(*root))),
        _ => return Err(USAGE.into()),
    };
    let i = og_cook::cook::efk::import(Path::new(file), Path::new(out), root)?;
    for f in &i.findings {
        println!("finding: {f}");
    }
    println!(
        "og vfx import: {} files in {out}; og.load.vfx(world, \"{}\"); {} findings",
        i.written.len(),
        i.id,
        i.findings.len()
    );
    Ok(())
}

pub fn run(args: &[&str]) -> ExitCode {
    match go(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og vfx import: {e}");
            ExitCode::from(2)
        }
    }
}
