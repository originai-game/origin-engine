//! Project lints run alongside `og check`: locale fonts, lockstep,
//! material shader capabilities, and module-scope state.

use super::*;

pub(super) fn locale_fonts_lint(p: &Project, json: bool, to_stderr: bool) {
    let Some(doc) = std::fs::read_to_string(p.root.join("project.ogt"))
        .ok()
        .and_then(|text| og_core::ogt::parse(&text).ok())
    else {
        return;
    };
    let Some(lang) = doc
        .body
        .get("create")
        .and_then(|e| e.block.as_ref())
        .and_then(|b| b.get("lang"))
    else {
        return;
    };
    let mut missing: Vec<_> = lang
        .values
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|tag| {
            // Scripts a style's face does not draw: each needs a pack or
            // a table's own fonts.
            matches!(
                og_i18n::locale::language(tag).as_str(),
                "zh" | "ja"
                    | "ko"
                    | "ar"
                    | "fa"
                    | "ur"
                    | "he"
                    | "yi"
                    | "hi"
                    | "mr"
                    | "ne"
                    | "th"
                    | "ru"
                    | "uk"
                    | "bg"
                    | "sr"
                    | "el"
            )
        })
        .collect();
    if missing.is_empty() {
        return;
    }
    let assets = p.root.join("assets");
    let mut available = Vec::new();
    let mut dirs = vec![assets.clone()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(ty) = entry.file_type() else { continue };
            let path = entry.path();
            if ty.is_dir() {
                dirs.push(path);
            } else if ty.is_file()
                && path.file_name().is_some_and(|n| n == "extension.json")
                && let Some(m) = std::fs::read(&path)
                    .ok()
                    .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                    .filter(|m| m["kind"] == "font")
            {
                // An installed font pack (docs/ui.md §3.4): og.i18n chains it.
                let dir = path.parent().unwrap_or(&assets);
                let faces = m["faces"].as_array().cloned().unwrap_or_default();
                if !faces.is_empty()
                    && faces
                        .iter()
                        .all(|f| f.as_str().is_some_and(|f| dir.join(f).is_file()))
                {
                    available.extend(
                        m["languages"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(|l| l.as_str().map(og_i18n::locale::canonical)),
                    );
                }
            } else if ty.is_file()
                && path.extension().is_some_and(|e| e == "ogt")
                && let Some(table) = std::fs::read_to_string(path)
                    .ok()
                    .and_then(|text| og_i18n::StringTable::parse(&text).ok())
                    .filter(|t| {
                        !t.fonts.is_empty() && t.fonts.iter().all(|f| assets.join(f).is_file())
                    })
            {
                available.push(table.locale);
            }
        }
    }
    missing.retain(|tag| og_i18n::negotiate(&[tag], &available).is_none());
    if missing.is_empty() {
        return;
    }
    let message = format!(
        "{}: no font pack and no locale table with a font chain is installed for it. Install the pack with `og font add <pack>` (`og font list` names them; og.i18n chains an installed pack with no code), or declare a table's own fonts in its fonts line. Verify rendered glyphs.",
        missing.join(", ")
    );
    let text = if json {
        serde_json::json!({"event": "locale-fonts", "locales": missing, "message": message})
            .to_string()
    } else {
        format!("project.ogt: warning: {message}")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// The determinism lint (`lockstep_lint`) over a project's sources, beside
/// the entry, for every game: printed as warnings
/// (`<file>(<line>,<col>): warning OG<code>:`) or, with `json`, as one
/// `lockstep-lint` event (the name it had when only lockstep games were
/// linted; `lockstep` says whether this one is). Never fails a check.
pub(super) fn lockstep_lint(p: &Project, json: bool, to_stderr: bool) {
    let dir = p.entry.parent().unwrap_or(&p.root);
    let (lockstep, found) = crate::lockstep_lint::lint(&p.root, dir);
    if found.is_empty() {
        return;
    }
    let text = if json {
        serde_json::json!({
            "event": "lockstep-lint",
            "lockstep": lockstep,
            "warnings": found.iter().map(|(file, f)| serde_json::json!({
                "file": file, "line": f.line, "col": f.col, "code": f.code, "message": f.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    } else {
        let mut t: Vec<String> = found
            .iter()
            .map(|(file, f)| {
                format!(
                    "{file}({},{}): warning OG{}: {}",
                    f.line, f.col, f.code, f.message
                )
            })
            .collect();
        t.push(if lockstep {
            format!(
                "og check: {} determinism warning(s): a peer that runs these parts from the room, and a replay or a save differs on the other host (`og api show determinism`, §1 and §3); `// og-ok` on a line that only draws",
                found.len()
            )
        } else {
            format!(
                "og check: {} determinism warning(s): where a system that changes the world runs these, a replay, a restored save or a playtest's end hash differs (`og api show determinism`, §1); `// og-ok` on a line that only draws",
                found.len()
            )
        });
        t.join("\n")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// `og check`'s code for a material shader whose surface does not take
/// some of what a scene can ask of it (`shader_capabilities_lint`).
pub const SHADER_CAPABILITY_CODE: u32 = 9103;

/// Every material shader under `assets/` (`.ogt` of `type shader`) whose
/// mesh surface does not take a capability a game sees, read from its lit
/// module as the renderer builds it (`og_render_data::reach::material_shader_lacks`,
/// the check of og-render's capability table): printed as warnings, or
/// with `json` as one `shader-capabilities` event. Never fails a check; a
/// shader the cook would refuse is the cook's to report.
pub(super) fn shader_capabilities_lint(p: &Project, json: bool, to_stderr: bool) {
    let assets = p.root.join("assets");
    let mut found = Vec::new();
    let mut dirs = vec![assets.clone()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let Ok(ty) = entry.file_type() else { continue };
            let path = entry.path();
            if ty.is_dir() {
                if !path.file_name().is_some_and(|n| {
                    n == ".og" || n == "cooked" || n == "og-out" || n == "node_modules"
                }) {
                    dirs.push(path);
                }
                continue;
            }
            if !ty.is_file() || path.extension().is_none_or(|e| e != "ogt") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if og_core::ogt::parse(&text)
                .ok()
                .is_none_or(|d| d.ty != og_cook::shader::SHADER_TYPE)
            {
                continue;
            }
            let file = path
                .strip_prefix(&p.root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            let mut read = |rel: &str| {
                if rel.split('/').any(|c| c == ".." || c.is_empty()) {
                    return Err(format!("'{rel}' is not a path under assets/"));
                }
                std::fs::read_to_string(assets.join(rel)).map_err(|e| e.to_string())
            };
            let Ok(shader) = og_cook::shader::parse_shader(&name, &text, &mut read) else {
                continue;
            };
            let Ok(lacks) = og_render_data::reach::material_shader_lacks(&shader) else {
                continue;
            };
            if !lacks.is_empty() {
                found.push((file, shader.mode, lacks));
            }
        }
    }
    if found.is_empty() {
        return;
    }
    let mode = |m: og_render_data::ShaderMode| match m {
        og_render_data::ShaderMode::Hooks => "hooks",
        og_render_data::ShaderMode::Full { lit: true } => "full lit",
        og_render_data::ShaderMode::Full { lit: false } => "full unlit",
        og_render_data::ShaderMode::Sprite => "sprite",
    };
    let message = |m: og_render_data::ShaderMode, lacks: &[og_render_data::Capability]| {
        let names: Vec<_> = lacks.iter().map(|c| c.name()).collect();
        format!(
            "a mesh drawn with this shader (mode {}) does not take: {}; in a scene that has them, they leave its surface as it is (`og api show render`, §6)",
            mode(m),
            names.join(", ")
        )
    };
    let text = if json {
        serde_json::json!({
            "event": "shader-capabilities",
            "warnings": found.iter().map(|(file, m, lacks)| serde_json::json!({
                "file": file,
                "code": SHADER_CAPABILITY_CODE,
                "mode": mode(*m),
                "lacks": lacks.iter().map(|c| c.name()).collect::<Vec<_>>(),
                "message": message(*m, lacks),
            })).collect::<Vec<_>>(),
        })
        .to_string()
    } else {
        found
            .iter()
            .map(|(file, m, lacks)| {
                format!(
                    "{file}(1,1): warning OG{SHADER_CAPABILITY_CODE}: {}",
                    message(*m, lacks)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// The module-scope state check (`module_state`) over the game's sources
/// beside the entry: each finding an error of the report, as a type error
/// is, so `og check` fails and `og run`, `og test` and `og build` stop.
pub(super) fn module_state(p: &Project, report: &mut Report) {
    let dir = p.entry.parent().unwrap_or(&p.root);
    for (file, f) in crate::module_state::check(&p.root, dir) {
        let message = f.message();
        report.raw.push_str(&format!(
            "{file}({},{}): error OG{}: {message}\n",
            f.line,
            f.col,
            crate::module_state::CODE
        ));
        report.diagnostics.push(Diagnostic {
            file: Some(file),
            line: Some(f.line),
            col: Some(f.col),
            code: crate::module_state::CODE,
            message,
        });
    }
}
