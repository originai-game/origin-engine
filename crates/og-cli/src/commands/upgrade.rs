//! `og upgrade [project] [--to <version>] [--dry-run] [--json]`: moves a
//! game to another engine version (docs/release.md §3).
//!
//! The installed engine package's `changelog.json` says what each version
//! between the pin (`project.ogt`'s `engine`) and the target (`--to`, else
//! the installed version) changed: `og upgrade` writes the target into
//! project.ogt's `engine` line (the one pin: a game has no package.json
//! dependency on the engine since 0.7.8, docs/release.md §1), applies the
//! mechanical rewrites (`codemod`) of the breaking entries it crosses to the
//! game's sources (`src/`, `assets/` text), and lists every place the other
//! breaking entries' identifiers (`find`) appear, with the change and its
//! fix, as `og migrate three` lists what it cannot rewrite. A breaking entry
//! without identifiers is listed under `review`. It never downgrades; the og
//! launcher's `og upgrade` installs the target version and runs this with
//! it. A dev build `dev.<date>.<n>` stands after the release it is based on
//! (never above a release by its numbers), and a release pin moves onto a
//! dev build only when `--to` names it; then the build's `unreleased` entry
//! is crossed as its own.
//!
//! Exit status: 0 done (nothing left to edit by hand), 3 done with places
//! to edit by hand (`manual`), 1 failed (nothing written), 2 usage.

use crate::project::{self, Project};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og upgrade [project] [--to <version>] [--dry-run] [--json]";

/// The exit status of an upgrade that left places to edit by hand.
const NEEDS_EDITS: u8 = 3;

pub fn run(args: &[&str]) -> ExitCode {
    let mut dir = ".".to_string();
    let mut to: Option<String> = None;
    let (mut json_out, mut dry) = (false, false);
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => json_out = true,
            "--dry-run" => dry = true,
            "--to" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og upgrade: --to needs a version. {USAGE}");
                    return ExitCode::from(2);
                };
                to = Some(v.to_string());
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og upgrade: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => dir = a.to_string(),
        }
        i += 1;
    }
    let report = upgrade(Path::new(&dir), to.as_deref(), dry);
    let status = report["status"].as_str().unwrap_or("failed").to_string();
    if json_out {
        println!("{report}");
    } else {
        print_text(&report, dry);
    }
    match status.as_str() {
        "done" => ExitCode::SUCCESS,
        "manual" => ExitCode::from(NEEDS_EDITS),
        _ => ExitCode::FAILURE,
    }
}

fn failed(reason: String, extra: Value) -> Value {
    let mut v = json!({ "status": "failed", "reason": reason });
    if let (Some(o), Value::Object(e)) = (v.as_object_mut(), extra) {
        o.extend(e);
    }
    v
}

