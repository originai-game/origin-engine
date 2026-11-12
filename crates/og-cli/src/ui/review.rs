//! `og ui review <group>` (docs/ui.md §6): every page of a component
//! group's review sheet (`sheets/<group>` of the engine package, or a
//! sheet directory) × state fixture × style, at 1920×1080 and on a phone both ways,
//! 100 % and 150 % text, in the game's locale, Chinese, Arabic and the
//! pseudo-locale; labelled contact sheets and the measures as read.
use super::{Project, declared_screens, extensions, project, spawn};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og ui review <group|sheet-directory> [project] [--style <pack>]... [--states a,b] [--grid N] \
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

/// The font packs of the locales the sheet has strings for and those
/// `--locales` names, installed into the sheet as `og create --lang`
/// installs a game's (docs/ui.md §3.4): without them a Japanese or Arabic
/// cell draws its translated labels in no face at all.
pub(super) fn install_font_packs(
    root: &Path,
    demo: &Path,
    forward: &[String],
) -> Result<(), String> {
    let mut locales: Vec<String> = std::fs::read_dir(demo.join("assets/strings"))
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let name = e.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".ogt").map(str::to_owned)
        })
        .collect();
    if let Some(i) = forward.iter().position(|a| a == "--locales") {
        locales.extend(forward[i + 1].split(',').map(str::trim).map(str::to_owned));
    }
    let packs: Vec<_> = extensions::discover(root)?
        .into_iter()
        .filter(|e| e.manifest.kind == "font")
        .collect();
    let tags: Vec<String> = packs
        .iter()
        .flat_map(|p| p.manifest.languages.clone().unwrap_or_default())
        .collect();
    let mut chosen: Vec<&extensions::Extension> = Vec::new();
    for locale in locales
        .iter()
        .filter(|l| !matches!(l.as_str(), "" | "game" | "pseudo"))
    {
        let Some(tag) = og_i18n::negotiate(&[locale.as_str()], &tags) else {
            continue;
        };
        if let Some(pack) = packs.iter().find(|p| {
            p.manifest
                .languages
                .as_deref()
                .unwrap_or_default()
                .iter()
                .any(|l| l == tag)
        }) && !chosen.iter().any(|c| c.manifest.name == pack.manifest.name)
        {
            chosen.push(pack);
        }
    }
    for pack in chosen {
        if !demo.join("assets/fonts").join(&pack.manifest.name).exists() {
            extensions::install(pack, demo)?;
        }
    }
    Ok(())
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
    let group = super::sheet::group_sheet(&package, name);
    let matches: Vec<_> = if group.is_some() {
        Vec::new()
    } else if Path::new(name).is_dir() {
        vec![extensions::read(Path::new(name), false)?]
    } else {
        extensions::discover(root)?
            .into_iter()
            .filter(|e| e.manifest.name == name)
            .collect()
    };
    if group.is_none() && matches.len() != 1 {
        return Err(format!(
            "`{name}` must identify one pack (og ui list), component group (a sheets/<group> of the engine package) or sheet directory"
        ));
    }
    if matches.first().is_some_and(|e| e.manifest.kind == "style") {
        super::sheet::sheet(args, true)?;
        return Ok(ExitCode::SUCCESS);
    }
    let name = match (&group, matches.first()) {
        (Some(dir), _) => dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sheet")
            .to_owned(),
        (None, Some(e)) => e.manifest.name.clone(),
        (None, None) => unreachable!("checked above"),
    };
    let name = name.as_str();
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
    // A group's sheet uses the built-in components by name; a pack's sheet
    // is its `sheet/`, with the pack installed as a game installs it.
    let source = group
        .clone()
        .unwrap_or_else(|| matches[0].directory.clone());
    for file in extensions::files(&source)? {
        let dest = scratch.join(&file);
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(source.join(&file), &dest).map_err(|e| e.to_string())?;
    }
    let demo = if group.is_some() {
        scratch.clone()
    } else {
        scratch.join("sheet")
    };
    let screens = if demo.join("project.ogt").is_file() {
        declared_screens(&demo)?
    } else {
        Vec::new()
    };
    if screens.is_empty() {
        return Err(format!(
            "`{name}` needs a sheet project.ogt with fixed ui screen entries (docs/ui.md §6)"
        ));
    }
    if group.is_none() {
        extensions::install(&matches[0], &demo)?;
    }
    install_font_packs(root, &demo, &forward)?;
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
