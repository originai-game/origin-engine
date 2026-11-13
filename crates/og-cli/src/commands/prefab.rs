//! `og prefab` (`docs/design.md` §2.5, prefabs):
//!
//! - `og prefab check [project]`: every scene and prefab flattened as the
//!   cook flattens it; cycles, sibling name clashes, generated names that
//!   collide, and orphaned overrides and removes (an override whose node,
//!   component or field the prefab no longer has) are listed, and the
//!   command exits 1 while any exists;
//! - `og prefab show <prefab> [--scene <id> --instance <name>] [--project <dir>]`:
//!   the flattened tree, each field with the text its value comes from (the
//!   prefab, a variant's or an instance's override);
//! - `og prefab extract <scene> <entity> <prefab> [--project <dir>]`: an
//!   authored entity's block becomes a new prefab and the scene's block an
//!   instance of it with no overrides.

use og_cook::cook::prefab::{Flattener, Node, PREFAB_TYPE};
use og_core::ogt::Document;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og prefab check [project] | og prefab show <prefab> [--scene <id> --instance <name>] [--project <dir>] | og prefab extract <scene> <entity> <prefab> [--project <dir>]";

pub fn run(args: &[&str]) -> ExitCode {
    let r = match args {
        ["check", rest @ ..] => check(rest),
        ["show", rest @ ..] => show(rest),
        ["extract", rest @ ..] => extract(rest),
        _ => Err(USAGE.to_string()),
    };
    match r {
        Ok(code) => code,
        Err(e) => {
            eprintln!("og prefab: {e}");
            ExitCode::from(2)
        }
    }
}

/// Every `.ogt` under `assets/` by asset id, with its document type.
fn texts(root: &Path) -> Result<BTreeMap<String, (String, String)>, String> {
    let assets = root.join("assets");
    if !assets.is_dir() {
        return Err(format!("{}: no assets/ directory", root.display()));
    }
    let mut out = BTreeMap::new();
    let mut todo = vec![assets.clone()];
    while let Some(dir) = todo.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                todo.push(path);
            } else if path.extension().is_some_and(|x| x == "ogt")
                && let Ok(text) = std::fs::read_to_string(&path)
                && let Ok(doc) = Document::parse(&text)
            {
                let id = path
                    .strip_prefix(&assets)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(id, (doc.ty, text));
            }
        }
    }
    Ok(out)
}

/// `--flag value` of `args`.
fn flag<'a>(args: &[&'a str], name: &str) -> Option<&'a str> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1])
}

/// The positional arguments (those not a flag or a flag's value).
fn positional<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
        } else if a.starts_with("--") {
            skip = true;
        } else {
            out.push(*a);
        }
    }
    out
}

fn check(args: &[&str]) -> Result<ExitCode, String> {
    let root = PathBuf::from(positional(args).first().copied().unwrap_or("."));
    let all = texts(&root)?;
    let schemas: BTreeMap<String, og_core::Schema> = crate::cook::engine_schemas()
        .into_iter()
        .map(|s| (s.name.clone(), s))
        .collect();
    let prefab_text = |id: &str| {
        all.get(id)
            .filter(|(ty, _)| ty == PREFAB_TYPE)
            .map(|(_, t)| t.clone())
    };
    let mut problems = Vec::new();
    let mut checked = 0;
    for (id, (ty, text)) in &all {
        if ty != "scene" && ty != PREFAB_TYPE {
            continue;
        }
        checked += 1;
        let mut f = Flattener::new(&prefab_text, &schemas);
        let r = if ty == PREFAB_TYPE {
            f.prefab(id).map(|_| ())
        } else {
            Document::parse(text)
                .map_err(|e| og_cook::CookError(e.to_string()))
                .and_then(|d| f.scene(id, &d.body).map(|_| ()))
        };
        if let Err(e) = r {
            problems.push(format!("assets/{id}: {}", e.0));
        }
        for o in f.orphans.iter().filter(|o| o.source == *id) {
            problems.push(format!("assets/{o}"));
        }
    }
    for p in &problems {
        println!("{p}");
    }
    if problems.is_empty() {
        println!("og prefab check: {checked} scene(s) and prefab(s), nothing orphaned");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("og prefab check: {} problem(s)", problems.len());
        Ok(ExitCode::FAILURE)
    }
}

/// One entry as its text line (`hp 40`, `model "m.glb"`).
fn line(e: &og_core::ogt::Entry) -> String {
    let mut e = e.clone();
    e.block = None;
    Document {
        ty: "x".into(),
        version: 1,
        body: og_core::ogt::Block::new().with(e),
    }
    .to_text()
    .ok()
    .and_then(|t| t.trim_end().lines().last().map(str::to_string))
    .unwrap_or_default()
}

fn print(node: &Node) {
    for (path, n) in node.walk() {
        let depth = if path.is_empty() {
            0
        } else {
            path.matches('/').count() + 1
        };
        let pad = "  ".repeat(depth);
        println!("{pad}entity \"{}\"", n.name);
        let origin = |c: &str, f: &str| {
            n.origins
                .get(&(c.to_string(), f.to_string()))
                .map_or("", String::as_str)
                .to_string()
        };
        for e in &n.entries {
            match &e.block {
                None => println!("{pad}  {}  # {}", line(e), origin(&e.key, "")),
                Some(b) => {
                    println!("{pad}  {} {{", e.key);
                    for f in &b.entries {
                        println!("{pad}    {}  # {}", line(f), origin(&e.key, &f.key));
                    }
                    println!("{pad}  }}");
                }
            }
        }
    }
}

