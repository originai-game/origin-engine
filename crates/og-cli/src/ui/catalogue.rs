//! The discoverable UI library, generated with the API, never a second schema.
use super::{extensions, project};
use og_idl::model::{Api, Item};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::ExitCode;

pub const PATH: &str = "packages/knowledge/ui/catalogue.json";
const GUIDANCE: &str = "A starting point to restyle, not finished game UI. Keep its behaviour, input and localization; change typography, palette, spacing, art and hierarchy for this game's look before publishing.";
/// What to restyle first, by kind (docs/ui.md §12.1): og ui check reports
/// what is left of each, and og build --publish refuses it.
const COMPONENT: &str = "A starting point to restyle, not finished game UI. Keep its behaviour, input and localization; it draws in the game's own style, so restyle that first: the palette, then this part's dressing in the style, then type and icons.";
const SCREEN: &str = "A starting point to change, not finished game UI. Eject it, then change its layout, hierarchy, art slots and text for this game first; og build --publish refuses an ejected document left as shipped and an art slot still on its placeholder.";
const STYLE: &str = "Never shipped as it is: og new copies the look's pack into the game (assets/styles/<game>/), or og style eject copies it. Restyle the copy first: its palette, then its frames and buttons (art/, og style art), fonts, icons and motion; og build --publish refuses a style identical to this pack or still as generated.";
const KIT: &str = "Keep its code: behaviour, input, localization and layout rules. Restyle what it draws first: the game's own style, its documents (eject and change them) and its art slots (the game's art); og build --publish refuses ejected documents left as shipped.";
const FALLBACK: &str = "Development only: temporary icons while the game's own are made. og build --publish refuses a game that still has them; draw the game's own icon set first.";

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub purpose: String,
    pub genres: Vec<String>,
    pub example: String,
    pub kit: Option<String>,
    pub pack: Option<String>,
    pub install: Vec<String>,
    pub source: String,
    pub thumbnail: String,
    pub guidance: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Catalogue {
    pub format: u32,
    pub guidance: String,
    pub entries: Vec<Entry>,
}

