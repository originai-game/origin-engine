use super::*;
use og_ui::prelude::*;

pub(super) fn plan_style<'a>(
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
    outs.push((s.rel.clone(), format::Style::NAME, st, Work::Style(s)));
}

pub(super) fn plan_document<'a>(
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
            let table = sources.iter().find(|source| source.rel == *id && source.kind == Kind::Strings)
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
        Work::Document(s, uses),
    ));
}

pub(super) fn plan_behaviour<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers every subtree its `use` nodes reach.
    let deps = e6::behaviour_dependencies(s);
    let st = stamp(&[b"behaviour", &hash, deps.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Behaviour::NAME,
        st,
        Work::Behaviour(s),
    ));
}

pub(super) fn plan_dialogue<'a>(
    cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        story_functions,
        story_functions_stamp,
        ..
    } = *cx;
    let st = stamp(&[b"dialogue", &hash, story_functions_stamp]);
    outs.push((
        s.rel.clone(),
        format::Dialogue::NAME,
        st,
        Work::Dialogue(s, story_functions),
    ));
}

pub(super) fn plan_quest<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"quest", &hash]);
    outs.push((s.rel.clone(), format::Quest::NAME, st, Work::Quest(s)));
}

pub(super) fn plan_strings<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[b"strings", &hash]);
    outs.push((s.rel.clone(), format::Strings::NAME, st, Work::Strings(s)));
}

pub(super) fn plan_font_pack<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers its faces' bytes: their cmaps are the coverage.
    let deps = font_pack::dependencies(s);
    let st = stamp(&[b"font-pack 1", &hash, deps.as_bytes()]);
    outs.push((s.rel.clone(), format::FontPack::NAME, st, Work::FontPack(s)));
}

pub(super) fn plan_font<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { options, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    match font_settings(s, &options.font) {
        Ok(f) => {
            // "font 2": the Unicode spaces a font lacks are cooked as empty
            // glyphs; "font 3": the field is of each glyph's outline
            // union; "font 4": negative-bearing fill and post-sign error
            // correction; "font 5": every face carries OpenType data
            // for paragraph layout; "font 6": default substitutions
            // and referenced contextual lookups; "font 7": layout-only
            // tables without runtime outlines; "font 8": the vertical
            // forms (`vert`, `vrt2`) of its characters; "font 9": a glyph keyed
            // by code point and by id is one image in the atlas.
            // Older fonts re-cook.
            let st = stamp(&[b"font 9", f.describe().as_bytes(), &hash]);
            outs.push((s.rel.clone(), format::Font::NAME, st, Work::Font(s, f)));
        }
        Err(e) => early_errors.push((s.rel.clone(), "font", e)),
    }
}