fn show(args: &[&str]) -> Result<ExitCode, String> {
    let root = PathBuf::from(flag(args, "--project").unwrap_or("."));
    let [prefab] = positional(args)[..] else {
        return Err(USAGE.into());
    };
    let prefab = og_cook::runtime::asset_id(prefab).to_string();
    let all = texts(&root)?;
    let schemas: BTreeMap<String, og_core::Schema> = crate::cook::engine_schemas()
        .into_iter()
        .map(|s| (s.name.clone(), s))
        .collect();
    let prefab_text = |id: &str| {
        all.get(id)
            .filter(|(ty, _)| ty == PREFAB_TYPE)
            .map(|(_, t)| t.clone())
    };
    let mut f = Flattener::new(&prefab_text, &schemas);
    let node = match (flag(args, "--scene"), flag(args, "--instance")) {
        (Some(scene), Some(instance)) => {
            let scene = og_cook::runtime::asset_id(scene).to_string();
            let (_, text) = all
                .get(&scene)
                .ok_or_else(|| format!("no scene assets/{scene}"))?;
            let doc = Document::parse(text).map_err(|e| e.to_string())?;
            let roots = f.scene(&scene, &doc.body).map_err(|e| e.0)?;
            roots
                .iter()
                .flat_map(|r| r.walk())
                .find(|(_, n)| {
                    n.name == instance
                        && n.link
                            .as_ref()
                            .is_some_and(|l| l.node == 0 && l.prefab == prefab)
                })
                .map(|(_, n)| n.clone())
                .ok_or_else(|| format!("{scene} has no instance '{instance}' of {prefab}"))?
        }
        (None, None) => f.prefab(&prefab).map_err(|e| e.0)?,
        _ => return Err("--scene and --instance go together".into()),
    };
    print(&node);
    for o in &f.orphans {
        println!("# warning: {o}");
    }
    Ok(ExitCode::SUCCESS)
}

/// The lines of the block `entity "<name>" {` opens (that line to its `}`).
fn block_lines(lines: &[&str], name: &str) -> Option<(usize, usize)> {
    let head = format!("entity \"{name}\" {{");
    let start = lines.iter().position(|l| {
        let code = l.split('#').next().unwrap_or("").trim();
        code == head
    })?;
    let mut depth = 0i32;
    for (i, l) in lines.iter().enumerate().skip(start) {
        let code = l.split('#').next().unwrap_or("").trim();
        if code.ends_with('{') {
            depth += 1;
        }
        if code == "}" {
            depth -= 1;
            if depth == 0 {
                return Some((start, i));
            }
        }
    }
    None
}

fn extract(args: &[&str]) -> Result<ExitCode, String> {
    let root = PathBuf::from(flag(args, "--project").unwrap_or("."));
    let [scene, entity, prefab] = positional(args)[..] else {
        return Err(USAGE.into());
    };
    let (scene, prefab) = (
        og_cook::runtime::asset_id(scene),
        og_cook::runtime::asset_id(prefab),
    );
    let scene_path = root.join("assets").join(scene);
    let prefab_path = root.join("assets").join(prefab);
    if prefab_path.exists() {
        return Err(format!("{} exists already", prefab_path.display()));
    }
    let text = std::fs::read_to_string(&scene_path)
        .map_err(|e| format!("{}: {e}", scene_path.display()))?;
    let lines: Vec<&str> = text.split('\n').collect();
    let (start, end) =
        block_lines(&lines, entity).ok_or_else(|| format!("{scene} has no entity \"{entity}\""))?;
    let indent = &lines[start][..lines[start].len() - lines[start].trim_start().len()];
    let mut body = String::from("type prefab\nversion 1\n---\n");
    for l in &lines[start..=end] {
        body.push_str(l.strip_prefix(indent).unwrap_or(l));
        body.push('\n');
    }
    Document::parse(&body).map_err(|e| format!("the extracted prefab does not read: {e}"))?;
    let mut out: Vec<String> = lines[..start].iter().map(|l| l.to_string()).collect();
    out.push(lines[start].to_string());
    out.push(format!("{indent}  prefab \"{prefab}\""));
    out.push(format!("{indent}}}"));
    out.extend(lines[end + 1..].iter().map(|l| l.to_string()));
    let new_scene = out.join("\n");
    Document::parse(&new_scene).map_err(|e| format!("the scene would not read: {e}"))?;
    if let Some(dir) = prefab_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&prefab_path, body).map_err(|e| format!("{}: {e}", prefab_path.display()))?;
    std::fs::write(&scene_path, new_scene).map_err(|e| format!("{}: {e}", scene_path.display()))?;
    println!(
        "og prefab extract: assets/{prefab} from '{entity}' of assets/{scene}; its descendants are now named '{entity}/<path>'"
    );
    Ok(ExitCode::SUCCESS)
}
