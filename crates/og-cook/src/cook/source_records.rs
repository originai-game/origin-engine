//! Project-local record discovery. Only explicitly listed pure schema modules
//! execute; the game entry never runs. Bound documents retain IDL column/ref
//! semantics, rather than guessing author types from packed numeric layouts.
use super::{generator::Program, source_project::Project, source_schemas};
use og_core::reflect::source::{AuthorSchema, InputSchema};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, path::Path};

mod packing;
pub use packing::PackedRecord;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub kind: String,
    pub name: String,
    /// Ordered exactly as declared, not canonical IR's sorted object keys.
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub kind: String,
    pub default: Value,
    pub options: Map<String, Value>,
    pub capacity: Option<u32>,
    pub list: Option<u32>,
}

pub struct Declaration {
    pub module: String,
    pub file: String,
    pub export: String,
    pub record: Record,
    pub schema: Value,
}

pub struct Context {
    pub scene: InputSchema<source_schemas::world::scene::SceneInput>,
    pub prefab: InputSchema<source_schemas::world::scene::PrefabInput>,
    pub table: InputSchema<source_schemas::world::table::TableInput>,
}

impl Context {
    pub fn schema(&self, kind: &str) -> Option<&dyn AuthorSchema> {
        match kind {
            "scene" => Some(&self.scene),
            "prefab" => Some(&self.prefab),
            "table" => Some(&self.table),
            _ => None,
        }
    }
}

pub struct Records {
    /// Canonical runtime name → the single defining export.
    pub declarations: BTreeMap<String, Declaration>,
    pub contexts: BTreeMap<String, Context>,
    /// Per declaration entry evaluation identity and exact source dependencies.
    pub evaluations: BTreeMap<String, (String, Vec<String>)>,
}

impl Records {
    pub fn open(root: &Path, project: &Project) -> Result<Self, String> {
        Self::open_with_overlay(root, project, &BTreeMap::new())
    }

    pub fn open_with_overlay(
        root: &Path,
        project: &Project,
        overlay: &BTreeMap<String, String>,
    ) -> Result<Self, String> {
        let mut declarations = BTreeMap::<String, Declaration>::new();
        let mut evaluations = BTreeMap::new();
        for file in &project.input.schemas {
            if evaluations.contains_key(file) {
                return Err(format!("{file}: duplicate project.schemas entry"));
            }
            let program = Program::prepare_records(root, project, file, overlay)?;
            let exports: BTreeMap<String, Record> = serde_json::from_value(program.run()?)
                .map_err(|e| format!("{file}: expected named record exports: {e}"))?;
            for (export, record) in exports {
                let at = |message| format!("{file}: export {export}: {message}");
                if !matches!(record.kind.as_str(), "component" | "resource" | "event")
                    || record.name.is_empty()
                {
                    return Err(at("expected a named component, resource or event".into()));
                }
                if og_records::RECORDS.iter().any(|r| r.schema == record.name) {
                    return Err(at(format!("{} is engine-owned", record.name)));
                }
                if let Some(previous) = declarations.get(&record.name) {
                    return Err(at(format!(
                        "duplicate record {}; already defined by {} export {}",
                        record.name, previous.file, previous.export
                    )));
                }
                let schema = record_schema(&record).map_err(at)?;
                declarations.insert(
                    record.name.clone(),
                    Declaration {
                        module: project.module_for(file).id.clone(),
                        file: file.clone(),
                        export,
                        record,
                        schema,
                    },
                );
            }
            evaluations.insert(file.clone(), (program.key, program.dependencies));
        }
        let mut contexts = BTreeMap::new();
        for module in project.modules.values() {
            let mut roles: BTreeMap<&str, Map<String, Value>> = BTreeMap::from([
                ("component", Map::new()),
                ("resource", Map::new()),
                ("event", Map::new()),
            ]);
            for record in og_records::RECORDS {
                roles.get_mut(record.kind).expect("IDL record role").insert(
                    record.schema.into(),
                    source_schemas::reflected::schema_for_record(record.schema)?,
                );
            }
            for declaration in declarations
                .values()
                .filter(|d| d.module == module.id || module.depends_on.contains(&d.module))
            {
                roles
                    .get_mut(declaration.record.kind.as_str())
                    .unwrap()
                    .insert(declaration.record.name.clone(), declaration.schema.clone());
            }
            contexts.insert(module.id.clone(), bind(&roles)?);
        }
        Ok(Self {
            declarations,
            contexts,
            evaluations,
        })
    }
}

