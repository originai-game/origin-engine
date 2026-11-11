//! Cook-only metadata and reference graph. Never part of a shipped bundle.
use super::{CookError, Kind, Source, audio};
use crate::runtime::{Bundle, Manifest, ManifestOutput, asset_id};
use og_core::ogt::{Block, Document, Entry, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const SIDECAR: &str = ".assets.ogt";

/// Preserve the resolved ids in the cook's `id=hash` / `id=present` stamp inputs.
pub(super) fn record_dependencies(
    graph: &mut BTreeMap<String, BTreeSet<String>>,
    source: &str,
    dependencies: &str,
) {
    graph.entry(source.into()).or_default().extend(
        dependencies
            .lines()
            .filter_map(|line| line.rsplit_once('=').map(|(id, _)| id.to_owned())),
    );
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub source: String,
    pub kind: String,
    pub file: String,
    pub group: Option<String>,
    pub bytes: u64,
    pub hash: String,
    pub perceptual: Option<u64>,
    pub refs: BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputedLoad {
    pub source: String,
    pub line: usize,
    pub prefix: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Index {
    pub assets: Vec<Asset>,
    pub roots: BTreeSet<String>,
    pub users: BTreeMap<String, BTreeSet<String>>,
    pub computed: Vec<ComputedLoad>,
}

/// Local transport to the web exporter. The original manifest binds the plan
/// to the cook that produced it; neither this file nor the index is shipped.
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishPlan {
    pub input: String,
    pub manifest: String,
    pub removed_files: BTreeSet<String>,
    pub omitted: Vec<Omitted>,
    pub computed: Vec<ComputedLoad>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Omitted {
    pub id: String,
    pub file: String,
    pub bytes: u64,
}

/// Uses the same path patterns as project.conform: *, ** and folder/.
pub fn publish_keep(doc: &Document) -> Result<Vec<String>, String> {
    let mut patterns = Vec::new();
    if let Some(block) = doc.body.get("publish").and_then(|e| e.block.as_ref()) {
        for entry in block.get_all("keep") {
            if entry.values.is_empty() || entry.block.is_some() {
                return Err(format!(
                    "project.ogt:{}: publish.keep takes quoted path patterns",
                    entry.line
                ));
            }
            for value in &entry.values {
                let pattern = value.as_str().filter(|s| !s.is_empty()).ok_or_else(|| {
                    format!(
                        "project.ogt:{}: publish.keep takes quoted path patterns",
                        entry.line
                    )
                })?;
                let pattern = asset_id(pattern);
                if pattern.starts_with('/')
                    || pattern.contains('\\')
                    || pattern.split('/').any(|p| p == "..")
                {
                    return Err(format!(
                        "project.ogt:{}: publish.keep paths must stay under assets/",
                        entry.line
                    ));
                }
                patterns.push(pattern.to_string());
            }
        }
    }
    Ok(patterns)
}

impl Index {
    pub fn read(root: &Path) -> Result<Self, String> {
        let path = root.join("cooked").join(SIDECAR);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("{}: {e}; run og cook", path.display()))?;
        let doc = Document::parse(&text).map_err(|e| e.to_string())?;
        if doc.ty != "assets" || doc.version != 1 {
            return Err(format!(
                "{}: unsupported asset index; run og cook",
                path.display()
            ));
        }
        let string = |b: &Block, key: &str| -> Result<String, String> {
            b.get(key)
                .and_then(|e| e.string(0).ok())
                .map(str::to_owned)
                .ok_or_else(|| format!("asset index: missing {key}; run og cook"))
        };
        let mut index = Self::default();
        for entry in &doc.body.entries {
            if entry.key == "roots" {
                index.roots = strings(entry)?;
                continue;
            }
            let b = entry
                .block
                .as_ref()
                .ok_or("asset index: expected a block")?;
            let id = entry.string(0).map_err(|e| e.to_string())?.to_owned();
            match entry.key.as_str() {
                "asset" => index.assets.push(Asset {
                    id,
                    source: string(b, "source")?,
                    kind: string(b, "kind")?,
                    file: string(b, "file")?,
                    hash: string(b, "hash")?,
                    bytes: string(b, "bytes")?
                        .parse()
                        .map_err(|_| "asset index: invalid bytes")?,
                    group: b.get("group").map(|_| string(b, "group")).transpose()?,
                    perceptual: b
                        .get("perceptual")
                        .map(|_| {
                            string(b, "perceptual").and_then(|s| {
                                u64::from_str_radix(&s, 16).map_err(|e| e.to_string())
                            })
                        })
                        .transpose()?,
                    refs: b.get("refs").map(strings).transpose()?.unwrap_or_default(),
                }),
                "user" => {
                    index.users.insert(
                        id,
                        b.get("refs").map(strings).transpose()?.unwrap_or_default(),
                    );
                }
                "computed" => index.computed.push(ComputedLoad {
                    source: id,
                    line: string(b, "line")?
                        .parse()
                        .map_err(|_| "asset index: invalid line")?,
                    prefix: string(b, "prefix")?,
                    reason: string(b, "reason")?,
                }),
                key => return Err(format!("asset index: unknown {key}; run og cook")),
            }
        }
        Ok(index)
    }

    pub fn write(&self, cooked: &Path) -> Result<(), CookError> {
        let mut doc = Document::new("assets", 1);
        doc.body.push(list("roots", &self.roots));
        for a in &self.assets {
            let mut b = Block::default();
            for (key, value) in [
                ("source", &a.source),
                ("kind", &a.kind),
                ("file", &a.file),
                ("hash", &a.hash),
            ] {
                b.push(field(key, value));
            }
            b.push(field("bytes", &a.bytes.to_string()));
            if let Some(group) = &a.group {
                b.push(field("group", group));
            }
            if let Some(hash) = a.perceptual {
                b.push(field("perceptual", &format!("{hash:016x}")));
            }
            b.push(list("refs", &a.refs));
            let mut entry = field("asset", &a.id);
            entry.block = Some(b);
            doc.body.push(entry);
        }
        for (source, refs) in &self.users {
            let mut b = Block::default();
            b.push(list("refs", refs));
            let mut entry = field("user", source);
            entry.block = Some(b);
            doc.body.push(entry);
        }
        for c in &self.computed {
            let mut b = Block::default();
            b.push(field("line", &c.line.to_string()));
            b.push(field("prefix", &c.prefix));
            b.push(field("reason", &c.reason));
            let mut entry = field("computed", &c.source);
            entry.block = Some(b);
            doc.body.push(entry);
        }
        let text = doc.to_text().map_err(|e| CookError(e.to_string()))?;
        let path = cooked.join(SIDECAR);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(&text) {
            std::fs::write(&path, text)
                .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
        }
        Ok(())
    }

    pub fn reachable(&self) -> BTreeSet<String> {
        let rows: BTreeMap<_, _> = self.assets.iter().map(|a| (a.id.as_str(), a)).collect();
        let mut seen = BTreeSet::new();
        let mut todo: Vec<_> = self.roots.iter().cloned().collect();
        while let Some(id) = todo.pop() {
            if seen.insert(id.clone())
                && let Some(a) = rows.get(id.as_str())
            {
                todo.extend(a.refs.iter().cloned());
            }
        }
        seen
    }

    pub fn publish_plan(&self, manifest: &Manifest) -> PublishPlan {
        let mut graph = self.clone();
        let reachable = loop {
            let reachable = graph.reachable();
            let before = graph.roots.len();
            // Packs remain byte-identical. Retain every member and its
            // dependencies when any member or its declaring scene is needed.
            for group in &manifest.streams {
                if reachable.contains(&group.scene)
                    || group.assets.iter().any(|id| reachable.contains(id))
                {
                    graph.roots.insert(group.scene.clone());
                    graph.roots.extend(group.assets.iter().cloned());
                    graph.roots.extend(
                        self.assets
                            .iter()
                            .filter(|a| a.group.as_ref() == Some(&group.name))
                            .map(|a| a.id.clone()),
                    );
                }
            }
            if graph.roots.len() == before {
                break reachable;
            }
        };
        let mut kept = manifest.clone();
        let mut omitted = Vec::new();
        for asset in &mut kept.assets {
            asset.outputs.retain(|output| {
                if reachable.contains(&output.id) {
                    true
                } else {
                    omitted.push(Omitted {
                        id: output.id.clone(),
                        file: output.file.clone(),
                        bytes: output.bytes,
                    });
                    false
                }
            });
        }
        kept.assets.retain(|asset| !asset.outputs.is_empty());
        kept.streams.retain(|group| {
            reachable.contains(&group.scene) || group.assets.iter().any(|id| reachable.contains(id))
        });
        let files = |m: &Manifest| -> BTreeSet<String> {
            m.outputs()
                .map(|o| o.file.clone())
                .chain(m.streams.iter().map(|g| g.file.clone()))
                .collect()
        };
        let removed_files = files(manifest).difference(&files(&kept)).cloned().collect();
        omitted.sort_by(|a, b| a.id.cmp(&b.id));
        PublishPlan {
            input: manifest.to_text(),
            manifest: kept.to_text(),
            removed_files,
            omitted,
            computed: self.computed.clone(),
        }
    }

    pub fn write_publish_plan(&self, root: &Path, manifest: &Manifest) -> Result<(), CookError> {
        let dir = root.join(".og");
        std::fs::create_dir_all(&dir).map_err(|e| CookError(e.to_string()))?;
        let path = dir.join("publish-assets.json");
        let bytes = serde_json::to_vec(&self.publish_plan(manifest))
            .map_err(|e| CookError(e.to_string()))?;
        if std::fs::read(&path).ok().as_deref() != Some(bytes.as_slice()) {
            std::fs::write(&path, bytes)
                .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
        }
        Ok(())
    }

    pub fn referenced_by(&self, id: &str) -> BTreeSet<String> {
        self.assets
            .iter()
            .filter(|a| a.refs.contains(id))
            .map(|a| a.id.clone())
            .chain(
                self.users
                    .iter()
                    .filter(|(_, refs)| refs.contains(id))
                    .map(|(file, _)| file.clone()),
            )
            .collect()
    }
}

fn field(key: &str, value: &str) -> Entry {
    Entry::new(key).with(Value::String(value.into()))
}

fn list(key: &str, values: &BTreeSet<String>) -> Entry {
    let mut entry = Entry::new(key);
    entry
        .values
        .extend(values.iter().cloned().map(Value::String));
    entry
}

fn strings(entry: &Entry) -> Result<BTreeSet<String>, String> {
    entry
        .values
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "asset index: expected a string".into())
        })
        .collect()
}

