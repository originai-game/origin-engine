//! A style pack is a directory with a pack.json (og-ui-next's
//! CONVENTIONS.md §Theme and packs).
use std::process::ExitCode;

const USAGE: &str = "usage: og style check-project [project] [art-contract options]\n       og style add <directory> [project]\n       og style list [project] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["new", ..] => Err::<(), String>("CSS packs are removed, so `og style new` is too; write the pack's pack.json (og api show screens, §3) or add one with `og style add <directory>`".into()),
        ["check-project", rest @ ..] => return crate::style_check::run(rest),
        ["add", rest @ ..] => return crate::ui::extensions::add(rest, "style"),
        ["list", rest @ ..] => return crate::ui::extensions::list(rest, Some("style")),
        ["--help"] | ["help"] => { println!("{USAGE}"); Ok(()) },
        ["upgrade" | "art" | "pack", ..] => Err("legacy style.ogt conversion and atlas packing are removed; name the art in the pack's pack.json (og api show screens, §3)".into()),
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
