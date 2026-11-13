use og_cook::cook::cache::{Store, default_path, max_bytes};
use std::process::ExitCode;

pub fn run(args: &[&str]) -> ExitCode {
    match gc(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("og cache: {error}");
            ExitCode::from(2)
        }
    }
}

fn gc(args: &[&str]) -> Result<(), String> {
    let Some(("gc", rest)) = args.split_first().map(|(first, rest)| (*first, rest)) else {
        return Err("usage: og cache gc [--max-bytes N] [--json]".into());
    };
    let mut cap = max_bytes();
    let mut json = false;
    let mut rest = rest.iter().copied();
    while let Some(arg) = rest.next() {
        match arg {
            "--max-bytes" => {
                cap = rest
                    .next()
                    .ok_or("--max-bytes needs an integer")?
                    .parse()
                    .map_err(|_| "--max-bytes needs a nonnegative integer")?
            }
            "--json" => json = true,
            _ => return Err(format!("unknown option {arg}")),
        }
    }
    let path = default_path().ok_or("cook cache is disabled or no home directory is available")?;
    let report = Store::new(path.clone())
        .gc(cap)
        .map_err(|e| e.to_string())?;
    if json {
        println!(
            "{}",
            serde_json::json!({ "path": path, "max_bytes": cap, "before": report.before, "after": report.after, "removed": report.removed })
        );
    } else {
        println!(
            "{}: {} → {} bytes; {} files removed (cap {cap})",
            path.display(),
            report.before,
            report.after,
            report.removed
        );
    }
    Ok(())
}
