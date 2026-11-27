//! Project lints run alongside `og check`: locale fonts, lockstep,
//! material shader capabilities, and module-scope state.

use super::*;

pub(super) fn locale_fonts_lint(p: &Project, json: bool, to_stderr: bool) {
    let Ok(project) = og_cook::cook::source_project::Project::open(&p.root) else {
        return;
    };
    let Some(lang) = project.input.create.as_ref().and_then(|c| c.lang.as_ref()) else {
        return;
    };
    let mut missing: Vec<_> = lang
        .iter()
        .map(String::as_str)
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
    let mut available = Vec::new();
    if let Ok(index) = og_cook::cook::source_index::Index::open(&p.root, &project)
        && let Ok(tables) = og_cook::cook::strings_extract::read_tables(&p.root, &project, &index)
        && let Ok(bindings) = og_cook::cook::project_assets(&p.root, &project)
    {
        use og_cook::cook::{source_content, source_schemas};
        let mut evaluator = source_content::Evaluator::default();
        for entry in index
            .entries
            .values()
            .filter(|e| e.factory == "font-extension")
        {
            let Ok(content) =
                evaluator.evaluate(&p.root, &project, entry, &Default::default(), None)
            else {
                continue;
            };
            let Ok(file) =
                serde_json::from_value::<source_schemas::File>(content.data().root().into_json())
            else {
                continue;
            };
            let Ok(pack) = serde_json::from_value::<
                source_schemas::world::localization::FontExtensionInput,
            >(file.value) else {
                continue;
            };
            if !pack.faces.is_empty()
                && pack
                    .faces
                    .iter()
                    .all(|f| bindings.get(&f.id).is_some_and(|o| o.kind == "font"))
            {
                available.extend(pack.languages.iter().map(|l| og_i18n::locale::canonical(l)));
            }
        }
        available.extend(tables.into_iter().filter_map(|(_, table)| {
            (!table.fonts.is_empty()
                && table
                    .fonts
                    .iter()
                    .all(|font| bindings.get(font).is_some_and(|o| o.kind == "font")))
            .then_some(table.locale)
        }));
    }
    missing.retain(|tag| og_i18n::negotiate(&[tag], &available).is_none());
    if missing.is_empty() {
        return;
    }
    let message = format!(
        "{}: no font pack and no locale table with a font chain is installed for it. Install the pack with `og font add <pack>` (`og font list` names them; og.i18n chains an installed pack with no code), or declare a table's canonical font refs in its fonts array. Verify rendered glyphs.",
        missing.join(", ")
    );
    let text = if json {
        serde_json::json!({"event": "locale-fonts", "locales": missing, "message": message})
            .to_string()
    } else {
        format!("project.ts: warning: {message}")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// Installed font packs (assets/fonts/<pack>/) named like one this engine
/// bundles but whose faces differ from it: an older engine's copy, which
/// shadows the bundled pack and may lack characters the engine's draws.
pub(super) fn font_packs_lint(p: &Project, json: bool, to_stderr: bool) {
    let Some(bundled) = og_cook::cook::look::fonts_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(p.root.join("assets/fonts")) else {
        return;
    };
    let mut differ: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_owned();
            let ours = bundled.join(&name).join("assets/fonts").join(&name);
            let files = og_cook::cook::source_project::files(&ours).ok()?;
            files
                .iter()
                .filter(|f| {
                    f.extension()
                        .is_some_and(|ext| ext == "ttf" || ext == "otf")
                })
                .any(|f| {
                    std::fs::read(e.path().join(f.strip_prefix(&ours).unwrap())).ok()
                        != std::fs::read(f).ok()
                })
                .then_some(name)
        })
        .collect();
    if differ.is_empty() {
        return;
    }
    differ.sort();
    let message = format!(
        "{}: the installed font pack's faces differ from this engine's pack of that name (an older engine's copy shadows it and may lack characters it draws). Remove assets/fonts/<pack> and content/fonts/<pack>, then run `og font add <pack>` to install this engine's.",
        differ.join(", ")
    );
    let text = if json {
        serde_json::json!({"event": "font-packs", "packs": differ, "message": message}).to_string()
    } else {
        format!("assets/fonts: warning: {message}")
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

/// Every checked file.shader declaration whose
/// mesh surface does not take a capability a game sees, read from its lit
/// module as the renderer builds it (`og_render_data::reach::material_shader_lacks`,
/// the check of og-render's capability table): printed as warnings, or
/// with `json` as one `shader-capabilities` event. Never fails a check; a
/// shader the cook would refuse is the cook's to report.
pub(super) fn shader_capabilities_lint(p: &Project, json: bool, to_stderr: bool) {
    use og_cook::cook::{
        material_shader, source_content, source_index, source_project, source_schemas,
    };
    let Ok(project) = source_project::Project::open(&p.root) else {
        return;
    };
    let Ok(index) = source_index::Index::open(&p.root, &project) else {
        return;
    };
    let mut evaluator = source_content::Evaluator::default();
    let mut found = Vec::new();
    for entry in index
        .entries
        .values()
        .filter(|entry| entry.factory == "shader")
    {
        let Ok(content) = evaluator.evaluate(&p.root, &project, entry, &Default::default(), None)
        else {
            continue;
        };
        let Ok(file) =
            serde_json::from_value::<source_schemas::File>(content.data().root().into_json())
        else {
            continue;
        };
        let Ok(input) = source_schemas::media::shader::Shader::schema().decode(file.value) else {
            continue;
        };
        let Ok(shader) = material_shader(&entry.id, input) else {
            continue;
        };
        let Ok(lacks) = og_render_data::reach::material_shader_lacks(&shader) else {
            continue;
        };
        if !lacks.is_empty() {
            found.push((entry.file.clone(), shader.mode, lacks));
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