/// Game-owned source files only; no build products, dependency trees or nested projects.
pub fn game_typescript(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.')
                || matches!(
                    name.as_str(),
                    "node_modules" | "cooked" | "dist" | "og-out" | "target"
                )
            {
                continue;
            }
            let ty = entry.file_type().map_err(|e| e.to_string())?;
            let path = entry.path();
            if ty.is_dir() {
                if path != root && path.join("project.ogt").exists() {
                    continue;
                }
                walk(root, &path, out)?;
            } else if ty.is_file()
                && !name.ends_with(".d.ts")
                && matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("ts" | "tsx" | "js" | "jsx" | "mts" | "mjs")
                )
            {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn text_values(block: &Block, values: &mut Vec<String>) {
    for entry in &block.entries {
        values.extend(
            entry
                .values
                .iter()
                .filter_map(|v| v.as_str().or(v.as_word()))
                .map(str::to_owned),
        );
        if let Some(block) = &entry.block {
            text_values(block, values);
        }
    }
}

fn json_values(value: &serde_json::Value, values: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => values.push(value.clone()),
        serde_json::Value::Array(items) => {
            for item in items {
                json_values(item, values);
            }
        }
        serde_json::Value::Object(items) => {
            for item in items.values() {
                json_values(item, values);
            }
        }
        _ => {}
    }
}

