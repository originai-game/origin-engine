//! `og vfx import <file.efkproj|file.efkefc> <content-dir> [--root <project>] [--hit]`:
//! an Effekseer effect converted to checked TypeScript declarations
//! (og-cook's `efk`): `<content-dir>/<name>.ts` (`file.vfx`) and `<name>/`
//! beside it (emitters, effect materials), with raw meshes and shared textures
//! under the matching project `assets/` path, with what
//! did not convert listed. A one-shot effect gets an `impact` where the
//! most starts at once; `--hit` adds a light and a shake there.

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str =
    "usage: og vfx import <file.efkproj|file.efkefc> <content-dir> [--root <project>] [--hit]";

fn go(args: &[&str]) -> Result<(), String> {
    let hit = args.contains(&"--hit");
    let args: Vec<&str> = args.iter().copied().filter(|a| *a != "--hit").collect();
    let (file, out, root) = match args[..] {
        [file, out] => (file, out, None),
        [file, out, "--root", root] => (file, out, Some(Path::new(root))),
        _ => return Err(USAGE.into()),
    };
    let i = og_cook::cook::efk::import(Path::new(file), Path::new(out), root, hit)?;
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
