//! Source namespaces, delivery ownership and stream residency are independent.
//! Assignment follows checked project patterns, never traversal or bundle order.

use super::*;
use crate::cook::{source_content, source_project};

/// The cook's complete manifest is a build intermediate, not a runtime bundle.
pub(crate) const BUILD_MANIFEST: &str = ".og/cook/manifest.ogt";
pub(crate) const REPORT: &str = ".og/cook/delivery.json";

/// One delivery unit. Selected describes this cook's scope, not a claim that
/// an unselected unit's retained payloads match its current sources.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PackReport {
    pub id: String,
    pub directory: String,
    pub depends_on: Vec<String>,
    pub optional: bool,
    pub selected: bool,
    pub boot: bool,
    pub assets: Vec<String>,
    /// Complete runtime download, including the manifest and lazy payloads.
    pub bytes: u64,
    pub boot_bytes: u64,
    pub stream_bytes: u64,
    pub cross_pack_refs: BTreeMap<String, BTreeSet<String>>,
    /// Runtime files relative to cooked/, including this unit's manifest.
    pub files: Vec<String>,
}

pub(crate) struct Delivery<'a> {
    project: &'a source_project::Project,
    content: &'a source_content::Graph,
    canonical: BTreeMap<&'a str, &'a str>,
    pub(crate) selected: BTreeSet<String>,
}

impl<'a> Delivery<'a> {
    pub(crate) fn new(
        project: &'a source_project::Project,
        content: &'a source_content::Graph,
        pack: Option<&str>,
    ) -> Result<Self, CookError> {
        let mut selected = BTreeSet::new();
        let mut todo = match pack {
            Some(id) if project.packs.contains_key(id) => vec![id.to_owned()],
            Some(id) => return Err(CookError(format!("unknown delivery pack {id:?}"))),
            None => project.pack_order.clone(),
        };
        while let Some(id) = todo.pop() {
            if selected.insert(id.clone()) {
                todo.extend(project.packs[&id].depends_on.iter().cloned());
            }
        }
        Ok(Self {
            project,
            content,
            canonical: content
                .outputs
                .iter()
                .map(|(id, o)| (o.asset.as_str(), id.as_str()))
                .collect(),
            selected,
        })
    }

    pub(crate) fn split(&self) -> bool {
        !self.project.input.packs.is_empty()
    }

    pub(crate) fn owner(&self, id: &str, source: &str) -> Result<String, CookError> {
        let source = self.canonical.get(source).copied().unwrap_or(source);
        let id = self.canonical.get(id).copied().unwrap_or(id);
        let mut owner = None;
        for (name, pack) in &self.project.packs {
            if pack
                .include
                .iter()
                .any(|p| conform::matches(p, id) || conform::matches(p, source))
            {
                if let Some(previous) = owner {
                    return Err(self.error(&format!("{id} matches delivery packs {previous} and {name}; include patterns must not overlap")));
                }
                owner = Some(name.as_str());
            }
        }
        Ok(owner.unwrap_or("base").to_owned())
    }

    pub(crate) fn contains(&self, id: &str, source: &str) -> Result<bool, CookError> {
        Ok(self.selected.contains(&self.owner(id, source)?))
    }

    fn error(&self, message: &str) -> CookError {
        CookError(
            source_content::diagnostic(&self.project.source, "project.ts", "/packs", message)
                .to_string(),
        )
    }

    fn allows(&self, from: &str, to: &str) -> bool {
        let mut seen = BTreeSet::new();
        let mut todo = vec![from];
        while let Some(id) = todo.pop() {
            if id == to {
                return true;
            }
            if seen.insert(id) {
                todo.extend(self.project.packs[id].depends_on.iter().map(String::as_str));
            }
        }
        false
    }

