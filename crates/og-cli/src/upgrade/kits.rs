//! The `ui-kits` codemod: og.ui members and their types that moved into
//! kit extensions (docs/ui.md §17). The breaking entry says which kit took
//! which members and types (`codemod.kits`), so a new kit needs no edit
//! here. In a TypeScript or JavaScript source it rewrites
//!
//! - `og.ui.<member>` to `<kit>.<member>`;
//! - a moved type named in an `import { … } from "@origingame/engine"` to
//!   an import from the kit;
//! - `<ns>.<Type>` of an `import type * as <ns> from "@origingame/engine"`
//!   to the bare type, imported from the kit;
//!
//! and imports each kit it used from its bundled copy
//! (`@origingame/engine/extensions/<kit>/index.ts`). A file that already
//! imports the kit (bundled or installed) keeps its import. Everything else
//! that names a moved member (`ui.<member>` through another alias, a kit
//! name the file already uses for something else) is left as written and
//! reported for a manual edit. Rerunning it changes nothing.

use serde_json::Value;

/// One kit's moved names, from the changelog entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Kit {
    pub name: String,
    pub members: Vec<String>,
    pub types: Vec<String>,
}

/// The kits of a `codemod: { kind: "ui-kits", kits: { <kit>: { members, types } } }`.
pub fn kits(codemod: &Value) -> Vec<Kit> {
    let names = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    };
    codemod["kits"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, k)| Kit {
            name: name.clone(),
            members: names(&k["members"]),
            types: names(&k["types"]),
        })
        .collect()
}

/// What the codemod did to one file.
#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    /// Rewritten call sites and type names.
    pub count: usize,
    /// The kits it imported.
    pub kits: Vec<String>,
    /// Lines (1-based) with their text and the reason they need a hand edit.
    pub manual: Vec<(usize, String, String)>,
}

pub fn specifier(kit: &str) -> String {
    format!("@origingame/engine/extensions/{kit}/index.ts")
}

fn ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

/// Every start of `word` in `text` not inside a longer identifier or after a `.`.
fn starts(text: &str, word: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find(word) {
        let at = from + i;
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        if !before.is_some_and(|c| ident(c) || c == '.') && !after.is_some_and(ident) {
            out.push(at);
        }
        from = at + word.len();
    }
    out
}

/// Replaces `<prefix><name>` (not inside a longer identifier) by `<to><name>`.
fn replace_member(text: &mut String, prefix: &str, name: &str, to: &str) -> usize {
    let pattern = format!("{prefix}{name}");
    let hits = starts(text, &pattern);
    for &at in hits.iter().rev() {
        text.replace_range(at..at + prefix.len(), to);
    }
    hits.len()
}

/// An import statement: its byte range, its specifier and its braces' names.
struct Import {
    start: usize,
    end: usize,
    spec: String,
    type_only: bool,
    names: Option<Vec<String>>,
    namespace: Option<String>,
}

/// The file's top-level `import … from "…";` statements (possibly over lines).
fn imports(text: &str) -> Vec<Import> {
    let mut out = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if !line.starts_with("import ") {
            continue;
        }
        let Some(rel) = text[start..].find(';') else {
            continue;
        };
        let end = start + rel + 1;
        let stmt = &text[start..end];
        let Some(from) = stmt.rfind(" from ") else {
            continue;
        };
        let spec = stmt[from + 6..]
            .trim()
            .trim_end_matches(';')
            .trim_matches(['"', '\''])
            .to_string();
        let clause = stmt["import ".len()..from].trim();
        let (type_only, clause) = match clause.strip_prefix("type ") {
            Some(c) => (true, c.trim()),
            None => (false, clause),
        };
        let names = clause.find('{').zip(clause.rfind('}')).map(|(a, b)| {
            clause[a + 1..b]
                .split(',')
                .map(|n| n.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|n| !n.is_empty())
                .collect::<Vec<_>>()
        });
        let namespace = clause.strip_prefix("* as ").map(|n| n.trim().to_string());
        let end = end + usize::from(text[end..].starts_with('\n'));
        out.push(Import {
            start,
            end,
            spec,
            type_only,
            names,
            namespace,
        });
    }
    out
}

/// The name an import specifier binds (`type X as Y` binds Y) and the one it imports.
fn binding(name: &str) -> (&str, &str) {
    let n = name.strip_prefix("type ").unwrap_or(name);
    match n.split_once(" as ") {
        Some((a, b)) => (a.trim(), b.trim()),
        None => (n, n),
    }
}

fn line_of(text: &str, at: usize) -> (usize, String) {
    let n = text[..at].matches('\n').count();
    (n + 1, text.lines().nth(n).unwrap_or("").trim().to_string())
}

