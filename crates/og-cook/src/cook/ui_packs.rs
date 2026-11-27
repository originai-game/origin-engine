//! A checked native style layer, cooked into its one blob.

use super::{
    BlobKind, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, format, stamp,
};
use super::{
    generator, look, source_content, source_index, source_project, source_schemas, ui_themes,
};
use crate::format::Area;
use og_ui_next::{pack::Layer, theme};
use std::collections::BTreeMap;
use std::path::Path;
use ui_themes::Style;

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&PACK];

pub(crate) static PACK: KindDef = KindDef {
    name: "pack",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_pack,
};

/// Planning, grouping and execution consume the same prepared native fields.
pub(crate) fn pack_layer(s: &Source) -> Result<og_ui_next::pack::Layer, CookError> {
    og_ui_next::pack::Layer::from_bytes(&s.bytes).map_err(|e| CookError(format!("{}: {e}", s.rel)))
}

/// Resolve an explicit engine identity. This is shared by look composition,
/// author diagnostics and cooking; no project directory wins by basename.
pub fn builtin(id: &str, cache: &Path) -> Result<Style, String> {
    let mut inputs = Vec::new();
    if id.starts_with("@themes/") {
        let value = ui_themes::read_engine(&theme::Source::Theme(id), &mut inputs)?;
        let t = ui_themes::compile(value, id, &mut |src| {
            ui_themes::read_engine(src, &mut inputs)
        })?;
        let mut style = ui_themes::cook(t, id, cache)?;
        style.inputs = inputs;
        return Ok(style);
    }
    let name = id
        .strip_prefix("@styles/")
        .filter(|name| look::style_slugs().contains(name))
        .ok_or_else(|| format!("{id}: not an explicit built-in pack ID"))?;
    let dir = look::styles_dir().ok_or("engine styles were not found")?;
    let program = generator::Program::prepare_value(dir, &format!("{name}/pack.ts"))?;
    let file: source_schemas::File =
        serde_json::from_value(program.run()?).map_err(|e| e.to_string())?;
    file.check(None).map_err(|e| e.to_string())?;
    if file.kind != "pack" {
        return Err(format!("{id}: expected file.pack"));
    }
    let input: source_schemas::world::pack::PackInput =
        serde_json::from_value(file.value).map_err(|e| e.to_string())?;
    Ok(Style {
        layer: Layer::from_input(input, |r| r.id.clone())?,
        theme: None,
        art: Vec::new(),
        notes: Vec::new(),
        inputs: program.dependencies.iter().map(|p| dir.join(p)).collect(),
    })
}

pub(crate) fn project(
    root: &Path,
    project: &source_project::Project,
    looks: &BTreeMap<String, crate::look::LookFile>,
    prepared: &mut BTreeMap<String, source_content::Content>,
) -> Result<BTreeMap<String, Style>, CookError> {
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let mut evaluator = source_content::Evaluator::default();
    let mut files = BTreeMap::new();
    for entry in index
        .entries
        .values()
        .filter(|e| matches!(e.factory.as_str(), "pack" | "theme"))
    {
        let content = evaluator
            .evaluate(root, project, entry, &BTreeMap::new(), None)
            .map_err(|e| CookError(e.to_string()))?;
        let file: source_schemas::File = serde_json::from_value(content.data().root().into_json())
            .map_err(|e| CookError(e.to_string()))?;
        files.insert(entry.id.clone(), file);
        prepared.insert(entry.id.clone(), content);
    }
    let mut styles = BTreeMap::new();
    let cache = ui_themes::cache_dir(&root.join("assets"));
    for (id, file) in &files {
        let mut inputs = Vec::new();
        let style = (|| -> Result<Style, String> {
            if file.kind == "pack" {
                let input: source_schemas::world::pack::PackInput =
                    serde_json::from_value(file.value.clone()).map_err(|e| e.to_string())?;
                return Ok(Style {
                    layer: Layer::from_input(input, |r| r.id.clone())?,
                    theme: None,
                    art: Vec::new(),
                    notes: Vec::new(),
                    inputs: Vec::new(),
                });
            }
            let mut read = |src: &theme::Source| {
                if let theme::Source::Theme(id) = src
                    && !id.starts_with('@')
                {
                    let file = files
                        .get(*id)
                        .filter(|f| f.kind == "theme")
                        .ok_or_else(|| format!("{id}: no checked file.theme declaration"))?;
                    return Ok(file.value.as_object().expect("checked theme").clone());
                }
                ui_themes::read_engine(src, &mut inputs)
            };
            let t = ui_themes::compile(
                file.value.as_object().expect("checked theme").clone(),
                id,
                &mut read,
            )?;
            let mut style = ui_themes::cook(t, id, &cache)?;
            style.inputs = inputs;
            Ok(style)
        })()
        .map_err(|e| CookError(format!("{}: {e}", prepared[id].entry().file)))?;
        styles.insert(id.clone(), style);
    }
    for id in looks
        .values()
        .filter_map(|look| look.ui.as_deref())
        .filter(|id| id.starts_with('@'))
    {
        if !styles.contains_key(id) {
            styles.insert(id.into(), builtin(id, &cache).map_err(CookError)?);
        }
    }
    Ok(styles)
}