    /// Validate indexed references to delivered outputs. Source-only inputs
    /// have no delivered identity and impose no runtime dependency. The index
    /// is conservative: a declaration naming another delivered output keeps
    /// that edge even if a particular cooker flattened its data into its owner.
    fn validate(
        &self,
        manifest: &Manifest,
        index: &assets::Index,
    ) -> Result<BTreeMap<String, String>, CookError> {
        let mut owners = BTreeMap::new();
        for asset in &manifest.assets {
            for output in &asset.outputs {
                owners.insert(output.id.clone(), self.owner(&output.id, &asset.source)?);
            }
        }
        for asset in &index.assets {
            let Some(from) = owners.get(&asset.id) else {
                continue;
            };
            if !self.selected.contains(from) {
                continue;
            }
            for target in &asset.refs {
                if let Some(to) = owners.get(target)
                    && !self.allows(from, to)
                {
                    return Err(self.error(&format!("{} in pack {from} references {target} in pack {to}; declare a delivery dependsOn edge", asset.id)));
                }
            }
        }
        for group in &manifest.streams {
            let Some(owner) = owners.get(&group.scene) else {
                continue;
            };
            for output in manifest
                .outputs()
                .filter(|o| o.group.as_deref() == Some(&group.name))
            {
                if owners[&output.id] != *owner {
                    return Err(self.error(&format!("stream group {} crosses delivery packs: scene {} belongs to {owner}, {} belongs to {}; keep stream members in their owning bundle", group.name, group.scene, output.id, owners[&output.id])));
                }
            }
        }
        let boot = self.project.input.scene.as_deref().unwrap_or("main");
        for root in &self.content.roots {
            let from = if let Some(rest) = root.path.strip_prefix("/packs/") {
                rest.split('/')
                    .next()
                    .and_then(|n| n.parse::<usize>().ok())
                    .and_then(|i| self.project.input.packs.get(i))
                    .map(|p| p.id.as_str())
            } else if root.path.starts_with("/scenes/") && root.path != format!("/scenes/{boot}") {
                None
            } else {
                Some("base")
            };
            if let Some(from) = from
                && let Some(output) = self.content.outputs.get(&root.target)
                && let Some(to) = owners.get(&output.asset)
                && !self.allows(from, to)
            {
                return Err(self.error(&format!("project root {} in pack {from} references {} in pack {to} without a delivery dependsOn edge", root.path, root.target)));
            }
        }
        Ok(owners)
    }