fn record_schema(record: &Record) -> Result<Value, String> {
    use source_schemas::reflected::{column_schema, numeric_range};
    let mut fields = Map::new();
    for field in &record.fields {
        let at = |message| format!("{}.{}: {message}", record.name, field.name);
        let column = og_records::COLUMNS
            .iter()
            .find(|c| c.name == field.kind)
            .ok_or_else(|| at(format!("unknown column {}", field.kind)))?;
        if field.capacity.is_some() && (field.kind != "str" || field.capacity == Some(0)) {
            return Err(at("only str has a positive capacity in bytes".into()));
        }
        if field.list == Some(0) {
            return Err(at("a list needs a positive capacity in elements".into()));
        }
        let mut schema =
            column_schema(&field.kind, column.variants, field.capacity.unwrap_or(0)).map_err(at)?;
        let min = field.options.get("min").and_then(Value::as_f64);
        let max = field.options.get("max").and_then(Value::as_f64);
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            return Err(at("min exceeds max".into()));
        }
        if matches!(
            field.kind.as_str(),
            "Vector2" | "Vector3" | "Quaternion" | "Matrix4"
        ) {
            numeric_range(&mut schema["items"], min, max);
        } else if field.kind == "Color" {
            numeric_range(&mut schema["anyOf"][0]["items"], min, max);
        } else {
            numeric_range(&mut schema, min, max);
        }
        if let Some(capacity) = field.list {
            schema = json!({"type":"array", "items":schema, "maxItems":capacity});
        }
        // Runtime default values (e.g. Vector3 objects and null handles) are
        // not author patches (tuples and content refs). Preserve, do not coerce.
        schema["x-og-runtime-default"] = field.default.clone();
        schema["x-og-options"] = json!(field.options);
        if field.name.is_empty() || fields.insert(field.name.clone(), schema).is_some() {
            return Err(at("empty or duplicate field name".into()));
        }
    }
    Ok(
        json!({"type":"object", "properties":fields, "additionalProperties":false,
        "x-og-record":record.name}),
    )
}

fn bind(roles: &BTreeMap<&str, Map<String, Value>>) -> Result<Context, String> {
    use source_schemas::world::{records::bind, table::TableInput};
    let patches =
        |role| json!({"type":"object", "properties":roles[role], "additionalProperties":false});
    let names = json!({"type":"string", "enum":roles["component"].keys().collect::<Vec<_>>()});
    let scene = bind(&[
        ("component", patches("component")),
        ("component-name", names.clone()),
        ("resource", patches("resource")),
    ])?;
    let prefab = bind(&[
        ("component", patches("component")),
        ("component-name", names),
    ])?;
    // Keep the record discriminator and its row schema in the same branch.
    // Each branch gets private root definitions, so both the validator and
    // shared TS emitter see the same correlated record-name/row contract.
    let mut branches = Vec::new();
    let mut definitions = Map::new();
    for (name, schema) in &roles["component"] {
        let table: InputSchema<TableInput> = bind(&[
            ("record-name", json!({"type":"string", "const":name})),
            (
                "row",
                source_schemas::world::table::row_schema(schema.clone()),
            ),
        ])?;
        let mut document = table.document().clone();
        use og_core::reflect::source::schemars::{
            Schema,
            transform::{RecursiveTransform, Transform},
        };
        let prefix = format!("Record{}_", branches.len());
        let mut document_schema: Schema = document.try_into().expect("bound table schema");
        RecursiveTransform(|schema: &mut Schema| {
            if let Some(reference) = schema.get("$ref").and_then(Value::as_str)
                && let Some(name) = reference.strip_prefix("#/$defs/")
            {
                let reference = format!("#/$defs/{prefix}{name}");
                schema.insert("$ref".into(), json!(reference));
            }
        })
        .transform(&mut document_schema);
        document = document_schema.to_value();
        if let Some(Value::Object(defs)) = document.as_object_mut().unwrap().remove("$defs") {
            definitions.extend(
                defs.into_iter()
                    .map(|(name, schema)| (format!("{prefix}{name}"), schema)),
            );
        }
        branches.push(document);
    }
    let table = InputSchema::from_document(json!({"anyOf": branches, "$defs": definitions}))?;
    Ok(Context {
        scene,
        prefab,
        table,
    })
}
