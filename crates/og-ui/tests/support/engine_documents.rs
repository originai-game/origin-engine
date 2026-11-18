//! The engine's UI documents (`packages/engine`: `src/ui/components` with
//! its groups, and `src/ui/screens`) with their
//! `fixture:default` props and the English strings beside them, and what a
//! preview of one needs: the built-in icon sheet's names and a camera that
//! puts world-anchored documents on screen.

use og_core::glam::Vec3;
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

/// An engine document: its path under `packages/engine` (`src/ui/components/panels/menu.svelte`).
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

/// A camera 12 m back and 6 m up looking at the origin. On portrait
/// surfaces widen its lens, not its distance: preserve horizontal world
/// coverage without fading anchors past their authored distance limits.
pub fn view(width: f32, height: f32) -> WorldView {
    let aspect = width / height;
    // Preserve coverage inside the stage's 24px inset, which takes a
    // larger fraction of a phone's width than of the desktop reference.
    let stage_aspect = (width - 48.0) / (height - 48.0);
    let desktop_aspect = (1280.0 - 48.0) / (720.0 - 48.0);
    let lens = (desktop_aspect / stage_aspect).max(1.0);
    let fov = 2.0 * (25.0_f32.to_radians().tan() * lens).atan().to_degrees();
    WorldView::look_at(
        Vec3::new(0.0, 6.0, 12.0),
        Vec3::ZERO,
        Vec3::Y,
        fov,
        aspect,
        0.1,
        100.0,
    )
}

/// The frames [`preview`] builds: half a second at 60 Hz, past the longest
/// intro an engine document declares (300 ms; a killfeed row's `in:fly` is
/// 200 ms, 12 frames).
const PREVIEW_FRAMES: usize = 30;

