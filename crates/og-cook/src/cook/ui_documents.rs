//! The UI's own kinds: a `.svelte` document and a CSS style pack, each
//! cooked into its one blob.

use super::{
    BlobKind, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets, execute,
    format, hash_bytes, stamp, strings,
};
use og_ui::prelude::*;

pub(crate) static STYLE: KindDef = KindDef {
    name: "style",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_style,
};

fn plan_style<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let refs: Vec<String> = serde_json::from_slice(&s.bytes)
        .map_err(|e| e.to_string())
        .and_then(|files| og_ui::style_pack::StylePack::parse(&s.rel, files))
        .map(|pack| pack.assets)
        .unwrap_or_default()
        .into_iter()
        .inspect(|r| {
            parsed_refs
                .entry(s.rel.clone())
                .or_default()
                .insert(r.clone());
        })
        .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
        .collect();
    let st = stamp(&[b"CSS pack 1", &hash, refs.join("\n").as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Style::NAME,
        st,
        Box::new(StyleJob(s)),
    ));
}

/// A style pack's one output.
struct StyleJob<'a>(&'a Source);

impl Job for StyleJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let StyleJob(s) = self;
        let Exec { known, .. } = ex;
        let files =
            serde_json::from_slice(&s.bytes).map_err(|e| CookError(format!("{}: {e}", s.rel)))?;
        let pack = og_ui::style_pack::StylePack::parse(&s.rel, files).map_err(CookError)?;
        let missing: Vec<_> = pack
            .assets
            .iter()
            .filter(|r| !known.contains(r.as_str()))
            .collect();
        if !missing.is_empty() {
            return Err(CookError(format!(
                "{}: CSS url() assets are missing: {} (URLs resolve beside the CSS file)",
                s.rel,
                missing
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        let mut w = og_blob::BlobWriter::new(
            format::Style::type_hash(),
            format::Style::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::StyleMeta {
                text_bytes: s.bytes.len() as u32,
            }],
            format::row_hash::<format::StyleMeta>(),
        )
        .raw("text", s.bytes.clone(), 16);
        Ok((
            w.finish(),
            vec![format!(
                "CSS pack '{}', {} asset(s)",
                pack.name,
                pack.assets.len()
            )],
        ))
    }
}

pub(crate) static DOCUMENT: KindDef = KindDef {
    name: "document",
    extensions: &["svelte"],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_document,
};

fn plan_document<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options,
        sources,
        known,
        style_tokens_hash,
        document_themes,
        components,
        ..
    } = *cx;
    let mut uses = Vec::new();
    let PlanState {
        early_errors,
        parsed_refs,
        ..
    } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    let mut dependencies = Vec::new();
    let imports = match execute::document_imports(s, &text) {
        Ok(imports) => imports,
        Err(e) => {
            early_errors.push((s.rel.clone(), "document", e));
            return;
        }
    };
    for (path, css) in &imports {
        dependencies.push(format!(
            "@import {path}={:016x}",
            hash_bytes(css.as_bytes())
        ));
    }
    if let Err(errors) = og_ui::UiDocument::parse_with_imports(&s.rel, &text, imports).and_then(|doc| {
        doc.validate_tokens(document_themes)?;
        // Each @font-face's fonts join the bundle with it (the first
        // url() the project has is the face's; local() names none).
        for face in &doc.sheet.font_faces {
            let urls: Vec<&String> = face.src.iter().filter(|s| !s.starts_with("local:")).collect();
            if !urls.iter().any(|u| known.contains(u.as_str())) {
                return Err(vec![format!(
                    "{}: @font-face {}: no src is a font of the project ({}; a url() is read relative to the document)",
                    s.rel,
                    face.family,
                    if urls.is_empty() { "it names only local() fonts, and the engine has no system fonts".to_string() } else { urls.iter().map(|u| u.as_str()).collect::<Vec<_>>().join(", ") }
                )]);
            }
            for u in urls {
                dependencies.push(format!("{u}={}", known.contains(u.as_str())));
            }
        }
        for id in &doc.strings {
            let table = sources.iter().find(|source| source.rel == *id && source.kind == &strings::STRINGS)
                .ok_or_else(|| vec![format!("{}: missing string table {id}; add the table named by <meta name=\"strings\">", s.rel)])?;
            let text = std::str::from_utf8(&table.bytes).map_err(|_| vec![format!("{id}: string tables must be UTF-8")])?;
            og_i18n::StringTable::parse(text).map_err(|error| vec![format!("{id}: {error}")])?;
            dependencies.push(format!("{id}={:016x}", table.hash));
        }
        // Each component it uses is loaded with it: the game's document of
        // that name, else the engine's built-in one.
        for name in crate::cook::ui_components::uses(&doc) {
            if let Some(id) = components.get(&name).filter(|id| **id != s.rel) {
                dependencies.push(format!("{id}=<{name}>"));
                uses.push(id.clone());
            }
        }
        if options.publish && !doc.warnings.is_empty() {
            Err(doc.warnings)
        } else {
            Ok(())
        }
    }) {
        early_errors.push((s.rel.clone(), "document", errors.join("\n")));
        return;
    }
    let dependencies = dependencies.join("\n");
    assets::record_dependencies(parsed_refs, &s.rel, &dependencies);
    let st = stamp(&[
        b"svelte document 1",
        &hash,
        &style_tokens_hash,
        dependencies.as_bytes(),
    ]);
    outs.push((
        s.rel.clone(),
        format::Document::NAME,
        st,
        Box::new(DocumentJob(s, uses)),
    ));
}

/// A document's one output.
struct DocumentJob<'a>(&'a Source, Vec<String>);

impl Job for DocumentJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let DocumentJob(s, uses) = self;
        let text = std::str::from_utf8(&s.bytes)
            .map_err(|_| CookError("a document is text (UTF-8)".into()))?;
        let imports = execute::document_imports(s, text).map_err(CookError)?;
        let doc = og_ui::UiDocument::parse_with_imports(&s.rel, text, imports.clone())
            .map_err(|e| CookError(e.join("\n")))?;
        let mut w = og_blob::BlobWriter::new(
            format::Document::type_hash(),
            format::Document::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::DocumentMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::DocumentMeta>(),
        )
        .raw("text", text.as_bytes().to_vec(), 16);
        if !imports.is_empty() {
            let bytes = serde_json::to_vec(&imports).expect("strings serialize");
            w.raw("imports", bytes, 16);
        }
        if !uses.is_empty() {
            let bytes = serde_json::to_vec(uses).expect("strings serialize");
            w.raw("uses", bytes, 16);
        }
        let slots = doc
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, og_ui::doc::NodeKind::Render { .. }))
            .count();
        let mut notes = doc.warnings.clone();
        notes.push(format!(
            "{} '{}', {} node(s), {} timeline(s), {slots} content render(s)",
            match doc.kind {
                og_ui::doc::DocKind::Screen => "screen",
                og_ui::doc::DocKind::Component => "component",
            },
            doc.name,
            doc.nodes.len(),
            doc.timelines.len()
        ));
        Ok((w.finish(), notes))
    }
}

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&STYLE, &DOCUMENT];