fn resolve(
    value: &str,
    source: &str,
    known: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let value = asset_id(value);
    let relative = super::normalise_rel(&format!(
        "{}/{}",
        source.rsplit_once('/').map_or("", |(dir, _)| dir),
        value
    ));
    // Retain both if an asset field's relative/root interpretation is ambiguous.
    known
        .get(value)
        .into_iter()
        .chain(known.get(&relative))
        .flatten()
        .cloned()
        .collect()
}

fn image_hash(image: og_render::TextureData) -> Option<u64> {
    let image = image::RgbaImage::from_raw(image.width, image.height, image.rgba)?;
    let gray = image::DynamicImage::ImageRgba8(image).to_luma8();
    let small = image::imageops::resize(&gray, 9, 8, image::imageops::FilterType::Triangle);
    let mut hash = 0;
    for y in 0..8 {
        for x in 0..8 {
            if small.get_pixel(x, y)[0] > small.get_pixel(x + 1, y)[0] {
                hash |= 1 << (y * 8 + x);
            }
        }
    }
    Some(hash)
}

fn sound_hash(samples: &[i16], channels: usize) -> Option<u64> {
    let frames = samples.len() / channels;
    if frames == 0 {
        return None;
    }
    let mut envelope = [0.0; 65];
    for (n, energy) in envelope.iter_mut().enumerate() {
        let start = n * frames / 65;
        let end = ((n + 1) * frames / 65).max(start + 1).min(frames);
        let window = &samples[start * channels..end * channels];
        *energy = window.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / window.len() as f64;
    }
    Some(envelope.windows(2).enumerate().fold(0, |hash, (i, pair)| {
        hash | (u64::from(pair[0] > pair[1]) << i)
    }))
}

