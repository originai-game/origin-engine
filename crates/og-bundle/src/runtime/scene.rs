//! Decoded scenes, component tables and lightmaps.

use super::*;

/// One entity of a scene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityDesc {
    pub name: String,
    pub parent: Option<usize>,
    /// The asset id of a model this entity instantiates (its default scene
    /// becomes the entity's children).
    pub instance: Option<String>,
}

/// One component type's values in a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentTable {
    pub name: String,
    /// The schema the rows are packed with (from the cook's schema set, or
    /// inferred from the scene text when `inferred`). A host migrates rows to
    /// its registered layout by field name.
    pub schema: og_core::Schema,
    pub inferred: bool,
    /// `(entity index, packed field bytes)`.
    pub rows: Vec<(usize, Vec<u8>)>,
}

impl ComponentTable {
    /// The packed value of `entity`, if it has this component.
    pub fn get(&self, entity: usize) -> Option<&[u8]> {
        self.rows
            .iter()
            .find(|r| r.0 == entity)
            .map(|r| r.1.as_slice())
    }
}

/// A scene's baked lighting as cooked (`Bundle::lightmap`).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedLightmap {
    pub file: crate::lightmap::LightmapFile,
    pub width: u32,
    pub height: u32,
    /// Per page, RGBA f16 bits, rows top to bottom.
    pub pages: Vec<Vec<u16>>,
}

/// A decoded scene: plain data for the host to spawn.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneDesc {
    pub id: String,
    pub name: String,
    /// Parents precede children.
    pub entities: Vec<EntityDesc>,
    /// Sorted by type name.
    pub components: Vec<ComponentTable>,
    /// Its named paths (`path "<name>" { ... }`), in file order.
    pub paths: Vec<(String, og_timeline::PathDesc)>,
    /// The resource fields it sets when instantiated (`resource <Name> {
    /// ... }`), sorted by name.
    pub resources: Vec<ResourceTable>,
}

/// One resource's fields in a scene: `schema` holds only the fields the
/// text lists (the others keep the world's values) and `bytes` their
/// packed values.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceTable {
    pub name: String,
    pub schema: og_core::Schema,
    pub inferred: bool,
    pub bytes: Vec<u8>,
}

impl SceneDesc {
    pub(super) fn decode(id: &str, blob: &Blob) -> Result<SceneDesc, LoadError> {
        let meta = one::<SceneMeta>(blob)?;
        let entities = table::<EntityRow>(blob, "entities")?;
        let refs = table::<RefRow>(blob, "refs")?;
        let comps = table::<ComponentRow>(blob, "components")?;
        let strings = blob.raw("strings")?;
        let bad = |r: String| invalid(blob, r);
        let s = |o: u32, l: u32| {
            pool_str(strings, o, l)
                .map(str::to_string)
                .ok_or_else(|| bad("string out of range".into()))
        };
        let ref_ids = refs
            .iter()
            .map(|r| s(r.id_offset, r.id_len))
            .collect::<Result<Vec<_>, _>>()?;
        let mut out = Vec::with_capacity(entities.len());
        for (i, e) in entities.iter().enumerate() {
            let parent = opt(e.parent);
            if parent.is_some_and(|p| p >= i) {
                return Err(bad(format!("entity {i}: parent does not precede it")));
            }
            out.push(EntityDesc {
                name: s(e.name_offset, e.name_len)?,
                parent,
                instance: match opt(e.instance) {
                    Some(r) => Some(
                        ref_ids
                            .get(r)
                            .cloned()
                            .ok_or_else(|| bad("instance out of range".into()))?,
                    ),
                    None => None,
                },
            });
        }
        let mut components = Vec::with_capacity(comps.len());
        for c in &comps {
            let name = s(c.name_offset, c.name_len)?;
            let json = std::str::from_utf8(blob.raw(&format!("s.{name}"))?)
                .map_err(|_| bad(format!("schema of {name} is not UTF-8")))?;
            let schema = og_core::Schema::from_json(json)
                .map_err(|e| bad(format!("schema of {name}: {e}")))?;
            let hash = (c.layout_hash_hi as u64) << 32 | c.layout_hash_lo as u64;
            if schema.layout_hash() != hash {
                return Err(bad(format!(
                    "schema of {name} does not match its layout hash"
                )));
            }
            let (bytes, stride) = blob.rows(&format!("c.{name}"), hash)?;
            let size = og_core::Layout::of(&schema).size();
            if stride != 4 + size {
                return Err(bad(format!(
                    "{name}: {stride}-byte rows for a {size}-byte layout"
                )));
            }
            let mut rows = Vec::with_capacity(bytes.len() / stride);
            for r in bytes.chunks_exact(stride) {
                let entity = u32::from_le_bytes(r[..4].try_into().expect("4 bytes")) as usize;
                if entity >= out.len() {
                    return Err(bad(format!("{name}: entity index out of range")));
                }
                rows.push((entity, r[4..].to_vec()));
            }
            components.push(ComponentTable {
                name,
                schema,
                inferred: c.inferred != 0,
                rows,
            });
        }
        // Older blobs and scenes without paths have no `paths` section.
        let paths = match blob.raw_opt("paths") {
            None => Vec::new(),
            Some(text) => std::str::from_utf8(text)
                .map_err(|_| bad("the paths text is not UTF-8".into()))
                .and_then(|t| og_timeline::parse_paths(t).map_err(bad))?,
        };
        // Older blobs and scenes without resources have no `resources` table.
        let mut resources = Vec::new();
        if blob.section("resources").is_some() {
            for r in table::<ComponentRow>(blob, "resources")? {
                let name = s(r.name_offset, r.name_len)?;
                let json = std::str::from_utf8(blob.raw(&format!("rs.{name}"))?)
                    .map_err(|_| bad(format!("schema of resource {name} is not UTF-8")))?;
                let schema = og_core::Schema::from_json(json)
                    .map_err(|e| bad(format!("schema of resource {name}: {e}")))?;
                let hash = (r.layout_hash_hi as u64) << 32 | r.layout_hash_lo as u64;
                if schema.layout_hash() != hash {
                    return Err(bad(format!(
                        "schema of resource {name} does not match its layout hash"
                    )));
                }
                let (bytes, _) = blob.rows(&format!("r.{name}"), hash)?;
                let size = og_core::Layout::of(&schema).size();
                if bytes.len() < size {
                    return Err(bad(format!(
                        "resource {name}: {} bytes for a {size}-byte layout",
                        bytes.len()
                    )));
                }
                resources.push(ResourceTable {
                    name,
                    schema,
                    inferred: r.inferred != 0,
                    bytes: bytes[..size].to_vec(),
                });
            }
        }
        Ok(SceneDesc {
            id: id.to_string(),
            name: s(meta.name_offset, meta.name_len)?,
            entities: out,
            components,
            paths,
            resources,
        })
    }
}
