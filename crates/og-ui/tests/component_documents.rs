//! The engine's component documents (`packages/engine/src/ui/components`):
//! each parses, takes its strings and builds from its `fixture:default`
//! props at a desktop and a phone width with no author or frame warning.
//! A document that stops cooking, names a missing string, or builds a
//! tree the frame refuses fails here before a game ejects it.

use crate::support;

use og_core::glam::Vec3;
use og_ui::*;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use support::*;

fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui/components")
}

struct Messages(BTreeMap<String, String>);

impl bind::Env for Messages {
    fn translate(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

/// Every component document, parsed, with its default props, and the
/// messages of every string table beside them.
fn documents() -> (Vec<(String, UiDocument, Value)>, Messages) {
    let mut docs = Vec::new();
    let mut messages = BTreeMap::new();
    let mut files: Vec<_> = std::fs::read_dir(directory())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    for path in files {
        let file = path.file_name().unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        if file.ends_with(".svelte") {
            let doc = UiDocument::parse(&file, &text)
                .unwrap_or_else(|e| panic!("{file}: {}", e.join("\n")));
            assert!(doc.warnings.is_empty(), "{file}: {:?}", doc.warnings);
            let props = doc
                .fixtures
                .iter()
                .find(|(n, _)| n == "default")
                .unwrap_or_else(|| panic!("{file}: no fixture:default"))
                .1
                .clone();
            docs.push((file, doc, props));
        } else if file.ends_with(".en.ogt") {
            let table =
                og_i18n::StringTable::parse(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
            messages.extend(table.entries.into_iter().map(|(k, (v, _))| (k, v)));
        }
    }
    (docs, Messages(messages))
}

/// The built-in styles' icon sheet names (8 × 4) on texture `texture`.
fn icons(texture: Option<og_render::TextureHandle>) -> IconAtlas {
    IconAtlas {
        texture,
        columns: 8,
        rows: 4,
        names: og_ui::style_gen::ICON_NAMES.map(String::from).to_vec(),
        ..IconAtlas::default()
    }
}

/// A camera 6 m back and 3 m up looking at the origin: world tags and
/// ground shapes at `at` points near it are on screen.
fn view(width: f32, height: f32) -> WorldView {
    WorldView::look_at(
        Vec3::new(0.0, 3.0, 6.0),
        Vec3::ZERO,
        Vec3::Y,
        50.0,
        width / height,
        0.1,
        100.0,
    )
}

#[test]
fn component_documents_build_from_their_fixtures() {
    let (docs, messages) = documents();
    assert!(!docs.is_empty());
    let mut failures = Vec::new();
    for (file, doc, props) in &docs {
        for (width, height) in [(1280.0, 720.0), (390.0, 844.0)] {
            let mut ui = ui_with_font(test_font_metrics());
            for (_, d, _) in &docs {
                ui.define_document(d.clone());
            }
            ui.set_world_view(Some(view(width, height)));
            ui.set_icons(icons(og_render::TextureHandle::from_slot(1)));
            let mut props = props.clone();
            for _ in 0..4 {
                ui.begin(InputFrame::default(), Viewport::new(width, height));
                ui.component("Preview", &doc.name, &mut props, &messages);
                ui.end();
            }
            let frame = ui.frame();
            if !frame.warnings.is_empty() {
                failures.push(format!("{file} at {width}: {:?}", frame.warnings));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The ground marks drape over the scene (c8e4b7a8c): each document's
/// `<ground>` reaches the renderer as one layer carrying its drape. Fails
/// when a mark is rebuilt as a flat panel or a screen-space box.
#[test]
fn ground_mark_documents_reach_the_renderer_as_draped_grounds() {
    let (docs, messages) = documents();
    for name in ["SelectionRing", "Telegraph", "PathLine", "PlacementGhost"] {
        let (_, doc, props) = docs.iter().find(|(_, d, _)| d.name == name).unwrap();
        let (width, height) = (1280.0, 720.0);
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(doc.clone());
        ui.set_world_view(Some(view(width, height)));
        let mut props = props.clone();
        for _ in 0..4 {
            ui.begin(InputFrame::default(), Viewport::new(width, height));
            ui.component("Preview", &doc.name, &mut props, &messages);
            ui.end();
        }
        let mut overlay = og_render::Overlay::new(width, height, 1.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        let grounds = overlay
            .ui
            .layers
            .iter()
            .filter(|l| l.ground.is_some())
            .count();
        assert_eq!(grounds, 1, "{name}: {grounds} draped ground layer(s)");
    }
}
