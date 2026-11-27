//! Prefab inspection shares the cook's checked native tree. Extraction edits
//! only an OXC-proven literal entity and leaves computed source untouched.
use og_cook::cook::{
    generator::source::{Edit, Source},
    prefab::{self, Flattener, Node},
    source_content, source_index, source_project, source_records, source_transaction,
};
use serde_json::json;
use std::{io::Write, path::PathBuf, process::ExitCode};

const USAGE: &str = "usage: og prefab check [project] | og prefab show <prefab> [--scene <id> --instance <name>] [--project <dir>] | og prefab extract <scene> <entity> <prefab> [--project <dir>]";

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["check", rest @ ..] => check(rest),
        ["show", rest @ ..] => show(rest),
        ["extract", rest @ ..] => extract(rest),
        _ => Err(USAGE.into()),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("og prefab: {error}");
            ExitCode::from(2)
        }
    }
}

fn flag<'a>(args: &[&'a str], name: &str) -> Option<&'a str> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1])
}

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
    let all = prefab::documents(&root, &Default::default())?;
    let schemas = all
        .values()
        .flat_map(|d| d.records.iter().map(|(n, r)| (n.clone(), r.schema.clone())))
        .collect();
    let mut problems = Vec::new();
    for (id, doc) in &all {
        let mut f = Flattener::new(&all, &schemas);
        let result = if doc.prefab {
            f.prefab(id).map(|_| ())
        } else {
            f.scene(id, doc).map(|_| ())
        };
        if let Err(e) = result {
            problems.push(e.0);
        }
        problems.extend(f.orphans.iter().map(ToString::to_string));
    }
    for p in &problems {
        println!("{p}");
    }
    if problems.is_empty() {
        println!(
            "og prefab check: {} scene(s) and prefab(s), nothing orphaned",
            all.len()
        );
        Ok(ExitCode::SUCCESS)
    } else {
        println!("og prefab check: {} problem(s)", problems.len());
        Ok(ExitCode::FAILURE)
    }
}

fn print(node: &Node) {
    for (path, n) in node.walk() {
        let depth = if path.is_empty() {
            0
        } else {
            path.matches('/').count() + 1
        };
        let pad = "  ".repeat(depth);
        println!("{pad}entity {:?}", n.name);
        let origin = |c: &str, f: &str| {
            n.origins
                .get(&(c.into(), f.into()))
                .map_or("", String::as_str)
        };
        for (field, value) in [("model", n.model.as_ref()), ("scene", n.subscene.as_ref())] {
            if let Some(value) = value {
                println!("{pad}  {field}: {value:?} // {}", origin(field, ""));
            }
        }
        if let Some(value) = n.static_model {
            println!("{pad}  static: {value} // {}", origin("static", ""));
        }
        for (component, fields) in &n.components {
            println!("{pad}  {component} {{");
            for (field, value) in fields {
                println!("{pad}    {field}: {value} // {}", origin(component, field));
            }
            println!("{pad}  }}");
        }
    }
}

fn show(args: &[&str]) -> Result<ExitCode, String> {
    let root = PathBuf::from(flag(args, "--project").unwrap_or("."));
    let [id] = positional(args)[..] else {
        return Err(USAGE.into());
    };
    let all = prefab::documents(&root, &Default::default())?;
    let schemas = all
        .values()
        .flat_map(|d| d.records.iter().map(|(n, r)| (n.clone(), r.schema.clone())))
        .collect();
    let mut f = Flattener::new(&all, &schemas);
    let node = match (flag(args, "--scene"), flag(args, "--instance")) {
        (Some(scene), Some(instance)) => {
            let doc = all
                .get(scene)
                .filter(|d| !d.prefab)
                .ok_or_else(|| format!("no checked scene {scene}"))?;
            f.scene(scene, doc)
                .map_err(|e| e.0)?
                .iter()
                .flat_map(|r| r.walk())
                .find(|(_, n)| {
                    n.name == instance
                        && n.link
                            .as_ref()
                            .is_some_and(|l| l.node == 0 && l.prefab == id)
                })
                .map(|(_, n)| n.clone())
                .ok_or_else(|| format!("{scene} has no instance {instance:?} of {id}"))?
        }
        (None, None) => f.prefab(id).map_err(|e| e.0)?,
        _ => return Err("--scene and --instance go together".into()),
    };
    print(&node);
    for orphan in &f.orphans {
        println!("// warning: {orphan}");
    }
    Ok(ExitCode::SUCCESS)
}

