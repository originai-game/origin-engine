//! CSS directories are the authoring format; no manifest or generated atlas.
use std::process::ExitCode;

mod generate;
#[cfg(test)]
mod literals;

const USAGE: &str = "usage: og style new <name> [--look <preset>] [--scheme dark|light] [--finish flat|gradient|glass|hologram|neon|paper|bevel|brush] [--out <directory>]\n       og style check-project [project] [art-contract options]\n       og style add <directory> [project]\n       og style list [project] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["new", rest @ ..] => generate::run(rest),
        ["check-project", rest @ ..] => return crate::style_check::run(rest),
        ["add", rest @ ..] => return crate::ui::extensions::add(rest, "style"),
        ["list", rest @ ..] => return crate::ui::extensions::list(rest, Some("style")),
        ["--help"] | ["help"] => { println!("{USAGE}"); Ok(()) },
        ["upgrade" | "art" | "pack", ..] => Err("legacy style.ogt conversion and atlas packing are removed; author CSS images, border-image and mask-image directly".into()),
        _ => Err(USAGE.into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og style: {e}");
            ExitCode::FAILURE
        }
    }
}