    pub(crate) fn emit(&self, root: &Path, publish: bool) -> Result<Vec<PackReport>, CookError> {
        let cooked = root.join("cooked");
        let previous: Vec<PackReport> = std::fs::read(root.join(REPORT))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let text =
            std::fs::read_to_string(cooked.join(MANIFEST)).map_err(|e| CookError(e.to_string()))?;
        let full = Manifest::parse(&text).map_err(|e| CookError(e.to_string()))?;
        let index = assets::Index::read(root).map_err(CookError)?;
        let owners = self.validate(&full, &index)?;
        if self.split() {
            write(&root.join(BUILD_MANIFEST), text.as_bytes())?;
        }
        let manifest = if publish && self.split() {
            Manifest::parse(&index.publish_plan(&full).manifest)
                .map_err(|e| CookError(e.to_string()))?
        } else {
            full
        };
        let mut boot = BTreeSet::new();
        let mut todo = vec!["base"];
        while let Some(id) = todo.pop() {
            if boot.insert(id) {
                todo.extend(self.project.packs[id].depends_on.iter().map(String::as_str));
            }
        }
        let mut reports = Vec::new();
        for id in &self.project.pack_order {
            let pack = &self.project.packs[id];
            let directory = if id == "base" {
                String::new()
            } else {
                format!("delivery/{id}")
            };
            let dir = cooked.join(&directory);
            let mut view = manifest.clone();
            for asset in &mut view.assets {
                asset.outputs.retain(|o| owners[&o.id] == *id);
            }
            view.assets.retain(|a| !a.outputs.is_empty());
            view.streams.retain(|g| owners.get(&g.scene) == Some(id));
            if self.split() {
                view.modules = view
                    .assets
                    .iter()
                    .flat_map(|a| a.modules.iter().cloned())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
            }
            view.lightmaps
                .retain(|scene, _| owners.get(scene) == Some(id));
            if id != "base" {
                view.motion = None;
            }
            let own_files: BTreeSet<_> = view.outputs().map(|o| o.file.clone()).collect();
            view.preload.retain(|file| own_files.contains(file));
            view.requires.clear();
            for dependency in &pack.depends_on {
                let file = match (id.as_str(), dependency.as_str()) {
                    ("base", dep) => format!("delivery/{dep}/{MANIFEST}"),
                    (_, "base") => format!("../../{MANIFEST}"),
                    (_, dep) => format!("../{dep}/{MANIFEST}"),
                };
                let dependency_path = if dependency == "base" {
                    cooked.join(MANIFEST)
                } else {
                    cooked.join(format!("delivery/{dependency}/{MANIFEST}"))
                };
                let bytes = std::fs::read(dependency_path)
                    .map_err(|e| CookError(format!("delivery {id} requires {dependency}: {e}")))?;
                view.requires.push(crate::runtime::ManifestFile {
                    file,
                    bytes: bytes.len() as u64,
                    hash: hash_bytes(&bytes),
                    integrity: integrity(&bytes),
                });
            }
            let files: BTreeSet<String> = view
                .outputs()
                .filter(|o| o.group.is_none())
                .map(|o| o.file.clone())
                .chain(view.streams.iter().map(|g| g.file.clone()))
                .chain(view.motion.iter().map(|m| m.file.clone()))
                .collect();
            if id != "base" {
                for file in &files {
                    let bytes = std::fs::read(cooked.join(file))
                        .map_err(|e| CookError(format!("delivery {id}: {file}: {e}")))?;
                    write(&dir.join(file), &bytes)?;
                }
            }
            write(&dir.join(MANIFEST), view.to_text().as_bytes())?;
            let prefix = |file: &str| {
                if directory.is_empty() {
                    file.to_owned()
                } else {
                    format!("{directory}/{file}")
                }
            };
            let cross_pack_refs = index
                .assets
                .iter()
                .filter(|a| owners.get(&a.id) == Some(id))
                .filter_map(|a| {
                    let targets: BTreeSet<_> = a
                        .refs
                        .iter()
                        .filter(|r| owners.get(*r).is_some_and(|to| to != id))
                        .cloned()
                        .collect();
                    (!targets.is_empty()).then(|| (a.id.clone(), targets))
                })
                .collect();
            let files: Vec<_> = files
                .iter()
                .map(|file| prefix(file))
                .chain(std::iter::once(prefix(MANIFEST)))
                .collect();
            let bytes = files.iter().try_fold(0, |total, file| {
                std::fs::metadata(cooked.join(file))
                    .map(|m| total + m.len())
                    .map_err(|e| CookError(format!("delivery {id}: {file}: {e}")))
            })?;
            reports.push(PackReport {
                id: id.clone(),
                directory,
                depends_on: pack.depends_on.clone(),
                optional: pack.optional,
                selected: self.selected.contains(id),
                boot: boot.contains(id.as_str()),
                assets: view.outputs().map(|o| o.id.clone()).collect(),
                bytes,
                boot_bytes: view
                    .outputs()
                    .filter(|o| {
                        o.group.is_none()
                            && !matches!(
                                o.ty.as_str(),
                                "og.SoundChunk" | "og.VideoStream" | "og.VideoChunk"
                            )
                            && !o.id.ends_with("#mips")
                            && (!o.file.ends_with(".face.ogb") || view.preload.contains(&o.file))
                    })
                    .map(|o| (o.file.as_str(), o.bytes))
                    .collect::<BTreeMap<_, _>>()
                    .values()
                    .sum(),
                stream_bytes: view.streams.iter().map(|g| g.bytes).sum(),
                cross_pack_refs,
                files,
            });
        }
        let base =
            std::fs::read_to_string(cooked.join(MANIFEST)).map_err(|e| CookError(e.to_string()))?;
        index.write_publish_plan(
            root,
            &Manifest::parse(&base).map_err(|e| CookError(e.to_string()))?,
        )?;
        // Only delete files previously emitted by this stage. Never walk or
        // follow directory links into user-owned trees.
        let current: BTreeSet<_> = reports.iter().flat_map(|p| &p.files).collect();
        let canonical = cooked
            .canonicalize()
            .map_err(|e| CookError(e.to_string()))?;
        for file in previous.iter().flat_map(|p| &p.files) {
            if file.starts_with("delivery/") && !current.contains(file) {
                let path = cooked.join(file);
                if path.canonicalize().is_ok_and(|p| p.starts_with(&canonical)) && path.is_file() {
                    std::fs::remove_file(&path)
                        .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
                }
            }
        }
        let bytes = serde_json::to_vec(&reports).map_err(|e| CookError(e.to_string()))?;
        write(&root.join(REPORT), &bytes)?;
        Ok(reports)
    }
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), CookError> {
    if std::fs::read(path).ok().as_deref() != Some(bytes) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CookError(format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(path, bytes).map_err(|e| CookError(format!("{}: {e}", path.display())))?;
    }
    Ok(())
}