fn extract(args: &[&str]) -> Result<ExitCode, String> {
    let root = PathBuf::from(flag(args, "--project").unwrap_or("."));
    let [scene, name, prefab] = positional(args)[..] else {
        return Err(USAGE.into());
    };
    let project = source_project::Project::open(&root).map_err(|e| e.to_string())?;
    let index = source_index::Index::open(&root, &project).map_err(|e| e.to_string())?;
    let entry = index
        .entries
        .get(scene)
        .filter(|e| e.factory == "scene")
        .ok_or_else(|| format!("no scene {scene}"))?;
    let all = prefab::documents(&root, &Default::default())?;
    fn find(
        entities: &[og_cook::cook::source_schemas::world::scene::EntityInput],
        name: &str,
        parent: &str,
        out: &mut Vec<String>,
    ) {
        for (i, e) in entities.iter().enumerate() {
            let path = format!("{parent}/{i}");
            if e.name == name {
                out.push(path.clone());
            }
            find(&e.children, name, &format!("{path}/children"), out);
        }
    }
    let mut paths = Vec::new();
    find(&all[scene].scene.entities, name, "/entities", &mut paths);
    let [path] = paths.as_slice() else {
        return Err(format!(
            "{scene}: expected exactly one authored entity {name:?}"
        ));
    };
    let text = std::fs::read_to_string(root.join(&entry.file)).map_err(|e| e.to_string())?;
    let source = Source::new(&entry.file, &text);
    let field = source.field(path).map_err(|e| e.to_string())?;
    let entity = field.value.ok_or("entity has no literal source")?;
    let new_scene = source
        .patch(
            &text,
            &[Edit {
                path: path.clone(),
                expected: Some(entity.clone()),
                value: json!({"name":name,"prefab":{"$ogRef":prefab}}),
            }],
        )
        .map_err(|e| e.to_string())?;
    let (module, stem) = prefab
        .split_once('/')
        .ok_or("prefab must be a canonical module/content-id")?;
    if stem
        .split('/')
        .any(|s| s.is_empty() || s == "." || s == ".." || s.ends_with(['.', ' ']))
        || stem.contains(['\\', '#', ':', '*', '\0'])
    {
        return Err("prefab must be a canonical module/content-id".into());
    }
    let module = project.modules.get(module).ok_or("unknown prefab module")?;
    if index
        .entries
        .keys()
        .any(|id| id.to_lowercase() == prefab.to_lowercase())
    {
        return Err(format!("content identity {prefab} already exists"));
    }
    if entry.module != module.id
        && !project.modules[&entry.module]
            .depends_on
            .contains(&module.id)
    {
        return Err(format!(
            "module {} must declare a dependency on {}",
            entry.module, module.id
        ));
    }
    let mut pending = vec![&entity];
    while let Some(value) = pending.pop() {
        match value {
            serde_json::Value::Object(fields) => {
                if let Some(id) = fields.get("$ogRef").and_then(serde_json::Value::as_str) {
                    let owner = id.split('/').next().ok_or("reference has no module")?;
                    if owner != module.id && !module.depends_on.iter().any(|d| d == owner) {
                        return Err(format!(
                            "module {} cannot reference {id}; declare its module dependency first",
                            module.id
                        ));
                    }
                } else {
                    pending.extend(fields.values());
                }
            }
            serde_json::Value::Array(values) => pending.extend(values),
            _ => {}
        }
    }
    let new_file = PathBuf::from(&module.root)
        .join("content")
        .join(format!("{stem}.ts"));
    let prefab_text = format!(
        "import {{ file }} from '@origingame/engine/author';\nexport default file.prefab({});\n",
        serde_json::to_string_pretty(&json!({"entity":entity})).map_err(|e| e.to_string())?
    );
    let records = source_records::Records::open(&root, &project)?;
    let new_path = root.join(&new_file);
    let mut parent = root.clone();
    for component in new_file
        .parent()
        .ok_or("prefab has no parent")?
        .components()
    {
        parent.push(component);
        match std::fs::symlink_metadata(&parent) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(format!(
                    "{}: prefab parent must be a directory, never a symlink",
                    parent.display()
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&parent).map_err(|e| e.to_string())?;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let mut created = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&new_path)
        .map_err(|e| format!("{}: {e}", new_path.display()))?;
    if let Err(error) = created
        .write_all(prefab_text.as_bytes())
        .and_then(|_| created.sync_all())
    {
        drop(created);
        std::fs::remove_file(&new_path).map_err(|e| e.to_string())?;
        return Err(error.to_string());
    }
    drop(created);
    let result = source_transaction::commit(
        &root,
        &[source_transaction::FileEdit {
            file: entry.file.clone(),
            digest: source.digest(),
            text: new_scene,
        }],
        |files| {
            let overlay = files
                .iter()
                .map(|f| (f.file.clone(), f.text.clone()))
                .collect();
            let candidate = source_index::Index::open_with_overlay(&root, &project, &overlay)
                .map_err(|e| vec![e])?;
            let mut evaluator = source_content::Evaluator::default();
            for id in [scene, prefab] {
                let e = &candidate.entries[id];
                evaluator
                    .evaluate(
                        &root,
                        &project,
                        e,
                        &overlay,
                        records.contexts[&e.module].schema(&e.factory),
                    )
                    .map_err(|e| vec![e])?;
            }
            let validate = || -> Result<(), String> {
                let documents = prefab::documents(&root, &overlay)?;
                let schemas = documents
                    .values()
                    .flat_map(|d| d.records.iter().map(|(n, r)| (n.clone(), r.schema.clone())))
                    .collect();
                for (id, doc) in &documents {
                    let mut flattener = Flattener::new(&documents, &schemas);
                    if doc.prefab {
                        flattener.prefab(id).map_err(|e| e.0)?;
                    } else {
                        flattener.scene(id, doc).map_err(|e| e.0)?;
                    }
                    if !flattener.orphans.is_empty() {
                        return Err(flattener
                            .orphans
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("\n"));
                    }
                }
                Ok(())
            };
            validate().map_err(|error| {
                vec![og_cook::cook::generator::source::Diagnostic {
                    code: "prefab-extract",
                    file: entry.file.clone(),
                    path: path.clone(),
                    location: field.location.clone(),
                    message: error,
                }]
            })?;
            Ok(())
        },
    );
    if let Err(errors) = result {
        if std::fs::read_to_string(&new_path).ok().as_deref() == Some(&prefab_text) {
            std::fs::remove_file(&new_path).map_err(|e| e.to_string())?;
        }
        return Err(errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"));
    }
    println!(
        "og prefab extract: {prefab} from {name:?} of {scene}; descendants now use {name}/<path>"
    );
    Ok(ExitCode::SUCCESS)
}
