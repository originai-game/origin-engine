//! Per-file reference entries, generated directly from the checked inputs.
use super::source_schemas;
use crate::grammar::Line;
use og_core::reflect::source::{InputSchema, typescript};
use std::fmt::Write;

fn table(fields: &[Line]) -> String {
    let mut out = String::from("| Field | Arguments | Default | Meaning |\n|---|---|---|---|\n");
    for field in fields {
        let _ = writeln!(
            out,
            "| `{}` | `{}` | `{}` | {} |",
            field.key,
            field.args.replace('|', "\\|"),
            field.default,
            field.doc.replace('|', "\\|").replace('\n', " ")
        );
    }
    out
}

/// `(name, body)` entries for the shipped API reference. Project-specific
/// component fields are bound by `og check`, not guessed from prose.
pub fn entries() -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for kind in source_schemas::all() {
        out.push((
            kind.name.to_string(),
            format!(
                "Checked `file[{:?}](input)` declaration from `@origingame/engine/author`.\n\n```ts\n{}```\n",
                kind.name,
                typescript((kind.schema)().document(), "Input")
                    .map_err(|e| format!("file:{}: {e}", kind.name))?
            ),
        ));
    }
    let scene = InputSchema::<source_schemas::world::scene::SceneInput>::new()?;
    let prefab = InputSchema::<source_schemas::world::scene::PrefabInput>::new()?;
    let table_schema = InputSchema::<source_schemas::world::table::TableInput>::new()?;
    for (name, schema) in [
        ("scene", scene.document()),
        ("prefab", prefab.document()),
        ("table", table_schema.document()),
        (
            "project",
            source_schemas::project::ProjectInput::schema().document(),
        ),
    ] {
        let (language, body) = if name == "project" {
            ("ts", typescript(schema, "Input")?)
        } else {
            // Unbound record roles deliberately reject input (`not: {}`).
            // Show that schema, not a permissive invented TypeScript shape.
            (
                "json",
                serde_json::to_string_pretty(schema).map_err(|e| e.to_string())?,
            )
        };
        out.push((name.into(), format!("Checked `{name}` input. Record roles and project-specific declarations are bound by `og check`.\n\n```{language}\n{body}\n```\n")));
    }
    out.push(("tsx".into(), "UI components use TSX from @origingame/engine/ui. Query named components with `og api show <Name>`; use `og ui types` for project screen props and values.".into()));
    out.push((
        "probe-record".into(),
        format!(
            "Derived bake record, not an authored input.\n\n{}",
            table(super::probes::LINES)
        ),
    ));
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}