fn fingerprint(bundle: &Bundle, output: &ManifestOutput, source: Option<&Source>) -> Option<u64> {
    match output.ty.as_str() {
        "og.Texture" => image_hash(
            bundle
                .load_texture(&output.id)
                .ok()?
                .to_texture_data()
                .ok()?,
        ),
        "og.Sound" => {
            let sound = bundle.load_sound(&output.id).ok()?;
            if sound.chunked() {
                let source = source?;
                let pcm = audio::decode(&source.rel, &source.bytes).ok()?;
                sound_hash(&pcm.samples, pcm.channels as usize)
            } else {
                let pcm = sound.decode().ok()?;
                sound_hash(&pcm.samples, pcm.channels)
            }
        }
        _ => None,
    }
}

/// The ids a source or an output is loaded as, by its name.
fn known(manifest: &Manifest) -> BTreeMap<String, BTreeSet<String>> {
    let mut known: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for source in &manifest.assets {
        for output in &source.outputs {
            known
                .entry(source.source.clone())
                .or_default()
                .insert(output.id.clone());
            known
                .entry(output.id.clone())
                .or_default()
                .insert(output.id.clone());
        }
    }
    known
}

/// The index of the project's assets and `project.ogt`, before its code is
/// read ([`add_code`]).
pub(super) fn build(
    root: &Path,
    manifest: &Manifest,
    sources: &[Source],
    parsed_refs: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Index, CookError> {
    let known = known(manifest);
    let old = Index::read(root).unwrap_or_default();
    let mut fingerprints: BTreeMap<_, _> = old
        .assets
        .iter()
        .map(|a| (a.hash.clone(), a.perceptual))
        .collect();
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| CookError(e.to_string()))?;
    let mut index = Index::default();
    for source in &manifest.assets {
        if source.kind == super::probes::KIND {
            // Probe ids derive from runtime placement rather than source literals.
            index
                .roots
                .extend(source.outputs.iter().map(|o| o.id.clone()));
            index.computed.push(ComputedLoad {
                source: source.source.clone(),
                line: 1,
                prefix: "probes/".into(),
                reason: "runtime reflection-probe lookup".into(),
            });
        }
        let input = sources.iter().find(|s| s.rel == source.source);
        let mut values: Vec<_> = parsed_refs
            .get(&source.source)
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        if let Some(s) = input {
            if let Ok(text) = std::str::from_utf8(&s.bytes) {
                if let Ok(doc) = Document::parse(text) {
                    text_values(&doc.body, &mut values);
                    if matches!(s.kind, Kind::Scene | Kind::Prefab) {
                        for placed in super::scene::subscenes(text, &|_| None) {
                            index.roots.extend(resolve(&placed, &s.rel, &known));
                        }
                    }
                } else if let Ok(json) = serde_json::from_str(text) {
                    json_values(&json, &mut values);
                }
            }
            if let Some((_, settings)) = &s.settings {
                text_values(&settings.body, &mut values);
            }
        }
        let mut refs: BTreeSet<_> = values
            .iter()
            .flat_map(|v| resolve(v, &source.source, &known))
            .collect();
        // Loading a model, streamed sound, array or video also loads its sub-assets.
        refs.extend(source.outputs.iter().map(|o| o.id.clone()));
        for output in &source.outputs {
            let mut refs = if output.id == source.source {
                refs.clone()
            } else {
                BTreeSet::new()
            };
            let mips = crate::runtime::mips_id(&output.id);
            if known.contains_key(&mips) {
                refs.insert(mips);
            }
            refs.remove(&output.id);
            let hash = format!("{:016x}", output.hash);
            let perceptual = *fingerprints
                .entry(hash.clone())
                .or_insert_with(|| fingerprint(&bundle, output, input));
            index.assets.push(Asset {
                id: output.id.clone(),
                source: source.source.clone(),
                kind: output
                    .ty
                    .strip_prefix("og.")
                    .unwrap_or(&output.ty)
                    .to_lowercase(),
                file: output.file.clone(),
                group: output.group.clone(),
                bytes: output.bytes,
                hash,
                perceptual,
                refs,
            });
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("project.ogt")) {
        let doc = Document::parse(&text).map_err(|e| CookError(e.to_string()))?;
        let keep = publish_keep(&doc).map_err(CookError)?;
        index.roots.extend(
            index
                .assets
                .iter()
                .filter(|a| {
                    keep.iter().any(|p| {
                        super::conform::matches(p, &a.id) || super::conform::matches(p, &a.source)
                    })
                })
                .map(|a| a.id.clone()),
        );
        let mut values = Vec::new();
        text_values(&doc.body, &mut values);
        let refs: BTreeSet<_> = values.iter().flat_map(|v| resolve(v, "", &known)).collect();
        index.roots.extend(refs.iter().cloned());
        index.users.insert("project.ogt".into(), refs);
    }
    index.assets.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(index)
}

/// Adds what the project's code loads to an index built without it: each
/// file's references, the roots they make and its computed loads. Code is
/// the one input read here and nowhere else in the cook, so a change to
/// code alone redoes only this.
pub(super) fn add_code(
    root: &Path,
    manifest: &Manifest,
    index: &mut Index,
) -> Result<(), CookError> {
    let known = known(manifest);
    for path in game_typescript(root).map_err(CookError)? {
        let source = path
            .strip_prefix(root)
            .expect("game source")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| CookError(format!("{}: {e}", path.display())))?;
        let mut refs = BTreeSet::new();
        match super::ts_refs::parse(&path, &text) {
            Ok(parsed) => {
                for literal in parsed.literals {
                    refs.extend(resolve(&literal.value, "", &known));
                }
                for computed in parsed.computed {
                    let prefix = asset_id(&computed.prefix).to_string();
                    refs.extend(
                        index
                            .assets
                            .iter()
                            .filter(|a| a.id.starts_with(&prefix))
                            .map(|a| a.id.clone()),
                    );
                    index.computed.push(ComputedLoad {
                        source: source.clone(),
                        line: computed.line,
                        prefix,
                        reason: "computed og.load path".into(),
                    });
                }
            }
            Err(reason) => {
                refs.extend(index.assets.iter().map(|a| a.id.clone()));
                index.computed.push(ComputedLoad {
                    source: source.clone(),
                    line: 1,
                    prefix: String::new(),
                    reason,
                });
            }
        }
        index.roots.extend(refs.iter().cloned());
        index.users.insert(source, refs);
    }
    Ok(())
}