fn entry(name: &str, kind: &str, purpose: String, source: String) -> Entry {
    let id = format!("{kind}/{name}");
    Entry {
        thumbnail: format!("thumbnails/{id}.png"),
        id,
        name: name.into(),
        kind: kind.into(),
        purpose,
        genres: vec!["any".into()],
        example: String::new(),
        kit: None,
        pack: None,
        install: vec!["Built into og.ui; no install required".into()],
        source,
        guidance: GUIDANCE.into(),
    }
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn purpose(text: &str) -> String {
    text.lines()
        .skip_while(|l| !l.starts_with('#'))
        .take_while(|l| l.starts_with('#') && l.trim() != "#")
        .map(|l| l.trim_start_matches('#').trim())
        .collect::<Vec<_>>()
        .join(" ")
}

fn jsdoc(before: &str) -> Option<String> {
    let (_, text) = before.trim_end().strip_suffix("*/")?.rsplit_once("/**")?;
    Some(
        text.lines()
            .map(|l| l.trim().trim_start_matches('*').trim())
            .skip_while(|l| l.is_empty())
            .take_while(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn document(path: &Path, source: String, kit: Option<(&str, &str)>) -> Result<Entry, String> {
    let text = read(path)?;
    let stem = path.file_stem().unwrap().to_string_lossy();
    let name = kit.map_or_else(
        || stem.to_string(),
        |(k, p)| format!("{k}/{}", p.trim_end_matches(".ogt")),
    );
    let registered = text
        .lines()
        .find_map(|l| l.strip_prefix("name \"").and_then(|s| s.strip_suffix('"')))
        .unwrap_or(&stem);
    let mut e = entry(&name, "screen", purpose(&text), source);
    e.guidance = SCREEN.into();
    e.example = format!(
        "og.load.document(world, \"ui/{name}.ogt\");\nog.ui.push(world, \"{registered}\", vm);"
    );
    e.install = vec![format!("og ui eject {name}")];
    if let Some((k, _)) = kit {
        e.kit = Some(k.into());
        e.genres = vec![k.into()];
        e.install.insert(0, format!("og ui add {k}"));
    }
    Ok(e)
}

/// Reflection supplies every built-in call's purpose and parameter names.
/// Extension manifests supply discovery; their registered exports and documents
/// supply members, so adding a kit needs no edit to this generator.
pub fn generate(root: &Path, api: &Api) -> Result<og_idl::Generated, String> {
    let mut entries = Vec::new();
    let ui_source = root.join("packages/engine/src/ui");
    let sources = extensions::files(&ui_source)?;
    for module in api.modules.iter().filter(|m| m.name == "ui") {
        for item in &module.items {
            let Item::Fn(f) = item else { continue };
            if f.host || !f.params.iter().any(|p| p.name == "id") {
                continue;
            }
            let needle = format!("function {}(", f.name);
            let source = sources.iter().find(|p| {
                p.extension().is_some_and(|e| e == "ts")
                    && read(&ui_source.join(p)).is_ok_and(|s| s.contains(&needle))
            });
            let Some(source) = source else { continue };
            let group = source
                .components()
                .next()
                .unwrap()
                .as_os_str()
                .to_string_lossy();
            if group == "index.ts" {
                continue;
            }
            let kind = if group == "patterns" {
                "pattern"
            } else {
                "component"
            };
            let mut e = entry(
                &f.name,
                kind,
                f.doc.join(" "),
                format!("packages/engine/src/ui/{}", source.display()),
            );
            e.guidance = COMPONENT.into();
            e.example = format!(
                "og.ui.{}({});",
                f.name,
                f.params
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            entries.push(e);
        }
    }
    for p in &sources {
        if p.starts_with("screens") && p.extension().is_some_and(|e| e == "ogt") {
            entries.push(document(
                &ui_source.join(p),
                format!("packages/engine/src/ui/{}", p.display()),
                None,
            )?);
        }
    }
    for base in ["packages/engine/extensions", "templates/styles"] {
        for file in extensions::files(&root.join(base))? {
            if file.components().count() != 2
                || file.file_name().is_none_or(|n| n != "extension.json")
            {
                continue;
            }
            let dir = root.join(base).join(file.parent().unwrap());
            let extension = extensions::read(&dir, false)?;
            let m = &extension.manifest;
            let mut e = entry(
                &m.name,
                &m.kind,
                m.description.clone(),
                format!("{base}/{}", file.display()),
            );
            e.install = vec![format!(
                "og {} add {}",
                if m.kind == "style" { "style" } else { "ui" },
                m.name
            )];
            if m.kind == "kit" {
                e.guidance = if m.name == "development-fallback-icons" {
                    FALLBACK
                } else {
                    KIT
                }
                .into();
                e.kit = Some(m.name.clone());
                e.genres = if m.entry.is_some() {
                    vec![m.name.clone()]
                } else {
                    vec!["any".into()]
                };
                e.example = if m.entry.is_some() {
                    format!(
                        "import {{ {} }} from \"../assets/ui-kits/{}/index.ts\";",
                        m.name, m.name
                    )
                } else {
                    e.install[0].clone()
                };
            } else {
                e.pack = Some(m.name.clone());
                e.guidance = STYLE.into();
                e.install = vec![format!(
                    "og style eject {} <name> (a copy of the game's own to restyle; og new makes one from the game's look)",
                    m.name
                )];
                e.example =
                    "ui \"styles/<name>/style.ogt\" // in the game's look: its own copy, restyled"
                        .into();
            }
            entries.push(e);
            for doc in &extension.documents {
                let p = dir.join(doc);
                entries.push(document(
                    &p,
                    p.strip_prefix(root).unwrap().to_string_lossy().into(),
                    Some((&m.name, doc)),
                )?);
            }
            if let Some(index) = &m.entry {
                let text = read(&dir.join(index))?;
                let types = read(&dir.join(m.types.as_ref().ok_or("kit entry needs types")?))?;
                // The extension registration is the public list, not every
                // exported implementation helper in src/.
                let registration = text
                    .split("registerUiExtension(")
                    .nth(1)
                    .and_then(|s| s.split_once('{'))
                    .map(|(_, s)| s.split('}').next().unwrap())
                    .ok_or_else(|| {
                        format!("{}: missing registerUiExtension export", dir.display())
                    })?;
                let files = extensions::files(&dir.join("src"))?;
                for name in registration
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let needle = format!("export function {name}(");
                    let mut found = false;
                    for file in &files {
                        let text = read(&dir.join("src").join(file))?;
                        let Some((before, tail)) = text.split_once(&needle) else {
                            continue;
                        };
                        let signature = tail
                            .split("):")
                            .next()
                            .ok_or("missing function signature")?;
                        let args = signature
                            .split(',')
                            .filter_map(|p| p.trim().split(':').next())
                            .map(|p| p.trim_end_matches('?'))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let mut e = entry(
                            &format!("{}.{name}", m.name),
                            "component",
                            jsdoc(before)
                                .or_else(|| {
                                    let desc = signature
                                        .split("desc:")
                                        .nth(1)?
                                        .trim()
                                        .split(|c: char| !c.is_alphanumeric())
                                        .next()?;
                                    let (before, _) =
                                        types.split_once(&format!("export interface {desc} "))?;
                                    jsdoc(before)
                                })
                                .unwrap_or_else(|| format!("{name}: {}", m.description)),
                            format!("{base}/{}/src/{}", m.name, file.display()),
                        );
                        e.kit = Some(m.name.clone());
                        e.guidance = COMPONENT.into();
                        e.genres = vec![m.name.clone()];
                        e.install = vec![format!("og ui add {}", m.name)];
                        e.example = format!(
                            "import {{ {} }} from \"../assets/ui-kits/{}/index.ts\";\n{}.{name}({args});",
                            m.name, m.name, m.name
                        );
                        entries.push(e);
                        found = true;
                        break;
                    }
                    if !found {
                        return Err(format!(
                            "{}: registered component {name} has no source",
                            dir.display()
                        ));
                    }
                }
            }
        }
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    let catalogue = Catalogue {
        format: 1,
        guidance: GUIDANCE.into(),
        entries,
    };
    Ok(og_idl::Generated {
        path: PATH.into(),
        contents: serde_json::to_string_pretty(&catalogue).map_err(|e| e.to_string())? + "\n",
    })
}

fn matches(entry: &Entry, query: &str) -> bool {
    let text = format!(
        "{} {} {} {} {}",
        entry.name,
        entry.kind,
        entry.purpose,
        entry.genres.join(" "),
        entry.kit.as_deref().unwrap_or("")
    )
    .to_lowercase();
    query
        .split_whitespace()
        .all(|word| text.contains(&word.to_lowercase()))
}

pub fn find(args: &[&str]) -> ExitCode {
    let run = || -> Result<(), String> {
        let mut query = Vec::new();
        let mut root = Path::new(".");
        let mut json = false;
        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "--json" => json = true,
                "--project" if i + 1 < args.len() => {
                    i += 1;
                    root = Path::new(args[i]);
                }
                a if a.starts_with('-') => return Err(format!("unknown option {a}")),
                a => query.push(a),
            }
            i += 1;
        }
        if query.join(" ").trim().is_empty() {
            return Err("usage: og ui find <query> [--json] [--project DIR]".into());
        }
        let package = project::engine_package(Some(root)).ok_or(project::NO_ENGINE)?;
        let shipped = package.join("knowledge/ui/catalogue.json");
        let path = if shipped.is_file() {
            shipped
        } else {
            package.join("../knowledge/ui/catalogue.json")
        };
        let catalogue: Catalogue =
            serde_json::from_str(&read(&path)?).map_err(|e| e.to_string())?;
        if catalogue.format != 1 {
            return Err("unsupported UI catalogue format".into());
        }
        let mut entries: Vec<_> = catalogue
            .entries
            .iter()
            .filter(|e| matches(e, &query.join(" ")))
            .collect();
        entries.sort_by_key(|e| {
            (
                e.name.to_lowercase() != query.join(" ").to_lowercase(),
                &e.id,
            )
        });
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?
            );
        } else {
            println!("{GUIDANCE}\n");
            for e in entries {
                println!(
                    "{} [{}]\n  {}\n  {}\n  first: {}\n",
                    e.name,
                    e.kind,
                    e.purpose,
                    e.install.join("; "),
                    e.guidance
                );
            }
        }
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og ui find: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_uses_live_members_and_documents() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let api = og_idl::read(&root.join(og_idl::IDL_PATH)).unwrap();
        let output = generate(&root, &api).unwrap();
        let catalogue: Catalogue = serde_json::from_str(&output.contents).unwrap();
        let button = catalogue
            .entries
            .iter()
            .find(|e| e.name == "pushButton")
            .unwrap();
        assert!(matches(button, "BUTTON disabled"));
        assert!(!matches(button, "button spaceship"));
        assert_eq!(button.example, "og.ui.pushButton(id, label, desc);");
        // A pack is never named as it is: its entry says to copy and restyle it first.
        let pack = catalogue
            .entries
            .iter()
            .find(|e| e.id == "style/pixel")
            .unwrap();
        assert!(!pack.example.contains("\"pixel\""), "{}", pack.example);
        assert!(
            pack.guidance.starts_with("Never shipped as it is"),
            "{}",
            pack.guidance
        );
        assert!(
            pack.install[0].starts_with("og style eject pixel "),
            "{:?}",
            pack.install
        );
        assert!(
            catalogue
                .entries
                .iter()
                .any(|e| e.name == "pause" && e.kind == "screen")
        );
        assert!(
            catalogue
                .entries
                .iter()
                .any(|e| e.name == "rpg.inventoryGrid")
        );
        assert!(
            catalogue
                .entries
                .iter()
                .all(|e| root.join(&e.source).is_file())
        );
        let ids: std::collections::BTreeSet<_> = catalogue.entries.iter().map(|e| &e.id).collect();
        assert_eq!(ids.len(), catalogue.entries.len());
    }
}
