//! Game-owned sample art for document previews. The components retain their
//! real empty-asset contracts; only this fixture host populates their slots.

use super::*;
use og_ui::{bind, doc};
use serde_json::Value;

struct Assets<'a> {
    messages: &'a Messages,
    textures: Vec<og_draw::TextureHandle>,
}

impl bind::Env for Assets<'_> {
    fn translate(&self, key: &str) -> Option<String> {
        self.messages.0.get(key).cloned()
    }

    fn texture(&self, handle: f64) -> Option<og_draw::TextureHandle> {
        (handle as usize)
            .checked_sub(1)
            .and_then(|i| self.textures.get(i))
            .copied()
    }
}

fn populate(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let sample = match key.as_str() {
                    "art" | "thumb" | "picture" | "keyArt" | "image" => Some(1),
                    "logo" => Some(2),
                    "texture" => Some(3),
                    "portrait" => Some(4),
                    "figure" => Some(5),
                    "map" => Some(6),
                    _ => None,
                };
                if value.as_u64() == Some(0)
                    && let Some(sample) = sample
                {
                    *value = sample.into();
                } else {
                    populate(value);
                }
            }
        }
        Value::Array(rows) => rows.iter_mut().for_each(populate),
        _ => {}
    }
}

/// Mount a document as a game would: populated texture handles, a real icon
/// atlas, and a film still underneath a film's overlay. Answers that still.
pub fn populated_preview(
    ui: &mut Ui,
    scene: &mut RenderScene,
    all: &[EngineDocument],
    doc: &EngineDocument,
    messages: &Messages,
    width: f32,
    height: f32,
) -> Option<og_draw::TextureHandle> {
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core-screens");
    let rpg = engine_dir("sheets/rpg/assets/art");
    let paths = [
        core.join("coast.png"),
        core.join("logo.png"),
        core.join("thumbnail.png"),
        rpg.join("brenna.png"),
        rpg.join("brenna-full.png"),
        rpg.join("map.png"),
    ];
    let textures = paths
        .iter()
        .map(|path| {
            let image = image::open(path).unwrap().to_rgba8();
            scene.add_texture(og_render::TextureData {
                width: image.width(),
                height: image.height(),
                rgba: image.into_raw(),
                color_space: og_render::ColorSpace::Srgb,
                mips: og_render::Mips::Generate,
            })
        })
        .collect();
    let env = Assets { messages, textures };
    let atlas = image::open(engine_dir("sheets/rpg/assets/icons/ink-icons.png"))
        .unwrap()
        .to_rgba8();
    let texture = scene.add_texture(og_render::TextureData {
        width: atlas.width(),
        height: atlas.height(),
        rgba: atlas.into_raw(),
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::Generate,
    });
    ui.set_icons(icons(Some(texture)));
    let docs: Vec<_> = all
        .iter()
        .map(|d| {
            let mut parsed = d.doc.clone();
            let slots: Vec<_> = parsed
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(i, node)| {
                    if node.kind != doc::NodeKind::Element(doc::schema::Kind::Art) {
                        return None;
                    }
                    let kind = node
                        .props
                        .iter()
                        .find(|p| doc::schema::PROPS[p.spec].name == "kind")
                        .and_then(|p| match &p.value {
                            doc::PropValue::Lit(doc::Lit::Str(s)) => Some(s.as_str()),
                            _ => None,
                        });
                    let sample = match kind {
                        Some("logo") => "2",
                        Some("emblem" | "itemIcon") => "3",
                        Some("portrait" | "bust") => "4",
                        _ => "1",
                    };
                    Some((i, sample))
                })
                .collect();
            for (i, sample) in slots {
                parsed.set_prop(i, "texture", sample).unwrap();
            }
            let mut props = d.props.clone();
            populate(&mut props);
            EngineDocument {
                path: d.path.clone(),
                doc: parsed,
                props,
            }
        })
        .collect();
    let selected = docs.iter().find(|d| d.path == doc.path).unwrap();
    let warnings = preview(ui, &docs, selected, &env, width, height);
    assert!(warnings.is_empty(), "{}: {warnings:?}", doc.path);
    doc.path
        .ends_with("screens/film-choices.svelte")
        .then_some(env.textures[0])
}
