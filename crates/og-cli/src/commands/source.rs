//! Author-file generation and source-bound edits, shared by agents and Stage.
use og_cook::cook::{
    generator::source::{Edit, Source},
    prefab::{self, Flattener},
    source_content, source_index, source_project, source_records, source_transaction,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

pub fn run(args: &[&str], set: bool) -> ExitCode {
    let result = execute(args, set);
    match result {
        Ok(value) => {
            println!("{value}");
            if value["ok"] == false {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            if args.contains(&"--json") {
                println!("{}", json!({"ok":false,"error":error}));
            } else {
                eprintln!("og {}: {error}", if set { "set" } else { "add" });
            }
            ExitCode::from(2)
        }
    }
}

fn execute(args: &[&str], set: bool) -> Result<Value, String> {
    let mut root = PathBuf::from(".");
    let mut positional = Vec::new();
    let (mut from, mut value) = (None, None);
    let mut replace = false;
    let mut args = args.iter().copied();
    while let Some(arg) = args.next() {
        match arg {
            "--json" => {}
            "--project" => root = PathBuf::from(args.next().ok_or("--project needs a directory")?),
            "--from" if !set => {
                from = Some(args.next().ok_or("--from needs a file or - for stdin")?)
            }
            "--value" if !set => value = Some(args.next().ok_or("--value needs JSON")?),
            "--replace" if !set => replace = true,
            a if a.starts_with("--") => {
                return Err(format!(
                    "unknown option {a}; use og {} --help",
                    if set { "set" } else { "add" }
                ));
            }
            a => positional.push(a),
        }
    }
    let project = source_project::Project::open(&root).map_err(|e| e.to_string())?;
    if set {
        return edit(&root, &project, &positional);
    }
    let (file, text) = match positional.as_slice() {
        ["content", id, kind] if from.is_none() && !replace => {
            let (module, stem) = id.split_once('/').ok_or("content ID must be module/path, e.g. base/scenes/level")?;
            let module = project.modules.get(module).ok_or("unknown module; declare it in project.ts modules first")?;
            if !og_cook::cook::source_schemas::all().map(|k| k.name).chain(["scene", "prefab", "table"]).any(|k| k == *kind) {
                return Err(format!("unknown file kind {kind}; use og api list --kind file"));
            }
            let data: Value = serde_json::from_str(value.ok_or("content requires --value JSON; use og api show file:<kind> for its schema")?).map_err(|e| e.to_string())?;
            let file = if module.root.is_empty() { format!("content/{stem}.ts") } else { format!("{}/content/{stem}.ts", module.root) };
            (file, format!("import {{ file }} from '@origingame/engine/author';\nexport default file[{}]({});\n", json!(kind), serde_json::to_string_pretty(&data).unwrap()))
        }
        ["source", file] if file.starts_with("src/") && (file.ends_with(".ts") || file.ends_with(".tsx")) && value.is_none() => {
            (file.to_string(), read_input(from.ok_or("source requires --from file or --from -")?)?)
        }
        ["test", file] if file.starts_with("tests/") && file.ends_with(".og.ts") && value.is_none() => {
            (file.to_string(), match from { Some(f) => read_input(f)?, None => "import { expect, test } from 'bun:test';\nimport { runPlaytest } from '@origingame/engine/playtest';\n\ntest('the game starts', async () => {\n  const run = await runPlaytest({ commands: '0.1:quit' });\n  expect(run.quit).toBe(true);\n});\n".into() })
        }
        _ => return Err("use og add content <module/id> <kind> --value JSON, og add source src/path.ts --from file|-, or og add test tests/name.og.ts; og add --help lists the options".into()),
    };
    if replace {
        let previous = std::fs::read_to_string(root.join(&file))
            .map_err(|e| format!("{file}: {e}; omit --replace to create a new file"))?;
        source_transaction::commit(
            root.as_path(),
            &[source_transaction::FileEdit {
                file: file.clone(),
                digest: Source::new(&file, &previous).digest(),
                text,
            }],
            |_| Ok(()),
        )
        .map_err(|e| {
            e.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        return Ok(json!({"ok":true,"event":"source_replaced","file":file}));
    }
    create(&root, &file, &text)?;
    if positional[0] == "content" {
        let validation = (|| {
            let index = source_index::Index::open(&root, &project).map_err(|e| e.to_string())?;
            let entry = index
                .entries
                .get(positional[1])
                .ok_or("new content was not indexed; use a canonical module/path ID")?;
            let records = source_records::Records::open(&root, &project)?;
            source_content::Evaluator::default()
                .evaluate(
                    &root,
                    &project,
                    entry,
                    &BTreeMap::new(),
                    records.contexts[&entry.module].schema(&entry.factory),
                )
                .map_err(|e| e.to_string())?;
            Ok::<_, String>(())
        })();
        if let Err(error) = validation {
            std::fs::remove_file(root.join(&file)).map_err(|e| e.to_string())?;
            return Err(error);
        }
    }
    Ok(json!({"ok":true,"event":"source_created","file":file}))
}

fn read_input(file: &str) -> Result<String, String> {
    if file != "-" {
        return std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"));
    }
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    Ok(text)
}

fn create(root: &Path, file: &str, text: &str) -> Result<(), String> {
    if file.contains(['\\', ':', '\0'])
        || file.split('/').any(|s| {
            s.is_empty() || s == "." || s == ".." || s.starts_with('.') || s.ends_with(['.', ' '])
        })
    {
        return Err(
            "use a canonical project-relative path with no hidden segments or traversal".into(),
        );
    }
    let mut path = root.to_path_buf();
    let parts: Vec<_> = file.split('/').collect();
    for part in &parts[..parts.len() - 1] {
        path.push(part);
        match std::fs::symlink_metadata(&path) {
            Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&path).map_err(|e| e.to_string())?
            }
            _ => {
                return Err(format!(
                    "{}: expected a directory, not a symlink",
                    path.display()
                ));
            }
        }
    }
    path.push(parts.last().unwrap());
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("{file}: {e}; choose a new path (og add never overwrites)"))?;
    output
        .write_all(text.as_bytes())
        .and_then(|_| output.sync_all())
        .map_err(|e| e.to_string())
}

fn edit(root: &Path, project: &source_project::Project, args: &[&str]) -> Result<Value, String> {
    let index = source_index::Index::open(root, project).map_err(|e| e.to_string())?;
    let (file, path, input) = match args {
        ["project", pointer, input] => ("project.ts".into(), pointer.to_string(), *input),
        [id, pointer, input] => {
            let entry = index.entries.get(*id).ok_or("unknown content ID; use og assets --json")?;
            (entry.file.clone(), pointer.to_string(), *input)
        }
        [id, entity, component, field, input] => {
            if let Some(entry) = index.entries.get(*id)
                && entry.factory == "scatter"
            {
                let text = std::fs::read_to_string(root.join(&entry.file)).map_err(|e| e.to_string())?;
                let source = Source::new(&entry.file, &text);
                let at = source.locations(&[""]).map_err(|e| e.to_string())?.remove(0);
                return Ok(json!({"ok":false,"code":"computed-source","file":entry.file,"line":at.line,"column":at.column,"error":"generated entity; edit the scatter declaration at this source location"}));
            }
            let documents = prefab::documents(root, &BTreeMap::new())?;
            let doc = documents.get(*id).ok_or("expected a scene or prefab content ID")?;
            let schemas = doc.records.iter().map(|(n, r)| (n.clone(), r.schema.clone())).collect();
            let mut flat = Flattener::new(&documents, &schemas);
            let nodes = if doc.prefab { vec![flat.prefab(id).map_err(|e| e.0)?] } else { flat.scene(id, doc).map_err(|e| e.0)? };
            let matches: Vec<_> = nodes.iter().flat_map(|n| n.walk().into_iter().map(move |(p,child)| (if p.is_empty() {n.name.clone()} else {format!("{}/{p}", n.name)},child))).filter(|(path,n)| n.name == *entity || path == entity).collect();
            let [(_, node)] = matches.as_slice() else { return Err(format!("entity {entity:?} is missing or ambiguous; give its root/child path")); };
            if !node.components.get(*component).is_some_and(|c| c.contains_key(*field)) { return Err(format!("entity {entity} has no authored {component}.{field}; add it to the declaration first")); }
            let site = node.sites.get(&format!("{component}.{field}")).ok_or("entity is generated, not source-bound; edit its generator")?;
            (site.file.clone(), site.path.clone(), *input)
        }
        _ => return Err("use og set <content-id> <pointer> <json>, or og set <scene> <entity> <component> <field> <json>; og set --help lists the options".into()),
    };
    let text = std::fs::read_to_string(root.join(&file)).map_err(|e| e.to_string())?;
    let source = Source::new(&file, &text);
    let old = match source.field(&path) {
        Ok(field) => field,
        Err(diagnostic) => {
            return Ok(
                json!({"ok":false,"error":diagnostic.to_string(),"code":diagnostic.code,"file":diagnostic.file,"line":diagnostic.location.line,"column":diagnostic.location.column,"path":diagnostic.path}),
            );
        }
    };
    let value: Value =
        serde_json::from_str(input).map_err(|e| format!("new value must be JSON: {e}"))?;
    let candidate = source
        .patch(
            &text,
            &[Edit {
                path: path.clone(),
                expected: old.value.clone(),
                value: value.clone(),
            }],
        )
        .map_err(|e| e.to_string())?;
    let records = source_records::Records::open(root, project)?;
    source_transaction::commit(
        root,
        &[source_transaction::FileEdit {
            file: file.clone(),
            digest: source.digest(),
            text: candidate,
        }],
        |files| {
            let overlay = files
                .iter()
                .map(|f| (f.file.clone(), f.text.clone()))
                .collect();
            if file == "project.ts" {
                source_project::Project::open_with_overlay(root, &overlay).map_err(|e| vec![e])?;
            } else {
                let candidate = source_index::Index::open_with_overlay(root, project, &overlay)
                    .map_err(|e| vec![e])?;
                let entry = candidate
                    .entries
                    .values()
                    .find(|e| e.file == file)
                    .expect("indexed source");
                source_content::Evaluator::default()
                    .evaluate(
                        root,
                        project,
                        entry,
                        &overlay,
                        records.contexts[&entry.module].schema(&entry.factory),
                    )
                    .map_err(|e| vec![e])?;
            }
            let checked = || -> Result<(), String> {
                let docs = prefab::documents(root, &overlay)?;
                let schemas = docs
                    .values()
                    .flat_map(|d| d.records.iter().map(|(n, r)| (n.clone(), r.schema.clone())))
                    .collect();
                let mut flat = Flattener::new(&docs, &schemas);
                for (id, doc) in &docs {
                    if doc.prefab {
                        flat.prefab(id).map_err(|e| e.0)?;
                    } else {
                        flat.scene(id, doc).map_err(|e| e.0)?;
                    }
                }
                Ok(())
            };
            checked().map_err(|message| {
                vec![og_cook::cook::generator::source::Diagnostic {
                    code: "source-validation",
                    file: file.clone(),
                    path: path.clone(),
                    location: old.location.clone(),
                    message,
                }]
            })
        },
    )
    .map_err(|e| {
        e.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let cooked = crate::cook::cook(root, 0, false);
    Ok(
        json!({"ok":cooked.is_ok(),"event":"source_changed","file":file,"line":old.location.line,"column":old.location.column,"path":path,"old":old.value,"new":value,"cooked":cooked.is_ok(),"error":cooked.err()}),
    )
}
