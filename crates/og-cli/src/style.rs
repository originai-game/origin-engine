//! CSS pack authoring commands are implemented in migration phase C.
use std::process::ExitCode;

pub fn run(args: &[&str]) -> ExitCode {
    if let ["check", rest @ ..] = args {
        return crate::style_check::run(rest);
    }
    eprintln!(
        "og style: style.ogt and its generator, upgrader, checker and atlas writer have been removed. A style pack is a directory of .css files and assets, loaded by directory. CSS pack authoring commands are not available yet (phase C)."
    );
    ExitCode::FAILURE
}
