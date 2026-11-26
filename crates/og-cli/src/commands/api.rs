//! `og api gen [--wiki]` and `og api check`: the IDL's generated files
//! (crates/og-api/README.md), including the wiki the knowledge package ships
//! (`packages/knowledge/wiki/`), and the look presets flattened by og-cook
//! for its art brief (`packages/knowledge/presets/`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// `og api gen [--wiki] [--root <dir>]` or `og api check [--root <dir>]`.
pub fn run(args: &[&str]) -> ExitCode {
    match args {
        ["gen", rest @ ..] => {
            let wiki = rest.contains(&"--wiki");
            let rest: Vec<&str> = rest.iter().copied().filter(|a| *a != "--wiki").collect();
            match root_option(&rest) {
                Ok(root) => api_gen(&root, wiki),
                Err(code) => code,
            }
        }
        [cmd @ ("find" | "show" | "list"), rest @ ..] => crate::api_lookup::run(cmd, rest),
        ["check", rest @ ..] => {
            let list = rest.contains(&"--budget");
            let rest: Vec<&str> = rest.iter().copied().filter(|a| *a != "--budget").collect();
            match root_option(&rest) {
                Ok(root) => {
                    let drift = api_check(&root);
                    let budget = api_budget(&root, list);
                    if drift == ExitCode::SUCCESS {
                        budget
                    } else {
                        drift
                    }
                }
                Err(code) => code,
            }
        }
        _ => {
            eprintln!(
                "og api: expected `og api find`, `og api show`, `og api list`, `og api gen` or `og api check`; run `og --help` for the options."
            );
            ExitCode::from(2)
        }
    }
}

/// `--root <dir>`, or the nearest ancestor of the current directory that holds the IDL.
fn root_option(rest: &[&str]) -> Result<PathBuf, ExitCode> {
    match rest {
        [] => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            crate::find_root(&cwd).ok_or_else(|| {
                eprintln!(
                    "og api: no {} above {}; run inside the engine repository or pass --root <dir>.",
                    og_idl::IDL_PATH,
                    cwd.display()
                );
                ExitCode::from(2)
            })
        }
        ["--root", dir] => Ok(PathBuf::from(dir)),
        _ => {
            eprintln!(
                "og api: unexpected arguments `{}`; the options are --root <dir> (and --wiki for `og api gen`).",
                rest.join(" ")
            );
            Err(ExitCode::from(2))
        }
    }
}

