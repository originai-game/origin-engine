//! `og ui review <kit>` (docs/ui.md §6): every page of a kit's review
//! sheet × state fixture × style, at 1920×1080 and on a phone both ways,
//! 100 % and 150 % text, in the game's locale, Chinese, Arabic and the
//! pseudo-locale; labelled contact sheets and the measures as read.
use super::{Project, declared_screens, extensions, project, spawn};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og ui review <kit> [project] [--style <pack>]... [--states a,b] [--grid N] \
                     [--breakpoints ...] [--locales ...] [--scales ...] [--out DIR] [--json]";

/// A sheet screen's commands without its `style:` steps, and those styles.
fn split_style(commands: &str) -> (String, Vec<String>) {
    let mut styles = Vec::new();
    let rest: Vec<&str> = commands
        .split(';')
        .map(str::trim)
        .filter(|c| match c.strip_prefix("style:") {
            Some(s) => {
                styles.push(s.to_owned());
                false
            }
            None => !c.is_empty(),
        })
        .collect();
    (rest.join(";"), styles)
}

/// The pages a kit's sheet declares (each once, without its style) and the
/// styles its screens wear, in order.
fn pages(screens: &[(String, String)]) -> (Vec<(String, String)>, Vec<String>) {
    let mut pages: Vec<(String, String)> = Vec::new();
    let mut styles: Vec<String> = Vec::new();
    for (name, commands) in screens {
        let (rest, worn) = split_style(commands);
        for s in worn {
            if !styles.contains(&s) {
                styles.push(s);
            }
        }
        if pages.iter().any(|(_, c)| *c == rest) {
            continue;
        }
        let page = rest
            .split(';')
            .find_map(|c| c.strip_prefix("page:"))
            .map_or_else(|| name.clone(), str::to_owned);
        pages.push((page, rest));
    }
    (pages, styles)
}

fn review(args: &[&str]) -> Result<ExitCode, String> {
    let Some((&name, rest)) = args.split_first().filter(|(n, _)| !n.starts_with('-')) else {
        return Err(USAGE.into());
    };
    let mut root = Path::new(".");
    let mut out = None;
    let mut styles: Vec<String> = Vec::new();
    let mut forward: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--out" if i + 1 < rest.len() => {
                i += 1;
                out = Some(PathBuf::from(rest[i]));
            }
            "--style" if i + 1 < rest.len() => {
                i += 1;
                styles.push(rest[i].to_owned());
            }
            a @ ("--states" | "--grid" | "--breakpoints" | "--locales" | "--scales"
            | "--settle")
                if i + 1 < rest.len() =>
            {
                forward.extend([a.to_owned(), rest[i + 1].to_owned()]);
                i += 1;
            }
            "--json" => forward.push("--json".into()),
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`. {USAGE}")),
            a => root = Path::new(a),
        }
        i += 1;
    }
    let package = project::engine_package(Some(root)).ok_or(project::NO_ENGINE)?;
    let matches: Vec<_> = extensions::discover(root)?
        .into_iter()
        .filter(|e| e.manifest.name == name)
        .collect();
    if matches.len() != 1 {
        return Err(format!("`{name}` must identify one extension (og ui list)"));
    }
    let kit = &matches[0];
    if kit.manifest.kind == "style" {
        return Err(format!(
            "`{name}` is a style pack: review a kit in it with og ui review <kit> --style {name}"
        ));
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let out = cwd.join(out.unwrap_or_else(|| root.join("og-out/ui-review").join(name)));
    // Under the caller's worktree, so every review there shares its resident host.
    let worktree = cwd
        .ancestors()
        .find(|p| p.join(".git").exists())
        .unwrap_or(&cwd);
    let scratch = worktree.join(".og/review").join(name);
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    }
    for file in extensions::files(&kit.directory)? {
        let dest = scratch.join(&file);
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(kit.directory.join(&file), &dest).map_err(|e| e.to_string())?;
    }
    let demo = scratch.join("sheet");
    let screens = if demo.join("project.ogt").is_file() {
        declared_screens(&demo)?
    } else {
        Vec::new()
    };
    if screens.is_empty() {
        return Err(format!(
            "kit `{name}` needs sheet/project.ogt with fixed ui screen entries (docs/ui.md §6)"
        ));
    }
    extensions::install(kit, &demo)?;
    let (pages, worn) = pages(&screens);
    let styles = if styles.is_empty() { worn } else { styles };
    let p = Project::open(&demo)?;
    crate::cook::cook(&p.root, 0, false)?;
    let bundle = project::bundle_native(&p, &package)?;
    let mut args: Vec<String> = Vec::new();
    for (page, commands) in &pages {
        args.extend(["--screen".into(), format!("{page}={commands}")]);
    }
    if !styles.is_empty() {
        args.extend(["--styles".into(), styles.join(",")]);
    }
    if !forward.iter().any(|a| a == "--settle") {
        args.extend(["--settle".into(), "2".into()]);
    }
    args.extend(forward);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    spawn("review", &p, &package, &bundle, &out, &args)
}

pub(super) fn run(args: &[&str]) -> ExitCode {
    match review(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("og ui review: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_s_styled_screens_review_as_pages_crossed_with_their_styles() {
        let screens = [
            ("inventory-fantasy", "style:fantasy;page:inventory"),
            ("inventory-pixel", "style:pixel;page:inventory"),
            ("quests-fantasy", "style:fantasy; page:quests"),
            ("dialogue-cartoon", "page:dialogue;style:cartoon"),
            ("plain", "show:hud"),
        ]
        .map(|(n, c)| (n.to_owned(), c.to_owned()));
        let (pages, styles) = pages(&screens);
        assert_eq!(
            pages,
            [
                ("inventory", "page:inventory"),
                ("quests", "page:quests"),
                ("dialogue", "page:dialogue"),
                ("plain", "show:hud"),
            ]
            .map(|(n, c)| (n.to_owned(), c.to_owned()))
        );
        assert_eq!(styles, ["fantasy", "pixel", "cartoon"]);
    }
}
