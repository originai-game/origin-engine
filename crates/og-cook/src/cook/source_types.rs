//! One contextual type index per source module. A checker includes exactly one
//! of these documents; combining them would merge ambient roles across modules.
use super::{source_index::Index, source_project::Project, source_records::Records};
use og_core::reflect::source::typescript;
use std::collections::BTreeMap;

/// Pure generation from the same discovery snapshots the cook consumes. Does
/// not evaluate content, write files or invent a second field/type registry.
pub fn declarations(
    project: &Project,
    index: &Index,
    records: &Records,
) -> Result<BTreeMap<String, String>, String> {
    let identities = index.declarations();
    let mut documents = BTreeMap::new();
    for module in project.modules.values() {
        let mut text = String::from("// Generated project author types. Do not edit.\n");
        text.push_str(&identities[&module.id]);
        for dependency in &module.depends_on {
            text.push_str(&identities[dependency]);
        }
        let context = &records.contexts[&module.id];
        for (kind, name) in [("scene", "Scene"), ("prefab", "Prefab"), ("table", "Table")] {
            text.push_str(&typescript(context.schema(kind).unwrap().document(), name)?);
        }
        text.push_str("declare module '@origingame/engine/author' {\ninterface ContextualFileInputs {\nreadonly scene: Scene;\nreadonly prefab: Prefab;\nreadonly table: Table;\n}\n}\n");
        documents.insert(module.id.clone(), text);
    }
    Ok(documents)
}
