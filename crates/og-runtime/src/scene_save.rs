//! The probe's scene write-back (`save_scene`, `docs/design.md` §3.4):
//! every scene the world was assembled from rewritten from the world by
//! `og_cook::scene_text` — the boot scene (`GameDesc.scene`) first, then
//! each scene `og.world.instantiate` made, each to its own `.ogt`. In each,
//! a field line of a scene entity changes when its value did, and a tuned
//! field the text does not list gets a line. A field's value is read from
//! the live component through its registered layout, as raw lanes, so a
//! value the cook would pack to the same bytes is the same value.
//!
//! Resources are `resource <Name> { ... }` blocks (og-cook's scene entry):
//! a scene that sets one has its lines rewritten, and a tuned resource no
//! scene sets gets a block in the boot scene. With `spawned`, the entities
//! the game spawned at run time are written under the boot scene's marked
//! section (`scene_text::SPAWNED_MARK`), regenerated whole on every save:
//! its own entities as the last cook made them, then the ones spawned
//! since, each with every component the text can give.

use super::codec::{Kind, Layout, RecordKind};
use og_cook::scene_text::{
    FieldValue, Plan, RESOURCE, SpawnedEntity, Tuned, rewrite_with, spawned_names,
};
use og_ecs::{Entity, World};
use og_probe::{SavedFile, SavedScene, SceneWriter};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct RuntimeWriter {
    pub engine: u32,
}

/// The value of field `field` of `bytes` (a record of `layout`) as scene text
/// gives it; `None` for handles, which are the engine's and not the text's.
fn field_value(layout: &Layout, bytes: &[u8], field: &str) -> Option<FieldValue> {
    let (i, f) = layout.field(field)?;
    let at = layout.offset(i);
    let b = bytes.get(at..at + f.ty.size())?;
    if f.ty.is_list() {
        // Its elements on its line; a list of handles is the engine's.
        if is_handle(f.kind) {
            return None;
        }
        let element = f.ty.scalar();
        let lanes = og_core::list_elements(f.ty, b)
            .map(|e| match element {
                og_core::Scalar::F32 => f32::from_le_bytes(e.try_into().expect("4 bytes")) as f64,
                og_core::Scalar::F64 => f64::from_le_bytes(e.try_into().expect("8 bytes")),
                og_core::Scalar::I32 => i32::from_le_bytes(e.try_into().expect("4 bytes")) as f64,
                _ => u32::from_le_bytes(e[..4].try_into().expect("4 bytes")) as f64,
            })
            .collect();
        return Some(FieldValue::Lanes(lanes));
    }
    let f32s = |n: usize| -> Vec<f64> {
        (0..n)
            .map(|k| f32::from_le_bytes(b[4 * k..4 * k + 4].try_into().expect("4 bytes")) as f64)
            .collect()
    };
    Some(match f.kind {
        Kind::F32
        | Kind::Vector2
        | Kind::Vector3
        | Kind::Quaternion
        | Kind::Color
        | Kind::Matrix4 => FieldValue::Lanes(f32s(b.len() / 4)),
        Kind::F64 => FieldValue::Lanes(vec![f64::from_le_bytes(b[..8].try_into().ok()?)]),
        Kind::U32 | Kind::Enum => {
            FieldValue::Lanes(vec![u32::from_le_bytes(b[..4].try_into().ok()?) as f64])
        }
        Kind::I32 => FieldValue::Lanes(vec![i32::from_le_bytes(b[..4].try_into().ok()?) as f64]),
        Kind::Bool => FieldValue::Bool(b.iter().any(|x| *x != 0)),
        Kind::Str => FieldValue::Text(og_core::unpack_str(b).to_string()),
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => return None,
    })
}

/// A handle field's raw bits (an entity, an asset handle).
fn handle_bits(layout: &Layout, bytes: &[u8], field: &str) -> Option<u64> {
    let (i, f) = layout.field(field)?;
    let at = layout.offset(i);
    let b = bytes.get(at..at + f.ty.size())?;
    match b.len() {
        8 => Some(u64::from_le_bytes(b.try_into().ok()?)),
        4 => Some(u64::from(u32::from_le_bytes(b.try_into().ok()?))),
        _ => None,
    }
}

