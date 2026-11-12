//! The engine's UI documents (`packages/engine`: `src/ui/components` with
//! its groups, and `src/ui/screens`) with their
//! `fixture:default` props and the English strings beside them, and what a
//! preview of one needs: the built-in icon sheet's names and a camera that
//! puts world-anchored documents on screen.

use og_core::glam::Vec3;
use og_ui::prelude::*;
use og_ui::*;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `packages/engine/<dir>`.
pub fn engine_dir(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/engine")
        .join(dir)
}

/// The messages of every `*.en.ogt` string table read.
pub struct Messages(pub BTreeMap<String, String>);

impl bind::Env for Messages {
    fn translate(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

/// An engine document: its path under `packages/engine` (`src/ui/components/menu.svelte`).
pub struct EngineDocument {
    pub path: String,
    pub doc: UiDocument,
    pub props: Value,
}

/// Every document under `dirs` (relative to `packages/engine`, searched
/// below, a game sheet's copies left out), sorted by path, with the
/// messages of the string tables beside them. A document that does not
/// parse, warns or has no `fixture:default` fails the test.
pub fn engine_documents(dirs: &[&str]) -> (Vec<EngineDocument>, Messages) {
    let root = engine_dir("");
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = dirs.iter().map(|d| engine_dir(d)).collect();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if !path.ends_with("sheet") && !path.ends_with("node_modules") {
                    stack.push(path);
                }
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut docs = Vec::new();
    let mut messages = BTreeMap::new();
    for path in files {
        // `/`-separated on every host: a reference is named after it.
        let name = path
            .strip_prefix(&root)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_str().unwrap())
            .collect::<Vec<_>>()
            .join("/");
        let file = path.file_name().unwrap().to_str().unwrap().to_string();
        if file.ends_with(".svelte") {
            let text = std::fs::read_to_string(&path).unwrap();
            let doc = UiDocument::parse(&file, &text)
                .unwrap_or_else(|e| panic!("{name}: {}", e.join("\n")));
            assert!(doc.warnings.is_empty(), "{name}: {:?}", doc.warnings);
            let props = doc
                .fixtures
                .iter()
                .find(|(n, _)| n == "default")
                .unwrap_or_else(|| panic!("{name}: no fixture:default"))
                .1
                .clone();
            docs.push(EngineDocument {
                path: name,
                doc,
                props,
            });
        } else if file.ends_with(".en.ogt") {
            let text = std::fs::read_to_string(&path).unwrap();
            let table =
                og_i18n::StringTable::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            messages.extend(table.entries.into_iter().map(|(k, (v, _))| (k, v)));
        }
    }
    (docs, Messages(messages))
}

/// The built-in styles' icon sheet names (8 × 4) on texture `texture`.
pub fn icons(texture: Option<og_draw::TextureHandle>) -> IconAtlas {
    IconAtlas {
        texture,
        columns: 8,
        rows: 4,
        names: [
            "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem",
            "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play",
            "back", "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull",
            "music", "person",
        ]
        .map(String::from)
        .to_vec(),
        ..IconAtlas::default()
    }
}

/// A camera 6 m back and 3 m up looking at the origin: world tags and
/// ground shapes at `at` points near it are on screen.
pub fn view(width: f32, height: f32) -> WorldView {
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

/// `doc` built from `props` in a UI that knows every document of `all`,
/// four frames (its transitions settled), at `width` × `height`.
pub fn preview(
    ui: &mut Ui,
    all: &[EngineDocument],
    doc: &EngineDocument,
    messages: &Messages,
    width: f32,
    height: f32,
) {
    for d in all {
        ui.define_document(d.doc.clone());
    }
    ui.set_world_view(Some(view(width, height)));
    let mut props = doc.props.clone();
    for _ in 0..4 {
        ui.begin(InputFrame::default(), Viewport::new(width, height));
        ui.component("Preview", &doc.doc.name, &mut props, messages);
        ui.end();
    }
}