/// `doc` mounted as `test:preview` with its `props` as its stores, in a UI
/// that knows every document of `all`, [`PREVIEW_FRAMES`] frames (its
/// intros settled), at `width` × `height`, inside a transparent 24px stage.
/// The renderer supplies the scene ground, below world-anchored paint.
/// Answers the warnings of those frames,
/// each once (an element's hold in every frame). A document that
/// does not mount or build from its props fails the test, named by its
/// path.
pub fn preview(
    ui: &mut Ui,
    all: &[EngineDocument],
    doc: &EngineDocument,
    messages: &dyn bind::Env,
    width: f32,
    height: f32,
) -> Vec<String> {
    for d in all {
        // Under a pack each is parsed against it, as a host loads it.
        let doc = if ui.styled_by(None) {
            d.doc.clone()
        } else {
            ui.parse_document(&d.doc.source, &d.doc.text, Vec::new())
                .unwrap_or_else(|e| panic!("{}: {}", d.doc.source, e.join("\n")))
        };
        ui.define_document(doc);
    }
    ui.set_world_view(Some(view(width, height)));
    let mount = reactive::SourceId::new("test", "preview");
    ui.mount_with_values(mount.clone(), &doc.doc.name, &doc.props)
        .unwrap_or_else(|e| panic!("{}: {e}", doc.path));
    // Hold values against the original document first: its iterators name
    // the collection keys (ActionHints uses `action`, not `id`). Replacing
    // the registration keeps those stores and writable prop bindings.
    let inputs = ui.docs.state().memory.retained.registrations[&mount]
        .inputs
        .clone();
    let root = &doc.doc.nodes[doc.doc.root].id;
    let root = if root.is_empty() { "Document" } else { root };
    let text = format!(
        r#"<meta name="component" content="EngineDocumentPreview">
<div id="PreviewStage"><div id="PreviewBody">
<ui:{name} id="{root}"/>
</div></div>
<style>
#PreviewStage {{ display: flex; flex-direction: column; width: stretch; height: stretch; padding: 24px; }}
#PreviewBody {{ width: stretch; height: stretch; min-width: 0; min-height: 0; contain: layout; }}
</style>"#,
        name = doc.doc.name,
    );
    let mut stage = ui
        .parse_document("<engine-document-preview>", &text, Vec::new())
        .unwrap_or_else(|e| panic!("{}: {}", doc.path, e.join("\n")));
    // Forward the document's prop names verbatim, not native attribute
    // aliases (for example `step`), and retain their writable models.
    let instance = stage
        .nodes
        .iter_mut()
        .find(|node| matches!(&node.kind, doc::NodeKind::Use(name) if name == &doc.doc.name))
        .unwrap();
    instance.args = doc
        .doc
        .bare_names()
        .into_iter()
        .map(|name| (name.clone(), doc::PropValue::Model(name), 3))
        .collect();
    stage.kind = doc.doc.kind;
    stage.layer = doc.doc.layer.clone();
    if !stage.layer.focus.is_empty() {
        stage.layer.focus = format!("PreviewBody/{root}/{}", stage.layer.focus);
    }
    let name = stage.name.clone();
    ui.define_document(stage);
    super::apply(
        ui,
        serde_json::json!({"op": "mount", "mount": mount, "document": name, "inputs": inputs}),
    );
    let mut warnings: Vec<String> = Vec::new();
    let mut world_subject = None;
    for frame in 0..PREVIEW_FRAMES {
        if frame == 1 && doc.path.contains("/world/") {
            // Read the authored anchor, not a component-name-specific camera
            // offset. Edge fixtures keep the point off screen deliberately.
            let anchors = ui.world_anchor_points();
            if !anchors.iter().any(|a| a.clamped) {
                let pivot = anchors.iter().find_map(|a| match a.target {
                    WorldTarget::Point(p) => Some(p + a.offset),
                    WorldTarget::Entity(_) => None,
                });
                let ground = !ui.frame().grounds.is_empty();
                let at = doc.props.get("at").and_then(Value::as_str).and_then(|s| {
                    let p: Vec<f32> = s
                        .split_whitespace()
                        .map(str::parse)
                        .collect::<Result<_, _>>()
                        .ok()?;
                    (p.len() == 3).then(|| Vec3::new(p[0], p[1], p[2]))
                });
                if let Some(pivot) = pivot.or(at.filter(|_| ground)) {
                    world_subject = Some(pivot);
                    let aspect = width / height;
                    let offset = if ground {
                        let extent = ui
                            .frame()
                            .grounds
                            .iter()
                            .map(|g| {
                                (g.rect.width * g.drape.scale.x)
                                    .max(g.rect.height * g.drape.scale.y)
                            })
                            .fold(0.0_f32, f32::max);
                        let distance = (extent / (2.0 * 25.0_f32.to_radians().tan())
                            * (1.0 / aspect).max(1.0)
                            * 1.4)
                            .max(12.0);
                        let pitch = 35.0_f32.to_radians();
                        Vec3::new(0.0, pitch.sin() * distance, pitch.cos() * distance)
                    } else {
                        Vec3::new(0.0, 0.0, 6.0)
                    };
                    ui.set_world_view(Some(WorldView::look_at(
                        pivot + offset,
                        pivot,
                        Vec3::Y,
                        50.0,
                        aspect,
                        0.1,
                        100.0,
                    )));
                }
            }
        } else if frame > 1
            && let Some(pivot) = world_subject
            && let Some(view) = ui.world_view()
            && ui.frame().grounds.iter().any(|g| {
                let pad = 24.0 * ui.theme().text_scale;
                g.bounds.x < pad
                    || g.bounds.y < pad
                    || g.bounds.right() > width - pad
                    || g.bounds.bottom() > height - pad
            })
        {
            // Ground descendants may paint beyond the authored square (a
            // path's endpoint badge does). Keep their full projected bounds,
            // not just the square's corners, clear of the preview padding.
            ui.set_world_view(Some(WorldView::look_at(
                pivot + (view.eye - pivot) * 1.15,
                pivot,
                Vec3::Y,
                50.0,
                width / height,
                0.1,
                100.0,
            )));
        }
        ui.begin(InputFrame::default(), Viewport::new(width, height));
        ui.build_retained_documents(messages)
            .unwrap_or_else(|e| panic!("{}: {e}", doc.path));
        ui.end();
        for warning in &ui.frame().warnings {
            if !warnings.contains(warning) {
                warnings.push(warning.clone());
            }
        }
    }
    warnings
}