fn is_handle(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound
    )
}

/// The component block names a scene text uses (`Transform`, `og.Light`).
fn blocks(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| {
            let code = l.split('#').next()?.trim();
            let head = code.strip_suffix('{')?.trim();
            (!head.is_empty() && !head.contains(char::is_whitespace)).then(|| head.to_string())
        })
        .collect()
}

/// The resources a scene text sets (`resource <Name> {`).
fn resource_blocks(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| {
            let code = l.split('#').next()?.trim();
            let head = code.strip_suffix('{')?.trim();
            let name = head.strip_prefix("resource ")?.trim().trim_matches('"');
            Some(name.to_string())
        })
        .collect()
}

/// A component's name as scene text writes it: the engine's without `og.`.
fn spelled(schema: &str) -> String {
    schema.strip_prefix("og.").unwrap_or(schema).to_string()
}

/// One scene file to write: its source, text and the world's entities by
/// the names its text gives them.
struct File {
    id: String,
    path: std::path::PathBuf,
    source: String,
    text: String,
    names: BTreeMap<String, u64>,
    boot: bool,
}

impl SceneWriter for RuntimeWriter {
    fn save(
        &mut self,
        world: &World,
        tuned: &[String],
        write: bool,
        spawned: bool,
    ) -> Result<SavedScene, String> {
        let eng = super::engine::engine(self.engine)?;
        let (boot, instances, game_spawned, layouts) = {
            let st = eng.state.borrow();
            (
                st.scene_source.clone(),
                st.scene_instances.clone(),
                st.game_spawned.clone(),
                st.layouts.clone(),
            )
        };
        if instances.is_empty() {
            return Err(
                "no scene was loaded (GameDesc.scene, og.world.instantiate), so there is no scene text to write back"
                    .into(),
            );
        }
        let root = eng.assets.borrow().root.clone();
        // The boot scene first, then each other scene once (its first
        // instance: a scene's names are the world's, so it is instantiated
        // once with names).
        let mut order: Vec<&super::engine::SceneInstance> = Vec::new();
        if let Some(b) = &boot
            && let Some(i) = instances.iter().find(|i| &i.id == b && i.root.is_none())
        {
            order.push(i);
        }
        for i in &instances {
            if !order.iter().any(|o| o.id == i.id) {
                order.push(i);
            }
        }
        let mut files = Vec::new();
        for inst in order {
            let source = format!("assets/{}", inst.id);
            if root.is_empty() {
                return Err(format!(
                    "{source}: this host has no project directory to read the scene's text from"
                ));
            }
            let path = std::path::Path::new(&root).join(&source);
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let names = inst
                .entities
                .iter()
                .map(|b| Entity::from_bits(*b))
                .filter(|e| world.is_alive(*e))
                .filter_map(|e| world.name(e).map(|n| (n.to_string(), e.to_bits())))
                .collect();
            files.push(File {
                id: inst.id.clone(),
                path,
                source,
                text,
                names,
                boot: boot.as_deref() == Some(inst.id.as_str()) && inst.root.is_none(),
            });
        }
        let has_boot = files.first().is_some_and(|f| f.boot);
        let by_schema: BTreeMap<String, Layout> = layouts
            .values()
            .map(|l| (l.schema.clone(), l.clone()))
            .collect();
        let resolve = |name: &str, kind: RecordKind| -> Option<&Layout> {
            layouts
                .get(name)
                .or_else(|| layouts.get(&format!("og.{name}")))
                .filter(|l| l.kind == kind)
        };

        // The spawned section: the boot scene's own section entities (made
        // by its last cook), then those the game spawned, alive.
        let section: Option<Vec<SpawnedEntity>> = if spawned {
            if !has_boot {
                return Err(
                    "save_scene:spawned writes under the boot scene (GameDesc.scene), and the world has none"
                        .into(),
                );
            }
            let boot_file = &files[0];
            let mut picked: Vec<Entity> = spawned_names(&boot_file.text)
                .iter()
                .filter_map(|n| boot_file.names.get(n).map(|b| Entity::from_bits(*b)))
                .collect();
            let roots: BTreeMap<u64, String> = instances
                .iter()
                .filter_map(|i| i.root.map(|r| (r, i.id.clone())))
                .collect();
            // A scene instance's root is the instantiate call's: the game
            // makes it again when it instantiates, and scene text cannot
            // name a scene to instantiate, so it is not written.
            let mut more: Vec<Entity> = game_spawned
                .iter()
                .filter(|b| !roots.contains_key(b))
                .map(|b| Entity::from_bits(*b))
                .filter(|e| world.is_alive(*e))
                .collect();
            og_core::sort::sort_by_key(&mut more, |e| e.index());
            picked.extend(more);
            Some(self.spawned_tree(world, &eng, &picked, &by_schema))
        } else {
            None
        };
        let in_section: BTreeSet<String> = section
            .as_ref()
            .map(|s| {
                fn walk(e: &SpawnedEntity, out: &mut BTreeSet<String>) {
                    out.insert(e.name.clone());
                    e.children.iter().for_each(|c| walk(c, out));
                }
                let mut out = BTreeSet::new();
                s.iter().for_each(|e| walk(e, &mut out));
                out
            })
            .unwrap_or_default();

        // Tuned fields, each to the file whose scene made its entity.
        let mut skipped = Vec::new();
        let mut per_file: Vec<Vec<Tuned>> = vec![Vec::new(); files.len()];
        for t in tuned {
            let Some((entity, rest)) = t.split_once('.') else {
                continue;
            };
            let Some((component, field)) = rest.rsplit_once('.') else {
                // `<Resource>.<field>`: the scenes that set it, else the boot scene.
                let resource = entity;
                let field = rest;
                let setters: Vec<usize> = (0..files.len())
                    .filter(|&k| resource_blocks(&files[k].text).contains(resource))
                    .collect();
                let to = if !setters.is_empty() {
                    setters
                } else if has_boot {
                    vec![0]
                } else {
                    skipped.push(format!(
                        "{t}: a resource no scene sets, and no boot scene (GameDesc.scene) takes new ones"
                    ));
                    continue;
                };
                for k in to {
                    per_file[k].push((RESOURCE.into(), resource.into(), field.into()));
                }
                continue;
            };
            let Some(k) = files.iter().position(|f| f.names.contains_key(entity)) else {
                if !in_section.contains(entity) {
                    skipped.push(format!(
                        "{t}: '{entity}' is an entity no scene made (save_scene:spawned writes the entities the game spawned)"
                    ));
                }
                continue;
            };
            let used = blocks(&files[k].text);
            let short = component.strip_prefix("og.").unwrap_or(component);
            let c = [
                component.to_string(),
                short.to_string(),
                format!("og.{short}"),
            ]
            .into_iter()
            .find(|c| used.contains(c))
            .unwrap_or_else(|| short.to_string());
            per_file[k].push((entity.to_string(), c, field.to_string()));
        }

        let mut out = SavedScene {
            spawned: in_section.len(),
            ..SavedScene::default()
        };
        for (k, file) in files.iter().enumerate() {
            let current = |entity: &str, component: &str, field: &str| -> Option<FieldValue> {
                if entity == RESOURCE {
                    let layout = resolve(component, RecordKind::Resource)?;
                    let sid = world.id_by_name(&layout.schema).ok()?;
                    let bytes = world.resource_bytes(sid).ok()?;
                    return field_value(layout, &bytes, field);
                }
                let bits = *file.names.get(entity)?;
                let layout = resolve(component, RecordKind::Component)?;
                let sid = world.id_by_name(&layout.schema).ok()?;
                let bytes = world.get_bytes(Entity::from_bits(bits), sid).ok()?;
                field_value(layout, &bytes, field)
            };
            let plan = Plan {
                tuned: std::mem::take(&mut per_file[k]),
                add_resources: file.boot,
                spawned: if file.boot { section.clone() } else { None },
            };
            let r = rewrite_with(&file.text, &current, &plan);
            for s in r.skipped {
                if !skipped.contains(&s) {
                    skipped.push(s);
                }
            }
            let written = if write && !r.lines.is_empty() {
                std::fs::write(&file.path, &r.text)
                    .map_err(|e| format!("{}: {e}", file.path.display()))?;
                file.source.clone()
            } else {
                String::new()
            };
            let _ = &file.id;
            out.files.push(SavedFile {
                source: file.source.clone(),
                written,
                text: r.text,
                lines: r.lines,
            });
        }
        out.skipped = skipped;
        Ok(out)
    }
}