/// Runs the codemod over one TypeScript or JavaScript source.
pub fn migrate(text: &str, kits: &[Kit]) -> (String, Outcome) {
    let mut out = text.to_string();
    let mut o = Outcome::default();
    let engine = |s: &str| s == "@origingame/engine";
    // Which kits the file names, and the type names it imports per kit.
    let mut used: Vec<(String, Vec<String>)> = Vec::new();
    let mut add = |kit: &str, ty: Option<String>| {
        let i = match used.iter().position(|(k, _)| k == kit) {
            Some(i) => i,
            None => {
                used.push((kit.to_string(), Vec::new()));
                used.len() - 1
            }
        };
        if let Some(t) = ty
            && !used[i].1.contains(&t)
        {
            used[i].1.push(t);
        }
    };
    // The kit's name bound to something else: a declaration or another import.
    let declared_elsewhere = |text: &str, kit: &str| {
        let declared = ["const", "let", "var", "function", "class"]
            .iter()
            .any(|k| !starts(text, &format!("{k} {kit}")).is_empty());
        let imported = imports(text).iter().any(|i| {
            !(i.spec == specifier(kit) || i.spec.ends_with(&format!("ui-kits/{kit}/index.ts")))
                && (i.names.iter().flatten().any(|n| binding(n).1 == kit)
                    || i.namespace.as_deref() == Some(kit))
        });
        declared || imported
    };
    // Members.
    for kit in kits {
        let named = kit
            .members
            .iter()
            .any(|m| !starts(&out, &format!("og.ui.{m}")).is_empty());
        if !named {
            continue;
        }
        if declared_elsewhere(&out, &kit.name) {
            for m in &kit.members {
                for at in starts(&out, &format!("og.ui.{m}")) {
                    let (n, l) = line_of(&out, at);
                    o.manual.push((n, l, format!("`{}` already names something else here: import the {} kit under another name and call it there", kit.name, kit.name)));
                }
            }
            continue;
        }
        for m in &kit.members {
            o.count += replace_member(&mut out, "og.ui.", m, &format!("{}.", kit.name));
        }
        add(&kit.name, None);
    }
    // Types in named imports from the engine.
    let type_kit = |t: &str| kits.iter().find(|k| k.types.iter().any(|x| x == t));
    for imp in imports(&out).into_iter().rev() {
        if !engine(&imp.spec) {
            continue;
        }
        if let Some(names) = &imp.names {
            let (moved, kept): (Vec<&String>, Vec<&String>) =
                names.iter().partition(|n| type_kit(binding(n).0).is_some());
            if moved.is_empty() {
                continue;
            }
            for n in &moved {
                let (from, local) = binding(n);
                let k = type_kit(from).unwrap();
                add(
                    &k.name,
                    Some(if from == local {
                        from.to_string()
                    } else {
                        format!("{from} as {local}")
                    }),
                );
                o.count += 1;
            }
            let stmt = if kept.is_empty() {
                String::new()
            } else {
                let kept: Vec<&str> = kept.iter().map(|s| s.as_str()).collect();
                format!(
                    "import {}{{ {} }} from \"@origingame/engine\";\n",
                    if imp.type_only { "type " } else { "" },
                    kept.join(", ")
                )
            };
            out.replace_range(imp.start..imp.end, &stmt);
        }
    }
    // `<ns>.<Type>` through a namespace import of the engine.
    for ns in imports(&out)
        .into_iter()
        .filter(|i| engine(&i.spec))
        .filter_map(|i| i.namespace)
    {
        for kit in kits {
            for t in &kit.types {
                let n = replace_member(&mut out, &format!("{ns}."), t, "");
                if n > 0 {
                    o.count += n;
                    add(&kit.name, Some(t.clone()));
                }
            }
        }
    }
    // What still names a moved member some other way.
    for kit in kits {
        for m in &kit.members {
            for at in starts(&out, &format!("ui.{m}")) {
                let (n, l) = line_of(&out, at);
                o.manual.push((
                    n,
                    l,
                    format!(
                        "og.ui.{m} moved into the {} kit: call {}.{m}",
                        kit.name, kit.name
                    ),
                ));
            }
        }
    }
    if used.is_empty() {
        return (out, o);
    }
    // The kits' imports: merged into one the file already has, else added
    // after its last import.
    let mut fresh = String::new();
    for (kit, types) in &used {
        let wants_value = kits.iter().find(|k| &k.name == kit).is_some_and(|k| {
            k.members
                .iter()
                .any(|m| !starts(&out, &format!("{kit}.{m}")).is_empty())
        });
        let existing = imports(&out).into_iter().find(|i| {
            (i.spec == specifier(kit) || i.spec.ends_with(&format!("ui-kits/{kit}/index.ts")))
                && i.names.is_some()
        });
        let mut names: Vec<String> = Vec::new();
        if wants_value {
            names.push(kit.clone());
        }
        names.extend(types.iter().map(|t| format!("type {t}")));
        if let Some(imp) = existing {
            let have = imp.names.clone().unwrap_or_default();
            let mut all = have.clone();
            for n in names {
                let bare = binding(&n).1.to_string();
                if !all.iter().any(|h| binding(h).1 == bare) {
                    all.push(if imp.type_only {
                        n.trim_start_matches("type ").to_string()
                    } else {
                        n
                    });
                }
            }
            if all != have {
                let stmt = format!(
                    "import {}{{ {} }} from \"{}\";\n",
                    if imp.type_only { "type " } else { "" },
                    all.join(", "),
                    imp.spec
                );
                out.replace_range(imp.start..imp.end, &stmt);
            }
        } else if !names.is_empty() {
            fresh.push_str(&format!(
                "import {{ {} }} from \"{}\";\n",
                names.join(", "),
                specifier(kit)
            ));
            o.kits.push(kit.clone());
        }
    }
    if !fresh.is_empty() {
        let at = imports(&out).last().map_or(0, |i| i.end);
        out.insert_str(at, &fresh);
    }
    (out, o)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kits() -> Vec<Kit> {
        vec![
            Kit {
                name: "racing".into(),
                members: vec!["speedometer".into(), "lapTimer".into()],
                types: vec!["SpeedometerDesc".into(), "StandingsEntry".into()],
            },
            Kit {
                name: "shooter".into(),
                members: vec!["crosshair".into()],
                types: vec!["KillfeedEntry".into()],
            },
        ]
    }

    #[test]
    fn calls_types_and_namespaces_move_to_their_kits_once() {
        let src = "import { og, type World, type StandingsEntry } from \"@origingame/engine\";\nimport type * as Api from \"@origingame/engine\";\nimport type {\n  KillfeedEntry,\n  Vector2,\n} from \"@origingame/engine\";\n\nconst rows: StandingsEntry[] = [];\nfunction hud(d: Api.SpeedometerDesc) {\n  og.ui.speedometer(\"Speed\", d);\n  og.ui.lapTimerX(\"keep\");\n  og.ui.crosshair(\"Aim\", {});\n  og.ui.statBar(\"Hp\", {});\n}\n";
        let (out, o) = migrate(src, &kits());
        assert_eq!(
            out,
            "import { og, type World } from \"@origingame/engine\";\nimport type * as Api from \"@origingame/engine\";\nimport type { Vector2 } from \"@origingame/engine\";\nimport { racing, type StandingsEntry, type SpeedometerDesc } from \"@origingame/engine/extensions/racing/index.ts\";\nimport { shooter, type KillfeedEntry } from \"@origingame/engine/extensions/shooter/index.ts\";\n\nconst rows: StandingsEntry[] = [];\nfunction hud(d: SpeedometerDesc) {\n  racing.speedometer(\"Speed\", d);\n  og.ui.lapTimerX(\"keep\");\n  shooter.crosshair(\"Aim\", {});\n  og.ui.statBar(\"Hp\", {});\n}\n"
        );
        assert_eq!(o.count, 5);
        assert!(o.manual.is_empty(), "{:?}", o.manual);
        let (again, o2) = migrate(&out, &kits());
        assert_eq!(again, out, "a second run changes nothing");
        assert_eq!(o2, Outcome::default());
    }

    #[test]
    fn an_installed_import_is_kept_and_a_taken_name_is_reported() {
        let src = "import { racing } from \"../assets/ui-kits/racing/index.ts\";\nimport { og } from \"@origingame/engine\";\nracing.speedometer(\"A\", {});\nog.ui.lapTimer(\"B\", {});\n";
        let (out, o) = migrate(src, &kits());
        assert!(out.contains("racing.lapTimer(\"B\""), "{out}");
        assert!(!out.contains("extensions/racing"), "{out}");
        assert!(o.manual.is_empty());

        let mentioned = "import { og } from \"@origingame/engine\";\n// the racing HUD\nconst s = \"racing\";\nog.ui.speedometer(\"A\", {});\n";
        let (out, o) = migrate(mentioned, &kits());
        assert!(
            out.contains("racing.speedometer(\"A\""),
            "a word in a comment or string does not block: {out}"
        );
        assert!(o.manual.is_empty());

        let taken = "import { og } from \"@origingame/engine\";\nconst shooter = 1;\nog.ui.crosshair(\"A\", {});\nconst ui = og.ui;\nui.speedometer(\"S\", {});\n";
        let (out, o) = migrate(taken, &kits());
        assert!(out.contains("og.ui.crosshair"), "left as written: {out}");
        let lines: Vec<usize> = o.manual.iter().map(|m| m.0).collect();
        assert_eq!(lines, vec![3, 5], "{:?}", o.manual);
    }
}