/// The whole upgrade as its report (written unless `dry`).
fn upgrade(dir: &Path, to: Option<&str>, dry: bool) -> Value {
    let p = match Project::open(dir) {
        Ok(p) if p.manifest => p,
        Ok(_) => return failed(format!("{}: no project.ogt", dir.display()), json!({})),
        Err(e) => return failed(e, json!({})),
    };
    let base = json!({ "project": p.root.display().to_string() });
    let Some(from) = p.engine.clone() else {
        return failed(
            "project.ogt has no `engine \"<version>\"` line: add the version the game is written against first".into(),
            base,
        );
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        return failed(project::NO_ENGINE.into(), base);
    };
    let installed = project::package_version(&package).unwrap_or_default();
    let log: Value = match std::fs::read_to_string(package.join("changelog.json"))
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            return failed(
                format!(
                    "the engine package at {} has no readable changelog.json ({e}); run og upgrade with an engine of 0.6.2 or later (the og launcher's og upgrade does)",
                    package.display()
                ),
                base,
            );
        }
    };
    if to.is_none() && project::is_dev(&installed) && !project::is_dev(&from) {
        return failed(
            format!(
                "the installed engine is the dev build {installed}: og upgrade moves a release pin onto a dev build only when --to names it (og upgrade --to {installed}); og upgrade --to <release> moves it to a release"
            ),
            base,
        );
    }
    let to = to.map(str::to_string).unwrap_or_else(|| installed.clone());
    let at = |v: &str| project::order(v, project::base_of(v, &package).as_deref());
    let (Some(f), Some(t)) = (at(&from), at(&to)) else {
        return failed(
            format!("not a version, or a dev build whose base release is unknown: {from} or {to}"),
            base,
        );
    };
    let versions = log["versions"].as_array().cloned().unwrap_or_default();
    let known = |v: &str| versions.iter().any(|e| e["version"] == v);
    // The installed dev build: its own changes are the changelog's unreleased.
    let own = (to == installed && project::is_dev(&to)).then(|| {
        let mut e = log["unreleased"].clone();
        e["version"] = json!(to);
        e
    });
    if !known(&to) && own.is_none() {
        return failed(
            format!(
                "{to} is not in the installed engine's changelog (the installed engine is {installed}): run og upgrade --to {to} through the og launcher, which installs {to} and runs its og upgrade"
            ),
            base,
        );
    }
    if t < f {
        return failed(
            format!(
                "{to} is older than the pinned {from}: og upgrade never downgrades (revert the upgrade's commit instead)"
            ),
            base,
        );
    }
    // The versions crossed, oldest first: after the pin, up to the target.
    let mut crossed: Vec<&Value> = versions
        .iter()
        .filter(|e| {
            e["version"]
                .as_str()
                .and_then(|v| project::order(v, None))
                .is_some_and(|v| v > f && v <= t)
        })
        .collect();
    crossed.reverse();
    if f < t {
        crossed.extend(own.as_ref());
    }
    let kind = if f == t {
        "none"
    } else if f.release.0 == t.release.0 && f.release.1 == t.release.1 {
        "patch"
    } else {
        "minor"
    };
    let simulation = if crossed.iter().any(|e| e["simulation"] != "unchanged") {
        "changed"
    } else {
        "unchanged"
    };
    let sources = game_sources(&p.root);
    let mut texts: Vec<(PathBuf, String, bool)> = sources
        .iter()
        .filter_map(|f| {
            std::fs::read_to_string(f)
                .ok()
                .map(|t| (f.clone(), t, false))
        })
        .collect();
    let rel = |f: &Path| {
        f.strip_prefix(&p.root)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let (mut codemods, mut manual, mut review) = (Vec::new(), Vec::new(), Vec::new());
    // The new UI core has no `uiCore` switch: its line leaves project.ogt.
    let mut drop_ui_core = false;
    // project.ogt's `profile` line becomes the creation form.
    let mut profile_form: Option<(&str, &Value)> = None;
    // The creation form `film` becomes `3d` (its genre stays).
    let mut film_form: Option<(&str, &Value)> = None;
    // The game's tsconfig.json, when a codemod rewrote it.
    let mut tsconfig: Option<(PathBuf, String)> = None;
    for e in &crossed {
        let version = e["version"].as_str().unwrap_or("");
        for b in e["breaking"].as_array().into_iter().flatten() {
            let member = b["member"].as_str().unwrap_or("");
            if b["codemod"]["kind"] == "particles-lit-mesh-blend" {
                // Reported, not rewritten: the new default is the fix, and
                // an emitter that meant to glow says `blend additive`.
                for (file, text, _) in &texts {
                    if file.extension().is_some_and(|e| e == "ogt")
                        && let Some((line, code)) = lit_mesh_without_blend(text)
                    {
                        manual.push(json!({
                            "version": version, "member": member, "change": b["change"], "fix": b["fix"],
                            "file": rel(file), "line": line, "text": code,
                        }));
                    }
                }
                continue;
            }
            if b["codemod"]["kind"] == "flow-rename" {
                for (file, text, touched) in &mut texts {
                    if let Some((upgraded, renamed)) = flow_rename(text) {
                        *text = upgraded;
                        *touched = true;
                        for (from, to, n) in renamed {
                            codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": from, "to": to, "count": n }));
                        }
                    }
                }
                continue;
            }
            if b["codemod"]["kind"] == "vfx-glow" {
                for (file, text, touched) in &mut texts {
                    if file.extension().is_some_and(|e| e == "ogt")
                        && let Some((upgraded, n)) = vfx_glow(text)
                    {
                        *text = upgraded;
                        *touched = true;
                        codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": "flash", "to": "glow", "count": n }));
                    }
                }
                continue;
            }
            if b["codemod"]["kind"] == "ui-core" {
                drop_ui_core = true;
                for (file, text, touched) in &mut texts {
                    if !file.extension().is_some_and(|e| e == "svelte") {
                        continue;
                    }
                    let Some((upgraded, rewrites, lines)) = ui_core(text) else {
                        continue;
                    };
                    let mut counted: Vec<(Rewrite, usize)> = Vec::new();
                    for r in rewrites {
                        match counted.iter_mut().find(|(k, _)| *k == r) {
                            Some((_, n)) => *n += 1,
                            None => counted.push((r, 1)),
                        }
                    }
                    for ((from_s, to_s), n) in counted {
                        codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": from_s, "to": to_s, "count": n }));
                    }
                    for line in lines {
                        manual.push(json!({
                            "version": version, "member": member, "change": b["change"], "fix": b["fix"],
                            "file": rel(file), "line": line, "text": upgraded.lines().nth(line - 1).unwrap_or("").trim(),
                        }));
                    }
                    *touched |= upgraded != *text;
                    *text = upgraded;
                }
                continue;
            }
            if b["codemod"]["kind"] == "raw-pads" {
                for (file, text, touched) in &mut texts {
                    let Some((upgraded, n, left)) = raw_pads(text) else {
                        continue;
                    };
                    if n > 0 {
                        codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": "gamepad buttons by name", "to": "raw button values", "count": n }));
                    }
                    for line in left {
                        manual.push(json!({
                            "version": version, "member": member, "change": b["change"], "fix": b["fix"],
                            "file": rel(file), "line": line, "text": upgraded.lines().nth(line - 1).unwrap_or("").trim(),
                        }));
                    }
                    *touched |= upgraded != *text;
                    *text = upgraded;
                }
                continue;
            }
            if b["codemod"]["kind"] == "film-form" {
                film_form = Some((version, b));
                continue;
            }
            if b["codemod"]["kind"] == "profile-form" {
                profile_form = Some((version, b));
                continue;
            }
            if b["codemod"]["kind"] == "engine-package" {
                let file = p.root.join("tsconfig.json");
                let Ok(text) = std::fs::read_to_string(&file) else {
                    continue;
                };
                match engine_package_tsconfig(&text) {
                    None => {}
                    Some(Ok(upgraded)) => {
                        codemods.push(json!({ "version": version, "member": member, "file": "tsconfig.json", "from": "paths into .og/engine", "to": "the engine package", "count": 1 }));
                        tsconfig = Some((file, upgraded));
                    }
                    Some(Err(line)) => manual.push(json!({
                        "version": version, "member": member, "change": b["change"], "fix": b["fix"],
                        "file": "tsconfig.json", "line": line, "text": text.lines().nth(line - 1).unwrap_or("").trim(),
                    })),
                }
                continue;
            }
            if b["codemod"]["kind"] == "font-atlas-keys" {
                for (file, text, touched) in &mut texts {
                    let Some((upgraded, n)) = font_atlas_keys(text) else {
                        continue;
                    };
                    codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": "font atlas settings", "to": "none (the file ships whole)", "count": n }));
                    *touched = true;
                    *text = upgraded;
                }
                continue;
            }
            if b["codemod"]["kind"] == "light-mobility" {
                for (file, text, touched) in &mut texts {
                    let Some((upgraded, n)) = light_mobility(text) else {
                        continue;
                    };
                    codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": "static", "to": "mobility", "count": n }));
                    *touched = true;
                    *text = upgraded;
                }
                continue;
            }
            if b["codemod"]["kind"] == "ui-kits" {
                review.push(json!({
                    "version": version, "member": member,
                    "change": "The ui-kits codemod and its target kits were removed.",
                    "fix": "Compose the UI with public core controls; no automatic kit rewrite is available."
                }));
                continue;
            }
            if let (Some(from_s), Some(to_s)) =
                (b["codemod"]["from"].as_str(), b["codemod"]["to"].as_str())
            {
                for (file, text, touched) in &mut texts {
                    let n = text.matches(from_s).count();
                    if n > 0 {
                        *text = text.replace(from_s, to_s);
                        *touched = true;
                        codemods.push(json!({ "version": version, "member": member, "file": rel(file), "from": from_s, "to": to_s, "count": n }));
                    }
                }
                continue;
            }
            let finds: Vec<&str> = b["find"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            if finds.is_empty() {
                review.push(json!({ "version": version, "member": member, "change": b["change"], "fix": b["fix"] }));
                continue;
            }
            for (file, text, _) in &texts {
                for (n, line) in text.lines().enumerate() {
                    if finds.iter().any(|w| line.contains(w)) {
                        manual.push(json!({
                            "version": version, "member": member, "change": b["change"], "fix": b["fix"],
                            "file": rel(file), "line": n + 1, "text": line.trim(),
                        }));
                    }
                }
            }
        }
    }
    // The pin and the dependency.
    let mut edited = Vec::new();
    let mut writes: Vec<(PathBuf, String)> = Vec::new();
    if kind != "none" {
        let manifest = p.root.join("project.ogt");
        match std::fs::read_to_string(&manifest) {
            Ok(text) => match set_engine_line(&text, &to)
                .map(|t| if drop_ui_core { without_ui_core(&t) } else { t })
                .map(|t| {
                    if profile_form.is_some() {
                        form_of_profile(&t)
                    } else {
                        t
                    }
                }) {
                Some(t) => {
                    let t = match film_form.and_then(|f| film_to_3d(&t).map(|u| (f, u))) {
                        Some(((version, b), upgraded)) => {
                            codemods.push(json!({ "version": version, "member": b["member"], "file": "project.ogt", "from": "form \"film\"", "to": "form \"3d\"", "count": 1 }));
                            upgraded
                        }
                        None => t,
                    };
                    if let Some((version, b)) = profile_form
                        && let Some(line) = t.lines().position(|l| l.starts_with("profile "))
                    {
                        manual.push(json!({
                            "version": version, "member": b["member"], "change": b["change"], "fix": b["fix"],
                            "file": "project.ogt", "line": line + 1, "text": t.lines().nth(line).unwrap_or("").trim(),
                        }));
                    }
                    writes.push((manifest, t))
                }
                None => {
                    return failed(
                        "project.ogt: the engine line could not be rewritten".into(),
                        base,
                    );
                }
            },
            Err(e) => return failed(format!("project.ogt: {e}"), base),
        }
    }
    writes.extend(tsconfig);
    writes.extend(
        texts
            .into_iter()
            .filter(|(_, _, touched)| *touched)
            .map(|(f, t, _)| (f, t)),
    );
    for (file, text) in &writes {
        if !dry && let Err(e) = std::fs::write(file, text) {
            return failed(format!("{}: {e}", file.display()), base);
        }
        edited.push(rel(file));
    }
    let status = if manual.is_empty() { "done" } else { "manual" };
    json!({
        "status": status,
        "project": p.root.display().to_string(),
        "from": from,
        "to": to,
        "installed": installed,
        "kind": kind,
        "simulation": simulation,
        "versions": crossed.iter().map(|e| e["version"].clone()).collect::<Vec<_>>(),
        "dryRun": dry,
        "edited": edited,
        "codemods": codemods,
        "manual": manual,
        "review": review,
    })
}

/// A particle emitter (`.ogt` of `type particles`) drawn as lit meshes
/// with no `blend` line, which blended additive until lit meshes took
/// alpha as their default. Its `render mesh` line (1-based) and its text.
fn lit_mesh_without_blend(text: &str) -> Option<(usize, &str)> {
    let (head, _) = text.split_once("\n---")?;
    if !head.lines().any(|l| l.trim() == "type particles") {
        return None;
    }
    let body_at = head.lines().count() + 1;
    let word = |l: &str| l.split('#').next().unwrap_or("").trim().to_string();
    let mut mesh = None;
    let mut lit = false;
    for (n, l) in text.lines().enumerate().skip(body_at) {
        let w = word(l);
        if w.starts_with("blend") {
            return None;
        }
        if w.starts_with("render mesh") {
            mesh = Some((n + 1, l.trim()));
        }
        if w == "lit true" {
            lit = true;
        }
    }
    mesh.filter(|_| lit)
}

/// A composed effect (`.ogt` of `type vfx`) at version 1 rewritten to
/// version 2: its `version` line, and each `flash` element (a line of the
/// body at the top level, not inside an element's block) renamed `glow`.
/// With the number of elements renamed; `None` for any other file and
/// for one with no `flash`.
fn vfx_glow(text: &str) -> Option<(String, usize)> {
    let (head, _) = text.split_once("\n---")?;
    let word = |l: &str| l.split('#').next().unwrap_or("").trim().to_string();
    if !head.lines().any(|l| word(l) == "type vfx") || !head.lines().any(|l| word(l) == "version 1")
    {
        return None;
    }
    let body_at = head.lines().count() + 1;
    let (mut out, mut n, mut depth) = (String::with_capacity(text.len()), 0, 0usize);
    for (i, l) in text.split_inclusive('\n').enumerate() {
        let w = word(l);
        if i < body_at {
            if w == "version 1" {
                out.push_str(&l.replacen("1", "2", 1));
                continue;
            }
        } else if depth == 0 && (w == "flash" || w.starts_with("flash ") || w.starts_with("flash{"))
        {
            let at = l.find("flash").expect("the word");
            out.push_str(&l[..at]);
            out.push_str("glow");
            out.push_str(&l[at + "flash".len()..]);
            n += 1;
            depth += w.matches('{').count();
            depth = depth.saturating_sub(w.matches('}').count());
            continue;
        }
        if i >= body_at {
            depth += w.matches('{').count();
            depth = depth.saturating_sub(w.matches('}').count());
        }
        out.push_str(l);
    }
    // A version 1 file without a flash reads the same at version 1.
    (n > 0).then_some((out, n))
}

/// How a name og.film gave must sit in the text to be one of its names.
#[derive(Clone, Copy)]
enum Sits {
    /// A whole name: no identifier character on either side
    /// (`FilmView`, never `MyFilmView` or `FilmViewProps`).
    Whole,
    /// The start of a name: no identifier character or `-` before it
    /// (`og-film`, `og-film-chapter`).
    Start,
    /// A whole quoted string (`"film.open"`), not an expression that
    /// reads a `film` of the game's.
    Quoted,
    /// A screen's name: a whole quoted string, the head of its strings'
    /// keys (`screen.film-choices.label`) or of its parts
    /// (`::part(film-choices.shade)`); never a file's path.
    Screen,
}

/// og.film's names as og.flow's (the flow-rename codemod).
const FLOW_NAMES: &[(&str, &str, Sits)] = &[
    ("og.film", "og.flow", Sits::Whole),
    ("og.Film", "og.Flow", Sits::Whole),
    ("FilmDesc", "FlowStartDesc", Sits::Whole),
    ("FilmInfo", "FlowInfo", Sits::Whole),
    ("FilmDeck", "FlowDeck", Sits::Whole),
    ("FilmHotspot", "FlowHotspot", Sits::Whole),
    ("FilmView", "FlowView", Sits::Whole),
    ("FilmChart", "FlowChart", Sits::Whole),
    ("FilmChartNode", "FlowChartNode", Sits::Whole),
    ("FilmChartLink", "FlowChartLink", Sits::Whole),
    ("FilmSegment", "FlowSegment", Sits::Whole),
    ("FilmChoice", "FlowChoice", Sits::Whole),
    ("FilmChapter", "FlowChapter", Sits::Whole),
    ("FilmEnded", "FlowEnded", Sits::Whole),
    ("FilmChoices", "FlowChoices", Sits::Whole),
    ("FilmChoicesProps", "FlowChoicesProps", Sits::Whole),
    ("FilmChaptersScreen", "FlowChaptersScreen", Sits::Whole),
    (
        "FilmChaptersScreenProps",
        "FlowChaptersScreenProps",
        Sits::Whole,
    ),
    ("FilmFlowchartScreen", "FlowChartScreen", Sits::Whole),
    (
        "FilmFlowchartScreenProps",
        "FlowChartScreenProps",
        Sits::Whole,
    ),
    ("FilmGalleryScreen", "FlowGalleryScreen", Sits::Whole),
    (
        "FilmGalleryScreenProps",
        "FlowGalleryScreenProps",
        Sits::Whole,
    ),
    ("film:choose", "flow:choose", Sits::Whole),
    ("film:play", "flow:play", Sits::Whole),
    ("film:pause", "flow:pause", Sits::Whole),
    ("film:skip", "flow:skip", Sits::Whole),
    ("og-film", "og-flow", Sits::Start),
    ("film.open", "flow.open", Sits::Quoted),
    ("film.lock", "flow.lock", Sits::Quoted),
    ("film.cut", "flow.cut", Sits::Quoted),
    ("film-choices", "flow-choices", Sits::Screen),
    ("film-chapters", "flow-chapters", Sits::Screen),
    ("film-flowchart", "flow-chart", Sits::Screen),
    ("film-gallery", "flow-gallery", Sits::Screen),
];

/// A name [`flow_rename`] rewrote: as it was, as it is, and how often.
type FlowRenamed = (&'static str, &'static str, usize);

/// og.film became og.flow: a game's text with every name of [`FLOW_NAMES`]
/// as it is now, and how often each was rewritten; `None` for a text that
/// names none. The game's own names stay (`FilmRef`, a `film` variable, a
/// file `film-view.tsx`), and so does the `film` field of og.FlowView,
/// og.FlowDeck and the events, which the type check names.
fn flow_rename(text: &str) -> Option<(String, Vec<FlowRenamed>)> {
    let ident = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '$';
    let quote = |c: char| matches!(c, '"' | '\'' | '`');
    let mut out = String::with_capacity(text.len());
    let mut counts = vec![0usize; FLOW_NAMES.len()];
    let mut i = 0;
    while let Some(c) = text[i..].chars().next() {
        let (head, rest) = text.split_at(i);
        let before = head.chars().next_back();
        let hit = FLOW_NAMES
            .iter()
            .enumerate()
            .filter(|(_, (from, _, sits))| {
                let Some(after) = rest.strip_prefix(from) else {
                    return false;
                };
                let after = after.chars().next();
                let quoted = before.is_some_and(quote) && after == before;
                match sits {
                    Sits::Whole => !before.is_some_and(ident) && !after.is_some_and(ident),
                    Sits::Start => !before.is_some_and(|c| ident(c) || c == '-'),
                    Sits::Quoted => quoted,
                    Sits::Screen => {
                        quoted
                            || (after == Some('.')
                                && (head.ends_with("screen.") || head.ends_with("part(")))
                    }
                }
            })
            .max_by_key(|(_, (from, ..))| from.len());
        match hit {
            Some((k, (from, to, _))) => {
                out.push_str(to);
                counts[k] += 1;
                i += from.len();
            }
            None => {
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
    let renamed: Vec<_> = FLOW_NAMES
        .iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|((from, to, _), n)| (*from, *to, n))
        .collect();
    (!renamed.is_empty()).then_some((out, renamed))
}

/// The font atlas's settings dropped from a font import's `font { … }`
/// block (and a `.riv` import's `font <name> { … }`): `charset`,
/// `chars`, `em_px`, `px_range`, `page_size`, `runtime`, `features`,
/// `color`, `color_px` and `palette` lines, which no longer mean anything
/// now that a face ships whole and draws at the size it is shown. Lines
/// outside a font block are left as they are.
fn font_atlas_keys(text: &str) -> Option<(String, usize)> {
    const KEYS: [&str; 10] = [
        "charset",
        "chars",
        "em_px",
        "px_range",
        "page_size",
        "runtime",
        "features",
        "color",
        "color_px",
        "palette",
    ];
    let (mut out, mut n) = (String::with_capacity(text.len()), 0);
    // The open blocks' first words, innermost last.
    let mut blocks: Vec<&str> = Vec::new();
    for l in text.split_inclusive('\n') {
        let code = l.split('#').next().unwrap_or("").trim();
        let key = code.split_whitespace().next().unwrap_or("");
        if blocks.last() == Some(&"font") && KEYS.contains(&key) {
            n += 1;
            continue;
        }
        for c in code.chars() {
            match c {
                '{' => blocks.push(code.split_whitespace().next().unwrap_or("")),
                '}' => {
                    blocks.pop();
                }
                _ => {}
            }
        }
        out.push_str(l);
    }
    (n > 0).then_some((out, n))
}

/// A light's `static` mark moved to its mobility: in a scene's
/// og.PointLight, og.SpotLight or og.RectLight block, `static true`
/// becomes `mobility stationary` (and `static false` `mobility movable`);
/// in code, on a line inserting or setting one of them, `static: true`
/// becomes `mobility: "stationary"` (`static: false`, `"movable"`). A
/// static light was one the far field (RenderSettings.staticLightReach)
/// may cache: a stationary one now.
fn light_mobility(text: &str) -> Option<(String, usize)> {
    const LIGHTS: [&str; 3] = ["PointLight", "SpotLight", "RectLight"];
    let (mut out, mut n) = (String::with_capacity(text.len()), 0);
    // The scene text's open blocks, innermost last.
    let mut blocks: Vec<String> = Vec::new();
    for l in text.split_inclusive('\n') {
        let code = l.split('#').next().unwrap_or("").trim();
        let in_light = blocks
            .last()
            .is_some_and(|b| LIGHTS.contains(&b.trim_start_matches("og.")));
        let mut line = l.to_string();
        if in_light && let Some(rest) = code.strip_prefix("static ") {
            let to = match rest.trim() {
                "true" | "1" => Some("stationary"),
                "false" | "0" => Some("movable"),
                _ => None,
            };
            if let Some(to) = to {
                let at = l.find("static").expect("the word");
                let end = at + l[at..].find(rest.trim()).expect("its value") + rest.trim().len();
                line = format!("{}mobility {to}{}", &l[..at], &l[end..]);
                n += 1;
            }
        } else if LIGHTS.iter().any(|k| l.contains(&format!("og.{k}"))) {
            for (from, to) in [
                ("static: true", "mobility: \"stationary\""),
                ("static: false", "mobility: \"movable\""),
            ] {
                let k = line.matches(from).count();
                if k > 0 {
                    line = line.replace(from, to);
                    n += k;
                }
            }
        }
        for c in code.chars() {
            match c {
                '{' => blocks.push(code.split('{').next().unwrap_or("").trim().to_string()),
                '}' => {
                    blocks.pop();
                }
                _ => {}
            }
        }
        out.push_str(&line);
    }
    (n > 0).then_some((out, n))
}

/// The `gamepad` events a game's code sends, moved to the raw form: on a
/// line naming `kind: "gamepad"`, `buttons: ["a", …]` becomes the values
/// by GamepadButton index, `triggers: [lt, rt]` folds into buttons 6 and
/// 7, and `device: "playstation"` becomes that vendor's `guid`. Lines of
/// an event spread over several lines (or built from variables) are left
/// for the author: their numbers come back.
fn raw_pads(text: &str) -> Option<(String, usize, Vec<usize>)> {
    const BUTTONS: [&str; 17] = [
        "a",
        "b",
        "x",
        "y",
        "leftBumper",
        "rightBumper",
        "leftTrigger",
        "rightTrigger",
        "back",
        "start",
        "leftStick",
        "rightStick",
        "up",
        "down",
        "left",
        "right",
        "home",
    ];
    // `key: [ … ]` or `"key": [ … ]` (or a string for `device`): the span
    // of the whole field (with a comma after it) and the value's text.
    fn field(line: &str, key: &str) -> Option<(usize, usize, String)> {
        let at = [format!("\"{key}\":"), format!("{key}:")]
            .iter()
            .find_map(|k| line.find(k.as_str()).map(|i| (i, i + k.len())))?;
        let rest = &line[at.1..];
        let lead = rest.len() - rest.trim_start().len();
        let v = &rest[lead..];
        let len = if v.starts_with('[') {
            v.find(']')? + 1
        } else {
            v.strip_prefix('"')?.find('"')? + 2
        };
        let mut end = at.1 + lead + len;
        let tail = &line[end..];
        if let Some(t) = tail.trim_start().strip_prefix(',') {
            end = line.len() - t.trim_start().len();
        }
        Some((at.0, end, v[..len].to_string()))
    }
    let is_pad = |l: &str| l.contains("kind: \"gamepad\"") || l.contains("\"kind\": \"gamepad\"");
    if !text.lines().any(is_pad) {
        return None;
    }
    let (mut out, mut n, mut left) = (String::with_capacity(text.len()), 0, Vec::new());
    for (i, l) in text.split_inclusive('\n').enumerate() {
        if !is_pad(l) {
            out.push_str(l);
            continue;
        }
        let quoted = l.contains("\"kind\"");
        let names = field(l, "buttons");
        let parsed: Option<Vec<usize>> = names.as_ref().and_then(|(_, _, v)| {
            let inner = v.trim_start_matches('[').trim_end_matches(']');
            inner
                .split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(|x| {
                    BUTTONS
                        .iter()
                        .position(|b| x.trim_matches('"') == *b && x.starts_with('"'))
                })
                .collect()
        });
        let triggers = field(l, "triggers");
        let device = field(l, "device");
        if names.is_some() && parsed.is_none()
            || (names.is_none() && triggers.is_none() && device.is_none())
        {
            // Raw already, or built from variables: the author's.
            if triggers.is_some()
                || device.is_some()
                || names.as_ref().is_some_and(|f| f.2.contains('"'))
            {
                left.push(i + 1);
            }
            out.push_str(l);
            continue;
        }
        let mut values = [0.0f32; 17];
        for b in parsed.unwrap_or_default() {
            values[b] = 1.0;
        }
        if let Some((_, _, t)) = &triggers {
            let t: Vec<f32> = t
                .trim_matches(['[', ']'])
                .split(',')
                .filter_map(|x| x.trim().parse().ok())
                .collect();
            for (k, v) in t.iter().take(2).enumerate() {
                values[6 + k] = values[6 + k].max(*v);
            }
        }
        let count = values.iter().rposition(|v| *v != 0.0).map_or(0, |k| k + 1);
        let list = values[..count]
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let guid = device
            .as_ref()
            .and_then(|(_, _, d)| match d.trim_matches('"') {
                "xbox" => Some("030000005e0400000000000000000000"),
                "playstation" => Some("030000004c0500000000000000000000"),
                "switch" => Some("030000007e0500000000000000000000"),
                _ => None,
            });
        let key = |k: &str| {
            if quoted {
                format!("\"{k}\": ")
            } else {
                format!("{k}: ")
            }
        };
        // Cut the fields right to left so earlier spans stay valid.
        let mut cuts: Vec<(usize, usize, String)> = Vec::new();
        if let Some((s, e, _)) = &names {
            cuts.push((*s, *e, format!("{}[{list}], ", key("buttons"))));
        }
        if let Some((s, e, _)) = &triggers {
            let put = if names.is_none() {
                format!("{}[{list}], ", key("buttons"))
            } else {
                String::new()
            };
            cuts.push((*s, *e, put));
        }
        if let Some((s, e, _)) = &device {
            let put = guid.map_or(String::new(), |g| format!("{}\"{g}\", ", key("guid")));
            cuts.push((*s, *e, put));
        }
        cuts.sort_by_key(|c| std::cmp::Reverse(c.0));
        let mut line = l.to_string();
        for (s, e, put) in cuts {
            line.replace_range(s..e, &put);
        }
        // A field that ended the object leaves ", }": close it up.
        let line = line.replace(", }", " }").replace(",  }", " }");
        n += 1;
        out.push_str(&line);
    }
    Some((out, n, left))
}

/// One attribute's text before and after a codemod.
type Rewrite = (String, String);

/// A `.svelte` document's markup moved to the new UI core's attributes
/// (`changelog/unreleased/ui-core-rewrite.json`): the layout data
/// attributes become typed fields, surface data attributes their names,
/// `part="Comp.x"` its surface and the `parts` meta line goes. Only tags
/// are read: text, quoted values, `<style>` and `<script>` stay as they
/// are. Answers the new text, each rewrite (old, new) and the 1-based lines
/// of the attributes it could not rewrite; `None` when nothing changed.
fn ui_core(text: &str) -> Option<(String, Vec<Rewrite>, Vec<usize>)> {
    let (mut out, mut rewrites, mut manual) = (String::new(), Vec::new(), Vec::new());
    let line_of = |at: usize| text[..at].matches('\n').count() + 1;
    let mut i = 0;
    while let Some(lt) = text[i..].find('<').map(|n| i + n) {
        out.push_str(&text[i..lt]);
        // A raw block's body is copied whole.
        let raw = ["style", "script"].into_iter().find(|t| {
            text[lt + 1..].starts_with(t)
                && text[lt + 1 + t.len()..].starts_with(|c: char| c == '>' || c.is_whitespace())
        });
        if let Some(t) = raw {
            let end = text[lt..]
                .find(&format!("</{t}>"))
                .map_or(text.len(), |n| lt + n + t.len() + 3);
            out.push_str(&text[lt..end]);
            i = end;
            continue;
        }
        let name_end = text[lt + 1..]
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .map_or(text.len(), |n| lt + 1 + n);
        let tag = &text[lt + 1..name_end];
        if tag.is_empty() || !tag.starts_with(|c: char| c.is_ascii_alphabetic()) {
            out.push('<');
            i = lt + 1;
            continue;
        }
        // The tag's attributes: (leading space, name, raw value with its `=`).
        let mut attrs: Vec<(&str, &str, &str, usize)> = Vec::new();
        let mut j = name_end;
        let close;
        loop {
            let ws = j;
            while text[j..].starts_with(char::is_whitespace) {
                j += text[j..].chars().next().map_or(1, char::len_utf8);
            }
            if j >= text.len() || text[j..].starts_with('>') || text[j..].starts_with("/>") {
                close = &text
                    [ws..(j + if text[j..].starts_with("/>") { 2 } else { 1 }).min(text.len())];
                j = (j + close.trim_start().len()).min(text.len());
                break;
            }
            let n0 = j;
            while j < text.len()
                && !text[j..]
                    .starts_with(|c: char| c.is_whitespace() || c == '=' || c == '>' || c == '/')
            {
                j += text[j..].chars().next().map_or(1, char::len_utf8);
            }
            if j == n0 {
                j += 1;
            }
            let name = &text[n0..j];
            let v0 = j;
            if text[j..].starts_with('=') {
                j += 1;
                match text[j..].chars().next() {
                    Some(q @ ('"' | '\'')) => {
                        j = text[j + 1..].find(q).map_or(text.len(), |n| j + 1 + n + 1);
                    }
                    Some('{') => {
                        let mut depth = 0;
                        for (k, c) in text[j..].char_indices() {
                            depth += match c {
                                '{' => 1,
                                '}' => -1,
                                _ => 0,
                            };
                            if depth == 0 {
                                j += k + 1;
                                break;
                            }
                        }
                    }
                    _ => {
                        while j < text.len()
                            && !text[j..].starts_with(|c: char| c.is_whitespace() || c == '>')
                        {
                            j += text[j..].chars().next().map_or(1, char::len_utf8);
                        }
                    }
                }
            }
            attrs.push((&text[ws..n0], name, &text[v0..j], n0));
        }
        fn literal(v: &str) -> Option<&str> {
            let v = v.strip_prefix('=').unwrap_or(v);
            (v.len() >= 2 && (v.starts_with('"') || v.starts_with('\'')))
                .then(|| &v[1..v.len() - 1])
        }
        if tag == "meta"
            && attrs
                .iter()
                .any(|a| a.1 == "name" && literal(a.2) == Some("parts"))
        {
            rewrites.push((text[lt..j].to_string(), String::new()));
            i = j;
            if text[i..].starts_with('\n') {
                i += 1;
            }
            continue;
        }
        out.push('<');
        out.push_str(tag);
        for (ws, name, value, at) in attrs {
            let lit = literal(value);
            let new: Option<String> = match (name, lit) {
                (
                    "data-layout",
                    Some("row" | "stack" | "overlay" | "scroll" | "center" | "wrap"),
                ) => Some(format!("layout{value}")),
                ("data-layout", Some("cluster" | "switcher")) => Some("layout=\"wrap\"".into()),
                ("data-layout", Some("shell")) => Some("layout=\"stack\" fill".into()),
                ("data-gap", Some(_)) => Some(format!("gap{value}")),
                (
                    "data-anchor",
                    Some(
                        "fill" | "top-start" | "top-end" | "bottom-start" | "bottom-end"
                        | "top-edge" | "bottom-edge",
                    ),
                ) => Some(format!("anchor{value}")),
                ("data-fill", Some("true" | "")) => Some("fill".into()),
                ("data-fill", None) if value.is_empty() => Some("fill".into()),
                ("data-from", Some("end")) => Some("reverse".into()),
                ("data-rarity", _) => Some(format!("tier{value}")),
                ("data-tone" | "data-shape" | "data-team" | "data-trend", _) => {
                    Some(format!("{}{value}", &name[5..]))
                }
                ("data-motion", Some(_)) => Some(format!("play{value}")),
                ("part", Some(v)) if v.contains('.') => Some(format!(
                    "part=\"{}\"",
                    v.split_whitespace()
                        .map(|p| p.rsplit('.').next().unwrap_or(p))
                        .collect::<Vec<_>>()
                        .join(" ")
                )),
                // Layout fields are literal on the new core: a bound one, or
                // a value with no field, is the author's to rewrite.
                (
                    "data-layout" | "data-gap" | "data-anchor" | "data-fill" | "data-from"
                    | "data-motion",
                    _,
                ) => {
                    manual.push(line_of(at));
                    None
                }
                _ => None,
            };
            out.push_str(ws);
            match new {
                Some(n) => {
                    rewrites.push((format!("{name}{value}"), n.clone()));
                    out.push_str(&n);
                }
                None => {
                    out.push_str(name);
                    out.push_str(value);
                }
            }
        }
        out.push_str(close);
        i = j;
    }
    out.push_str(&text[i..]);
    (!rewrites.is_empty() || !manual.is_empty()).then_some((out, rewrites, manual))
}

/// The game's own text: `src/`, `assets/` and `tests/` (TypeScript,
/// JavaScript, `.ogt`, UI documents, style sheets and Yarn), leaving out
/// build output and dependencies.
fn game_sources(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if !matches!(name, "node_modules" | ".og" | "cooked" | "dist" | "og-out") {
                    walk(&p, out);
                }
            } else if [
                ".ts", ".tsx", ".js", ".mjs", ".ogt", ".svelte", ".css", ".yarn",
            ]
            .iter()
            .any(|x| name.ends_with(x))
            {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join("src"), &mut out);
    walk(&root.join("assets"), &mut out);
    walk(&root.join("tests"), &mut out);
    out
}

/// A game's tsconfig.json with the engine resolved as a package: the
/// `paths` entries into the old `.og/engine` link (`@origingame/engine`
/// and its subpaths, `bun:test`) dropped, a `paths` block left empty with
/// them, and `bun:test`'s declarations named by `types`
/// (`@origingame/engine/bun-test`). Line by line, as og new writes it (one
/// entry a line), so the rest of the file stays as the author keeps it.
/// `None`: it names no `.og/engine`; `Err(line)`: a line the rewrite could
/// not take (the first that still names `.og/engine`, or 1 when the
/// result would not parse), for the author to edit.
fn engine_package_tsconfig(text: &str) -> Option<Result<String, usize>> {
    const LINK: &str = ".og/engine";
    if !text.contains(LINK) {
        return None;
    }
    let mut bun = false;
    let mut lines: Vec<String> = Vec::new();
    for l in text.lines() {
        let t = l.trim_start();
        if (t.starts_with("\"@origingame/engine") || t.starts_with("\"bun:test\""))
            && t.matches(LINK).count() == 1
            && t.matches('[').count() == 1
            && t.trim_end().trim_end_matches(',').ends_with(']')
        {
            bun |= t.starts_with("\"bun:test\"");
            continue;
        }
        lines.push(l.to_string());
    }
    if let Some(at) = lines.iter().position(|l| l.contains(LINK)) {
        return Some(Err(text
            .lines()
            .position(|l| l == lines[at])
            .map_or(1, |n| n + 1)));
    }
    // An empty `paths` block goes.
    if let Some(at) = lines
        .windows(2)
        .position(|w| w[0].trim() == "\"paths\": {" && w[1].trim().trim_end_matches(',') == "}")
    {
        lines.drain(at..at + 2);
    }
    if bun {
        const TYPES: &str = "\"@origingame/engine/bun-test\"";
        match lines
            .iter()
            .position(|l| l.trim_start().starts_with("\"types\":"))
        {
            Some(at) if lines[at].contains(TYPES) => {}
            Some(at) if lines[at].contains("[]") => {
                lines[at] = lines[at].replacen("[]", &format!("[{TYPES}]"), 1)
            }
            Some(at) if lines[at].contains('[') => {
                lines[at] = lines[at].replacen('[', &format!("[{TYPES}, "), 1)
            }
            Some(at) => return Some(Err(at + 1)),
            None => {
                let at = lines
                    .iter()
                    .position(|l| l.trim() == "\"compilerOptions\": {")?;
                let indent = lines[at].len() - lines[at].trim_start().len() + 2;
                lines.insert(
                    at + 1,
                    format!("{}\"types\": [{TYPES}],", " ".repeat(indent)),
                );
            }
        }
    }
    // A trailing comma before a block's close goes.
    for i in 0..lines.len() {
        let next = lines[i + 1..].iter().find(|l| !l.trim().is_empty());
        if lines[i].trim_end().ends_with(',')
            && next.is_some_and(|n| n.trim_start().starts_with(['}', ']']))
        {
            let end = lines[i].trim_end().len() - 1;
            lines[i].truncate(end);
        }
    }
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    if serde_json::from_str::<Value>(text).is_ok() && serde_json::from_str::<Value>(&out).is_err() {
        return Some(Err(1));
    }
    Some(Ok(out))
}

/// project.ogt without its `uiCore` line.
fn without_ui_core(text: &str) -> String {
    text.split_inclusive('\n')
        .filter(|l| {
            !l.trim_start()
                .strip_prefix("uiCore")
                .is_some_and(|r| r.starts_with(char::is_whitespace))
        })
        .collect()
}

/// project.ogt with its top-level `profile` line moved to the creation
/// form (`changelog/unreleased/profile-is-creation-form.json`): `profile
/// "2d"` becomes `create { form "2d" }` (a line of the create block when
/// the project has one without a form), `profile "3d"` goes (no form is
/// 3D, and a create block's form other than 2d already is). A line that
/// cannot move so (a 2d profile beside another form) is left for the
/// author.
fn form_of_profile(text: &str) -> String {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let Some(at) = lines.iter().position(|l| {
        l.strip_prefix("profile")
            .is_some_and(|r| r.starts_with(char::is_whitespace))
    }) else {
        return text.to_string();
    };
    let value = lines[at]["profile".len()..]
        .trim()
        .trim_matches('"')
        .to_string();
    let create = lines
        .iter()
        .position(|l| l.trim_end() == "create {" || l.trim_end() == "create{");
    let form = create.and_then(|c| {
        lines[c + 1..]
            .iter()
            .take_while(|l| !l.starts_with('}'))
            .find_map(|l| {
                l.trim()
                    .strip_prefix("form")
                    .map(|r| r.trim().trim_matches('"').to_string())
            })
    });
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match (value.as_str(), create, form.as_deref()) {
        ("3d", _, None | Some("2.5d" | "3d" | "film")) | ("2d", _, Some("2d")) => {
            out.remove(at);
        }
        ("2d", Some(c), None) => {
            out.insert(c + 1, "  form \"2d\"\n".into());
            out.remove(at);
        }
        ("2d", None, _) => out[at] = "create {\n  form \"2d\"\n}\n".into(),
        _ => {}
    }
    out.concat()
}

/// project.ogt with its create block's `form "film"` as `form "3d"`, the
/// form an interactive film's live scenes draw in; `None` without one.
fn film_to_3d(text: &str) -> Option<String> {
    let mut inside = false;
    let mut hit = false;
    let out: Vec<String> = text
        .split_inclusive('\n')
        .map(|l| {
            let t = l.trim();
            if t == "create {" || t == "create{" {
                inside = true;
            } else if inside && t.starts_with('}') {
                inside = false;
            } else if inside && !hit && t == "form \"film\"" {
                hit = true;
                return l.replacen("\"film\"", "\"3d\"", 1);
            }
            l.to_string()
        })
        .collect();
    hit.then(|| out.concat())
}

/// project.ogt with its `engine "<v>"` line set to `to`.
fn set_engine_line(text: &str, to: &str) -> Option<String> {
    let mut hit = false;
    let lines: Vec<String> = text
        .split('\n')
        .map(|l| {
            let t = l.trim_start();
            if !hit
                && t.strip_prefix("engine")
                    .is_some_and(|r| r.starts_with(char::is_whitespace))
            {
                hit = true;
                let indent = &l[..l.len() - t.len()];
                format!("{indent}engine \"{to}\"")
            } else {
                l.to_string()
            }
        })
        .collect();
    hit.then(|| lines.join("\n"))
}

fn print_text(r: &Value, dry: bool) {
    if r["status"] == "failed" {
        eprintln!("og upgrade: {}", r["reason"].as_str().unwrap_or("failed"));
        return;
    }
    let s = |v: &Value| v.as_str().unwrap_or("").to_string();
    if r["kind"] == "none" {
        println!("og upgrade: already on {}", s(&r["to"]));
        return;
    }
    println!(
        "og upgrade: {} -> {} ({}, simulation {}){}",
        s(&r["from"]),
        s(&r["to"]),
        s(&r["kind"]),
        s(&r["simulation"]),
        if dry {
            ", dry run: nothing written"
        } else {
            ""
        }
    );
    for f in r["edited"].as_array().into_iter().flatten() {
        println!("  edited   {}", s(f));
    }
    for c in r["codemods"].as_array().into_iter().flatten() {
        println!(
            "  rewrote  {}: {} -> {} ({}x, {})",
            s(&c["file"]),
            s(&c["from"]),
            s(&c["to"]),
            c["count"],
            s(&c["version"])
        );
    }
    for m in r["manual"].as_array().into_iter().flatten() {
        println!(
            "  edit     {}:{}: {} ({}): {} Fix: {}",
            s(&m["file"]),
            m["line"],
            s(&m["member"]),
            s(&m["version"]),
            s(&m["change"]),
            s(&m["fix"])
        );
    }
    for m in r["review"].as_array().into_iter().flatten() {
        println!(
            "  review   {} ({}): {} Fix: {}",
            s(&m["member"]),
            s(&m["version"]),
            s(&m["change"]),
            s(&m["fix"])
        );
    }
    if !dry {
        println!("  then: og check, og test");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// og.film's names become og.flow's wherever a game writes them (code,
    /// Yarn headers, a segment's markers, commands, screens' names, keys
    /// and parts), and nothing of the game's own is touched. Ways this
    /// could fail: a name rewritten inside a longer one (`FilmRef`,
    /// `MyFilmView`, `FilmViewer`), a file's path taken for a screen's
    /// name, the game's own `film.open` expression taken for the marker,
    /// or `FilmDesc` becoming og.nav's FlowDesc.
    #[test]
    fn flow_rename_takes_og_films_names_and_leaves_the_games_own() {
        let game = r#"import type { FilmView as View, FilmChartLink } from "@origingame/engine";
import { FilmChoices, FilmFlowchartScreen } from "@origingame/engine/ui/components";
import { FilmViewer, MyFilmView } from "../assets/ui/film-view.tsx";
import "./film-choices";
const FilmRef = defineResource("FilmRef", { film: Entity() });
og.film.load(world, "film");
const desc: og.FilmDesc = { dialogue: "film", paused: true };
const film = og.film.start(world, desc);
world.query(og.Film, og.FilmDeck);
og.story.start(world, { dialogue: "film", hold: "og-film" });
og.ui.push(world, 'film-flowchart');
const label = t("screen.film-choices.label");
if (film.open && marker === "film.cut") run("10.6:film:choose:0", `film:play`);
"#;
        let want = r#"import type { FlowView as View, FlowChartLink } from "@origingame/engine";
import { FlowChoices, FlowChartScreen } from "@origingame/engine/ui/components";
import { FilmViewer, MyFilmView } from "../assets/ui/film-view.tsx";
import "./film-choices";
const FilmRef = defineResource("FilmRef", { film: Entity() });
og.flow.load(world, "film");
const desc: og.FlowStartDesc = { dialogue: "film", paused: true };
const film = og.flow.start(world, desc);
world.query(og.Flow, og.FlowDeck);
og.story.start(world, { dialogue: "film", hold: "og-flow" });
og.ui.push(world, 'flow-chart');
const label = t("screen.flow-choices.label");
if (film.open && marker === "flow.cut") run("10.6:flow:choose:0", `flow:play`);
"#;
        let (out, renamed) = flow_rename(game).unwrap();
        assert_eq!(out, want);
        let count = |from: &str| renamed.iter().find(|r| r.0 == from).map_or(0, |r| r.2);
        assert_eq!(
            (count("og.film"), count("FilmView"), count("og-film")),
            (2, 1, 1)
        );
        assert_eq!(
            renamed.iter().map(|r| r.2).sum::<usize>(),
            15,
            "{renamed:?}"
        );
        // A Yarn story, a segment and a style sheet.
        let texts = [
            (
                "title: a\nog-film: segments/a.ogt\nog-film-chapter: chapter.a\n---\n===\n",
                "title: a\nog-flow: segments/a.ogt\nog-flow-chapter: chapter.a\n---\n===\n",
            ),
            (
                "track marker {\n  key 3 {\n    name \"film.open\"\n  }\n}\n",
                "track marker {\n  key 3 {\n    name \"flow.open\"\n  }\n}\n",
            ),
            (
                "Screen::part(film-choices.shade) { opacity: 0.5 }\n",
                "Screen::part(flow-choices.shade) { opacity: 0.5 }\n",
            ),
        ];
        for (text, want) in texts {
            assert_eq!(flow_rename(text).map(|r| r.0).as_deref(), Some(want));
        }
        // Nothing of og.film's: a look named film, a blog's header, a path.
        for own in [
            "post { filmGrain 0.04 }\n",
            "blog-film: yes\n",
            "import x from \"./ui/film-gallery.tsx\";\n",
            "const s = film.cut + film.lock;\n",
        ] {
            assert!(flow_rename(own).is_none(), "{own}");
        }
    }

    /// og new's tsconfig.json before the engine resolved as a package
    /// comes out as og new writes it now, and an entry the rewrite cannot
    /// take (two on one line) is left for the author, with its line.
    #[test]
    fn engine_package_drops_the_link_paths() {
        let old = concat!(
            "{\n  \"compilerOptions\": {\n    \"strict\": true,\n    \"types\": [],\n    \"jsx\": \"preserve\",\n",
            "    \"paths\": {\n      \"@origingame/engine\": [\"./.og/engine/types/index.d.ts\"],\n",
            "      \"bun:test\": [\"./.og/engine/vendor/bun-types/test.d.ts\"],\n",
            "      \"@origingame/engine/ui-check\": [\"./.og/engine/src/node/ui-check.ts\"]\n    }\n  },\n",
            "  \"include\": [\"src/**/*.ts\"]\n}\n"
        );
        let new = concat!(
            "{\n  \"compilerOptions\": {\n    \"strict\": true,\n    \"types\": [\"@origingame/engine/bun-test\"],\n",
            "    \"jsx\": \"preserve\"\n  },\n  \"include\": [\"src/**/*.ts\"]\n}\n"
        );
        assert_eq!(engine_package_tsconfig(old), Some(Ok(new.to_string())));
        assert_eq!(engine_package_tsconfig(new), None);
        // A game's own path stays, and so does its block.
        let own = old.replace(
            "      \"@origingame/engine/ui-check\"",
            "      \"~/*\": [\"./src/*\"],\n      \"@origingame/engine/ui-check\"",
        );
        let kept = engine_package_tsconfig(&own).unwrap().unwrap();
        assert!(
            kept.contains("\"paths\": {\n      \"~/*\": [\"./src/*\"]\n    }"),
            "{kept}"
        );
        serde_json::from_str::<Value>(&kept).unwrap();
        let packed = old.replace("],\n      \"bun:test\"", "], \"bun:test\"");
        assert_eq!(engine_package_tsconfig(&packed), Some(Err(7)));
    }

    #[test]
    fn font_atlas_keys_leave_a_font_block_its_other_lines_and_other_blocks_theirs() {
        let ogt = "type import\nversion 1\n---\nfont {\n  charset latin # the atlas\n  chars \"é\"\n  em_px 48\n  pixel 12\n  runtime true\n}\nmesh {\n  runtime true\n}\n";
        let (out, n) = font_atlas_keys(ogt).expect("rewritten");
        assert_eq!(n, 4);
        assert_eq!(
            out,
            "type import\nversion 1\n---\nfont {\n  pixel 12\n}\nmesh {\n  runtime true\n}\n"
        );
        let riv =
            "font \"Inter\" {\n  source \"fonts/Inter.ttf\"\n  charset latin\n  chars \"→\"\n}\n";
        let (out, n) = font_atlas_keys(riv).expect("rewritten");
        assert_eq!(
            (n, out.as_str()),
            (2, "font \"Inter\" {\n  source \"fonts/Inter.ttf\"\n}\n")
        );
        assert_eq!(font_atlas_keys("const charset = 1;\n"), None);
    }

    #[test]
    fn light_mobility_moves_a_lights_static_mark_and_leaves_a_models() {
        let scene = "entity lamp {\n  PointLight {\n    intensity 7\n    static true\n  }\n  Model {\n    static true\n  }\n}\n";
        let (out, n) = light_mobility(scene).expect("rewritten");
        assert_eq!(n, 1);
        assert!(out.contains("    mobility stationary\n"), "{out}");
        assert!(out.contains("  Model {\n    static true\n"), "{out}");
        let code = "world.insert(l, og.RectLight, { width: 2, static: true });\nworld.insert(m, og.Model, { static: true });\n";
        let (out, n) = light_mobility(code).expect("rewritten");
        assert_eq!(n, 1);
        assert!(
            out.contains("og.RectLight, { width: 2, mobility: \"stationary\" }"),
            "{out}"
        );
        assert!(out.contains("og.Model, { static: true }"), "{out}");
    }

    #[test]
    fn raw_pads_moves_gamepad_events_to_raw_values() {
        let src = concat!(
            "engine.probe.input({ kind: \"gamepad\", pad: 5, buttons: [\"a\", \"up\"], triggers: [0, 0.5], device: \"playstation\" });\n",
            "send({ \"kind\": \"gamepad\", \"pad\": 1, \"buttons\": [] });\n",
            "send({ kind: \"gamepad\", pad: 2, buttons: held });\n",
            "const x = { buttons: [\"a\"] };\n",
        );
        let (out, n, left) = raw_pads(src).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0],
            "engine.probe.input({ kind: \"gamepad\", pad: 5, buttons: [1, 0, 0, 0, 0, 0, 0, 0.5, 0, 0, 0, 0, 1], guid: \"030000004c0500000000000000000000\" });"
        );
        assert_eq!(
            lines[1],
            "send({ \"kind\": \"gamepad\", \"pad\": 1, \"buttons\": [] });"
        );
        assert_eq!(
            &lines[2..],
            &src.lines().collect::<Vec<_>>()[2..],
            "not an event's names: untouched"
        );
        assert_eq!((n, left), (2, vec![]));
        assert!(raw_pads("no pads here\n").is_none());
    }

    #[test]
    fn the_pin_is_rewritten_in_place() {
        let ogt =
            "type project\nversion 1\n---\nname \"x\"\nengine \"0.5.0\"\nentry \"src/main.ts\"\n";
        assert_eq!(
            set_engine_line(ogt, "0.6.2").unwrap(),
            ogt.replace("0.5.0", "0.6.2")
        );
        assert!(set_engine_line("type project\n---\nengineer \"a\"\n", "1.0.0").is_none());
    }

    #[test]
    fn ui_core_rewrites_tags_only() {
        let doc = concat!(
            "<meta name=\"type\" content=\"component\">\n",
            "<meta name=\"parts\" content=\"Slot.frame Slot.icon\">\n",
            "<meta name=\"prop:r\" content=\"string\" data-doc=\"on data-rarity (data-fill too)\">\n",
            "<div part=\"Slot.frame\" data-layout=\"cluster\" data-gap=\"tight\" data-rarity={r}>\n",
            "  <span data-fill data-from=\"end\" data-layout=\"cover\" class=\"a\">data-tone=\"x\"</span>\n",
            "  <i data-anchor=\"top\" data-motion=\"pulse\" data-tone='warn'/>\n",
            "  <b data-layout={a ? 'row' : 'stack'}></b>\n",
            "</div>\n",
            "<style>.a[data-tone=\"x\"] { color: red; }</style>\n",
        );
        let (out, rewrites, manual) = ui_core(doc).unwrap();
        assert_eq!(
            out,
            concat!(
                "<meta name=\"type\" content=\"component\">\n",
                "<meta name=\"prop:r\" content=\"string\" data-doc=\"on data-rarity (data-fill too)\">\n",
                "<div part=\"frame\" layout=\"wrap\" gap=\"tight\" tier={r}>\n",
                "  <span fill reverse data-layout=\"cover\" class=\"a\">data-tone=\"x\"</span>\n",
                "  <i data-anchor=\"top\" play=\"pulse\" tone='warn'/>\n",
                "  <b data-layout={a ? 'row' : 'stack'}></b>\n",
                "</div>\n",
                "<style>.a[data-tone=\"x\"] { color: red; }</style>\n",
            )
        );
        // cover, a `top` anchor and a bound layout have no field: the
        // author writes a class.
        assert_eq!(manual, [5, 6, 7]);
        assert_eq!(rewrites.len(), 9);
        assert!(ui_core("<div class=\"a\">{a<b}</div>\n").is_none());
        assert_eq!(
            without_ui_core("name \"x\"\nuiCore next\nuiCores 1\n"),
            "name \"x\"\nuiCores 1\n"
        );
    }

    #[test]
    fn a_film_form_becomes_3d_inside_the_create_block_only() {
        assert_eq!(
            film_to_3d(
                "form \"film\"\ncreate {\n  form \"film\"\n  genre \"interactive-film\"\n}\n"
            )
            .as_deref(),
            Some("form \"film\"\ncreate {\n  form \"3d\"\n  genre \"interactive-film\"\n}\n")
        );
        assert_eq!(film_to_3d("create {\n  form \"2d\"\n}\n"), None);
    }

    #[test]
    fn the_profile_line_becomes_the_creation_form() {
        assert_eq!(
            form_of_profile("name \"x\"\nprofile \"2d\"\ntick 60\n"),
            "name \"x\"\ncreate {\n  form \"2d\"\n}\ntick 60\n"
        );
        assert_eq!(
            form_of_profile("profile \"2d\"\ncreate {\n  genre \"rpg\"\n}\n"),
            "create {\n  form \"2d\"\n  genre \"rpg\"\n}\n"
        );
        assert_eq!(form_of_profile("profile \"3d\"\ntick 60\n"), "tick 60\n");
        let clash = "profile \"3d\"\ncreate {\n  form \"2d\"\n}\n";
        assert_eq!(form_of_profile(clash), clash, "left for the author");
    }
}
