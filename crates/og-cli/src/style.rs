//! CSS directories are the authoring format; no manifest or generated atlas.
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod generate;

const USAGE: &str = "usage: og style new <name> [--look <preset>] [--scheme dark|light] [--finish flat|gradient|glass|hologram|neon|paper|bevel|brush] [--out <directory>]\n       og style check <directory> [--json]\n       og style check-project [project] [art-contract options]\n       og style add <directory> [project]\n       og style list [project] [--json]";

fn files(
    directory: &Path,
    root: &Path,
    namespace: &str,
    sources: &mut Vec<(String, String)>,
) -> Result<(), String> {
    for entry in
        std::fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_symlink() {
            return Err(format!("{}: symlinks are not supported", path.display()));
        }
        if ty.is_dir() {
            files(&path, root, namespace, sources)?;
        } else if path.extension().is_some_and(|e| e == "css") {
            let id = format!(
                "{namespace}/{}",
                path.strip_prefix(root).unwrap().to_string_lossy()
            );
            sources.push((
                id,
                std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
            ));
        }
    }
    Ok(())
}

fn check(args: &[&str]) -> Result<(), String> {
    let (directory, json) = match args {
        [directory] => (Path::new(directory), false),
        [directory, "--json"] => (Path::new(directory), true),
        _ => return Err(USAGE.into()),
    };
    let asset_root = directory
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "assets"));
    let namespace = asset_root
        .map(|p| {
            directory
                .strip_prefix(p)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "pack".into());
    let prefix = format!("{namespace}/");
    let mut sources = Vec::new();
    files(directory, directory, &namespace, &mut sources)?;
    sources.sort();
    let mut findings = Vec::new();
    for (id, text) in &sources {
        if let Err(error) = og_ui::style_pack::StylePack::validate_source(id, text, sources.clone())
        {
            findings.push(error.replace(&prefix, &format!("{}/", directory.display())));
        }
        // TODO(ui-css-registry): validate built-in host/part selectors here using
        // og_ui::doc::schema's forthcoming registry, preserving file:line errors.
        // Author-defined custom parts remain valid; do not add a local name list.
        for url in og_ui::doc::at_rules::urls(text) {
            if url.starts_with('#') || url.starts_with("data:") {
                continue;
            }
            let resolved = og_ui::doc::at_rules::resolve(id, &url);
            let path = if !url.starts_with('/') {
                directory
                    .join(id.strip_prefix(&prefix).unwrap())
                    .parent()
                    .unwrap()
                    .join(&url)
            } else if let Some(relative) = resolved.strip_prefix("@styles/") {
                og_cook::cook::look::styles_dir()
                    .map(|p| p.join(relative))
                    .unwrap_or_default()
            } else {
                asset_root
                    .map(|p| p.join(&resolved))
                    .unwrap_or_else(|| PathBuf::from(&resolved))
            };
            if !path.is_file() {
                let line = text[..text.find(&url).unwrap_or(0)]
                    .bytes()
                    .filter(|b| *b == b'\n')
                    .count()
                    + 1;
                findings.push(format!(
                    "{}:{}: unresolved url({url})",
                    directory.join(id.strip_prefix(&prefix).unwrap()).display(),
                    line
                ));
            }
        }
    }
    let mut contrast = Vec::new();
    match og_ui::style_pack::StylePack::parse(&namespace, sources) {
        Ok(pack) => {
            for (scheme, theme) in [
                ("dark", og_ui::Theme::dark()),
                ("light", og_ui::Theme::light()),
            ] {
                let mut ui = og_ui::Ui::new(og_ui::UiDesc {
                    theme,
                    ..Default::default()
                });
                ui.set_style_pack(&pack)?;
                for (ink, ground) in [
                    ("ink", "panel"),
                    ("inkMuted", "panel"),
                    ("onPrimary", "primary"),
                ] {
                    if let (Some(a), Some(b)) = (ui.theme().color(ink), ui.theme().color(ground)) {
                        contrast.push(serde_json::json!({"scheme": scheme, "ink": ink, "ground": ground, "ratio": a.over(b).contrast(b)}));
                    }
                }
            }
        }
        Err(e) if findings.is_empty() => findings.push(e),
        Err(_) => (),
    }
    if json {
        println!(
            "{}",
            serde_json::json!({"errors": findings, "rootTokenContrast": contrast})
        );
    } else {
        for finding in &findings {
            println!("{finding}");
        }
        for fact in contrast {
            println!("root-token contrast {fact}");
        }
        println!(
            "Contrast describes root token pairs, not text over images, gradients or translucent backdrops. Use og ui sheet to inspect those surfaces."
        );
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(format!("{} error(s)", findings.len()))
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["new", rest @ ..] => generate::run(rest),
        ["check", rest @ ..] => check(rest),
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