/// Writes every generated file, or with `wiki` only the wiki pages, and
/// removes wiki pages the IDL no longer generates (a deleted module's page).
fn api_gen(root: &Path, wiki: bool) -> ExitCode {
    // The flattened look presets the art brief reads.
    if !wiki {
        let presets = match presets(root) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("og api gen: {e}");
                return ExitCode::FAILURE;
            }
        };
        for (path, text) in &presets.files {
            if std::fs::read_to_string(root.join(path)).ok().as_deref() != Some(text.as_str()) {
                if let Err(e) = std::fs::create_dir_all(root.join(PRESETS_DIR))
                    .and_then(|()| std::fs::write(root.join(path), text))
                {
                    eprintln!("og api gen: cannot write {path}: {e}");
                    return ExitCode::FAILURE;
                }
                println!("wrote {path}");
            }
        }
        for path in &presets.stale {
            let stale = root.join(path);
            let removed = if stale.is_dir() {
                std::fs::remove_dir_all(&stale)
            } else {
                std::fs::remove_file(&stale)
            };
            if let Err(e) = removed {
                eprintln!("og api gen: cannot remove the stale preset {path}: {e}");
                return ExitCode::FAILURE;
            }
            println!("removed {path}");
        }
    }
    // The written guides (the component catalogue from the components' TSX,
    // the effect packs from their manifests, the cook grammar); the wiki reads them with the others.
    let guides = match written_guides(root) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("og api gen: {e}");
            return ExitCode::FAILURE;
        }
    };
    for (path, text) in guides {
        let file = root.join(path);
        if std::fs::read_to_string(&file).ok().as_deref() != Some(text.as_str()) {
            if let Err(e) = std::fs::write(&file, &text) {
                eprintln!("og api gen: cannot write {}: {e}", file.display());
                return ExitCode::FAILURE;
            }
            println!("wrote {path}");
        }
    }
    let api = match og_idl::read(&root.join(og_idl::IDL_PATH)) {
        Ok(api) => api,
        Err(e) => {
            eprint!("og api gen: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut files = if wiki {
        og_idl::generate_wiki(&api)
    } else {
        og_idl::generate(&api)
    };
    let write = |files: &[og_idl::Generated]| -> Result<Vec<String>, ExitCode> {
        og_idl::write(root, files)
            .map(|w| w.into_iter().map(str::to_string).collect())
            .map_err(|e| {
                eprintln!("og api gen: cannot write under {}: {e}", root.display());
                ExitCode::FAILURE
            })
    };
    // The IDL's files first: the UI declarations are emitted by type-checking
    // src/ui against them (types/index.d.ts, src/generated/calls.ts), so a
    // stale copy of either never blocks its own regeneration.
    let mut written = match write(&files) {
        Ok(w) => w,
        Err(code) => return code,
    };
    if !wiki {
        let ui = match ui_declarations(root) {
            Ok(ui) => ui,
            Err(e) => {
                eprintln!("og api gen: {e}");
                return ExitCode::FAILURE;
            }
        };
        match write(&ui) {
            Ok(w) => written.extend(w),
            Err(code) => return code,
        }
        files.extend(ui);
    }
    for path in &written {
        println!("wrote {path}");
    }
    // Unchanged files keep their modification time, so cargo rebuilds only
    // the crates whose generated Rust changed.
    println!(
        "{} of {} generated files unchanged",
        files.len() - written.len(),
        files.len()
    );
    // Pages the IDL no longer generates, at any depth (a deleted module's
    // page and its items' pages, a removed type's page).
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
    let mut found = Vec::new();
    walk(&root.join(og_idl::WIKI_DIR), &mut found);
    let mut stale: Vec<PathBuf> = found
        .into_iter()
        .filter(|p| {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/");
            !files.iter().any(|f| f.path == rel)
        })
        .collect();
    stale.sort();
    for p in stale {
        if let Err(e) = std::fs::remove_file(&p) {
            eprintln!(
                "og api gen: cannot remove the stale page {}: {e}",
                p.display()
            );
            return ExitCode::FAILURE;
        }
        println!("removed {}", p.display());
    }
    ExitCode::SUCCESS
}

/// `@origingame/engine/ui`'s declarations (`og_idl::ui_declarations`), emitted
/// with the tsrs `og check` uses.
fn ui_declarations(root: &Path) -> Result<Vec<og_idl::Generated>, String> {
    let tsrs = crate::commands::check::locate_tsrs(Some(&root.join("packages/engine")))?
        .ok_or_else(|| crate::commands::check::NO_TSRS.to_string())?;
    og_idl::ui_declarations(root, &tsrs.path)
}

/// Where `og api gen` writes the flattened look presets: the knowledge
/// package's `presets/` (og-cook's `look::preset_documents`).
const PRESETS_DIR: &str = "packages/knowledge/presets";

/// The preset files `og api gen` writes, as (path, text), and the files
/// under [`PRESETS_DIR`] it no longer writes.
struct Presets {
    files: Vec<(String, String)>,
    stale: Vec<String>,
}

fn presets(root: &Path) -> Result<Presets, String> {
    let files: Vec<(String, String)> = og_cook::cook::look::preset_documents()?
        .into_iter()
        .map(|(name, text)| (format!("{PRESETS_DIR}/{name}"), text))
        .collect();
    let mut stale: Vec<String> = std::fs::read_dir(root.join(PRESETS_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| format!("{PRESETS_DIR}/{}", e.file_name().to_string_lossy()))
        .filter(|p| !files.iter().any(|(f, _)| f == p))
        .collect();
    stale.sort();
    Ok(Presets { files, stale })
}

/// The guides `og api gen` writes from other sources than the IDL, as (path, text).
fn written_guides(root: &Path) -> Result<[(&'static str, String); 7], String> {
    Ok([
        (
            crate::ui_components::PATH,
            crate::ui_components::markdown(root),
        ),
        (
            crate::ui_api_docs::UI_TSX_PATH,
            crate::ui_api_docs::ui_tsx_markdown(root)?,
        ),
        (
            crate::ui_api_docs::THEME_TOKENS_PATH,
            crate::ui_api_docs::theme_tokens_markdown(root)?,
        ),
        (
            crate::ui_api_docs::PLAYTEST_PATH,
            crate::ui_api_docs::playtest_markdown(root)?,
        ),
        (
            crate::vfx_catalogue::PATH,
            crate::vfx_catalogue::markdown(root),
        ),
        (og_cook::grammar::PATH, og_cook::grammar::markdown()),
        (
            og_cook::cook::file_types::PATH,
            og_cook::cook::file_types::markdown()?,
        ),
    ])
}

/// The description budget (`og_idl::budget`): a summary, or with `--budget`
/// every description over it; a failure when more are over than the backlog.
fn api_budget(root: &Path, list: bool) -> ExitCode {
    use og_idl::budget::{BACKLOG, FIELD, ITEM, over};
    let Ok(api) = og_idl::read(&root.join(og_idl::IDL_PATH)) else {
        return ExitCode::FAILURE;
    };
    let over = over(&api);
    let items = over.iter().filter(|o| o.budget == ITEM).count();
    if list {
        for o in &over {
            let (part, line) = api.locate(o.line);
            println!(
                "{part}:{line}: {} has {} characters of {}",
                o.entry, o.chars, o.budget
            );
        }
    }
    println!(
        "og api check: {} description(s) over budget ({items} over {ITEM} characters for modules, functions and types; {} over {FIELD} for fields, parameters and the rest), backlog {BACKLOG}{}",
        over.len(),
        over.len() - items,
        if list || over.is_empty() {
            ""
        } else {
            "; `og api check --budget` lists them"
        }
    );
    if over.len() > BACKLOG {
        eprintln!(
            "og api check: more descriptions are over budget than the backlog of {BACKLOG}: move workflow, sizes, timings and cross-member caveats to a guide and point at it with `guide \"<guide> §N\"` (crates/og-idl/src/budget.rs states the rule)"
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn api_check(root: &Path) -> ExitCode {
    let presets = match presets(root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og api check: {e}");
            return ExitCode::FAILURE;
        }
    };
    let guides = match written_guides(root) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("og api check: {e}");
            return ExitCode::FAILURE;
        }
    };
    let written = guides
        .into_iter()
        .map(|(path, text)| (path.to_string(), text))
        .chain(presets.files);
    let stale: Vec<og_idl::Drift> = written
        .filter(|(path, text)| {
            std::fs::read_to_string(root.join(path)).ok().as_deref() != Some(text.as_str())
        })
        .map(|(path, _)| og_idl::Drift::Differs { path, line: 1 })
        .chain(presets.stale.into_iter().map(og_idl::Drift::Unexpected))
        .collect();
    let ui = match ui_declarations(root) {
        Ok(ui) => ui,
        Err(e) => {
            eprintln!("og api check: {e}");
            return ExitCode::FAILURE;
        }
    };
    match og_idl::check(root, &ui).map(|d| stale.into_iter().chain(d).collect::<Vec<_>>()) {
        Err(e) => {
            eprint!("og api check: {e}");
            ExitCode::FAILURE
        }
        Ok(drift) if drift.is_empty() => {
            println!(
                "og api check: the generated files match {}.",
                og_idl::IDL_PATH
            );
            ExitCode::SUCCESS
        }
        Ok(drift) => {
            eprintln!(
                "og api check: {} file(s) differ from what {} generates:",
                drift.len(),
                og_idl::IDL_PATH
            );
            for d in &drift {
                eprintln!("  {d}");
            }
            ExitCode::FAILURE
        }
    }
}