impl RuntimeWriter {
    /// The spawned section's entities: each of `picked` with its
    /// components, nested under its parent where the parent is picked too.
    fn spawned_tree(
        &self,
        world: &World,
        eng: &super::engine::Engine,
        picked: &[Entity],
        by_schema: &BTreeMap<String, Layout>,
    ) -> Vec<SpawnedEntity> {
        let set: BTreeSet<u64> = picked.iter().map(|e| e.to_bits()).collect();
        let mut nodes: BTreeMap<u64, SpawnedEntity> = BTreeMap::new();
        let mut parent_of: BTreeMap<u64, u64> = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for &e in picked {
            if !seen.insert(e.to_bits()) {
                continue;
            }
            let mut node = SpawnedEntity {
                name: world.name(e).unwrap_or_default().to_string(),
                ..SpawnedEntity::default()
            };
            if node.name.is_empty() {
                node.notes
                    .push("an entity without a name: nothing can find it by name".into());
            }
            for sid in world.components_of(e).unwrap_or_default() {
                let Ok(schema) = world.schema(sid) else {
                    continue;
                };
                let Some(layout) = by_schema
                    .get(&schema.name)
                    .filter(|l| l.kind == RecordKind::Component)
                else {
                    continue;
                };
                let Ok(bytes) = world.get_bytes(e, sid) else {
                    continue;
                };
                let name = spelled(&schema.name);
                if name == "Parent" {
                    if let Some(p) = handle_bits(layout, &bytes, "entity") {
                        parent_of.insert(e.to_bits(), p);
                    }
                    continue;
                }
                if name == "Model" {
                    // The model line, and Model's fields beside it.
                    let path = handle_bits(layout, &bytes, "asset").and_then(|h| {
                        let i = super::assets::index_of(h as f64)?;
                        let a = eng.assets.borrow();
                        a.entries
                            .get(i)
                            .map(|x| og_cook::runtime::asset_id(&x.path).to_string())
                    });
                    match path {
                        Some(p) => node.model = Some(p),
                        None => {
                            node.notes
                                .push("not written: Model (its asset is not a loaded file)".into());
                            continue;
                        }
                    }
                }
                let handles: Vec<&str> = layout
                    .fields
                    .iter()
                    .filter(|f| is_handle(f.kind) && !(name == "Model" && f.name == "asset"))
                    .map(|f| f.name.as_str())
                    .collect();
                if !handles.is_empty() {
                    node.notes.push(format!(
                        "not written: {name} (its {} hold{} engine handles, which scene text cannot name)",
                        handles.join(", "),
                        if handles.len() == 1 { "s" } else { "" }
                    ));
                    continue;
                }
                let fields: Vec<(String, FieldValue)> = layout
                    .fields
                    .iter()
                    .filter(|f| !(name == "Model" && f.name == "asset"))
                    .filter_map(|f| {
                        field_value(layout, &bytes, &f.name).map(|v| (f.name.clone(), v))
                    })
                    .collect();
                if !fields.is_empty() {
                    node.components.push((name, fields));
                }
            }
            nodes.insert(e.to_bits(), node);
        }
        // Children under their parents, in the order picked.
        let order: Vec<u64> = picked
            .iter()
            .map(|e| e.to_bits())
            .filter({
                let mut once = BTreeSet::new();
                move |b| once.insert(*b)
            })
            .collect();
        for b in order.iter().rev() {
            match parent_of.get(b) {
                Some(p) if set.contains(p) && p != b => {
                    if let Some(child) = nodes.remove(b)
                        && let Some(parent) = nodes.get_mut(p)
                    {
                        parent.children.insert(0, child);
                    }
                }
                Some(p) => {
                    let parent = world.name(Entity::from_bits(*p)).map_or_else(
                        || "an entity without a name".to_string(),
                        |n| format!("'{n}'"),
                    );
                    if let Some(n) = nodes.get_mut(b) {
                        n.notes.push(format!(
                            "its parent {parent} is not in this section: written at the top level"
                        ));
                    }
                }
                None => {}
            }
        }
        order.iter().filter_map(|b| nodes.remove(b)).collect()
    }
}