/// Checked styles for editor/CLI diagnostics. The cook calls `project` with
/// its existing bootstrap and retains each evaluated envelope for the graph.
pub fn project_styles(root: &Path) -> Result<BTreeMap<String, Style>, String> {
    let source = source_project::Project::open(root).map_err(|e| e.to_string())?;
    let look::Prepared { mut content, looks } =
        look::prepare(root, &source).map_err(|e| e.to_string())?;
    project(root, &source, &looks, &mut content).map_err(|e| e.to_string())
}

pub(crate) struct Sources {
    pub(crate) sources: Vec<Source>,
    pub(crate) inputs: Vec<std::path::PathBuf>,
    pub(crate) notes: Vec<String>,
}

pub(crate) fn sources(
    root: &Path,
    prepared: &BTreeMap<String, source_content::Content>,
    styles: BTreeMap<String, Style>,
) -> Result<Sources, CookError> {
    let mut sources = Vec::new();
    let mut inputs = Vec::new();
    let mut notes = Vec::new();
    for (id, style) in styles {
        let builtin;
        let entry = match prepared.get(&id) {
            Some(content) => content.entry(),
            None => {
                builtin = source_index::Entry {
                    id: id.clone(),
                    module: "base".into(),
                    file: format!("{id}.ts"),
                    factory: "pack".into(),
                    kind: "pack".into(),
                    digest: String::new(),
                };
                &builtin
            }
        };
        let source = super::pipeline::author::declaration(
            root,
            entry,
            &PACK,
            style.layer.to_bytes().map_err(CookError)?,
        );
        sources.push(source);
        for (resource, bytes) in style.art {
            let mut source =
                super::pipeline::author::declaration(root, entry, &super::texture::TEXTURE, bytes);
            source.rel = format!("{id}#{resource}");
            sources.push(source);
        }
        inputs.extend(style.inputs);
        notes.extend(style.notes.into_iter().map(|n| format!("{id}: {n}")));
    }
    Ok(Sources {
        sources,
        inputs,
        notes,
    })
}

fn plan_pack<'a>(
    cx: &Cx<'a>,
    _: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let refs: Vec<String> = pack_layer(s)
        .map(|l| l.assets().into_iter().map(str::to_owned).collect())
        .unwrap_or_default();
    let refs: Vec<String> = refs
        .iter()
        .map(|r| format!("{r}={}", cx.known.contains(crate::runtime::asset_id(r))))
        .collect();
    // The default pack is in the stamp: a token change cooks every pack's
    // class check again.
    let st = stamp(
        &[Area::Ui],
        &[
            b"style pack 2",
            &hash,
            refs.join("\n").as_bytes(),
            og_ui_next::pack::design::DEFAULT_PACK.as_bytes(),
        ],
    );
    outs.push((s.rel.clone(), format::Pack::NAME, st, Box::new(PackJob(s))));
}

/// One native pack output: every variant's classes checked against its
/// theme over the default in both schemes, with canonical assets present.
struct PackJob<'a>(&'a Source);

impl Job for PackJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let PackJob(s) = self;
        let layer = pack_layer(s)?;
        let missing: Vec<&str> = layer
            .assets()
            .into_iter()
            .filter(|r| !ex.known.contains(crate::runtime::asset_id(r)))
            .collect();
        if !missing.is_empty() {
            return Err(CookError(format!(
                "{}: assets are missing: {} (references must name canonical content IDs)",
                s.rel,
                missing.join(", ")
            )));
        }
        let pack =
            og_ui_next::pack::Pack::new(&[og_ui_next::pack::Layer::default_pack(), layer.clone()]);
        let mut errors = Vec::new();
        for scheme in og_ui_next::pack::Scheme::ALL {
            let (mut classes, ignored) =
                og_ui_next::classes::Classes::new(Some(&pack.theme_json(scheme)))
                    .map_err(CookError)?;
            for e in ignored {
                errors.push(format!("theme ({}): {e}", scheme.name()));
            }
            for (key, class) in &layer.variants {
                // `nine-<name>` names the pack's nine-slice art, not a utility.
                let (images, class) = og_ui_next::pack::split_nine(class);
                for name in images {
                    if !pack.images(scheme).contains_key(name) {
                        errors.push(format!(
                            "variants.{key}: `nine-{name}` names no image in `images` ({})",
                            scheme.name()
                        ));
                    }
                }
                for u in classes.unknown(&class) {
                    errors.push(format!(
                        "variants.{key}: `{u}` is not a class ({})",
                        scheme.name()
                    ));
                }
            }
        }
        errors.dedup();
        if !errors.is_empty() {
            return Err(CookError(format!("{}: {}", s.rel, errors.join("; "))));
        }
        let data = layer.to_bytes().map_err(CookError)?;
        let mut w = og_blob::BlobWriter::new(
            format::Pack::type_hash(),
            format::Pack::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::PackMeta {
                data_bytes: data.len() as u32,
            }],
            format::row_hash::<format::PackMeta>(),
        )
        .raw("data", data, 16);
        Ok((
            w.finish(),
            vec![format!(
                "style pack: {} variant(s), {} image(s), {} face(s), {} asset(s)",
                layer.variants.len(),
                layer.images[0].len().max(layer.images[1].len()),
                layer.fonts.len(),
                layer.assets().len()
            )],
        ))
    }
}
