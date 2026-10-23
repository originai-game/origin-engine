//! The probe's scene write-back (`save_scene`, `docs/design.md` §3.4): the
//! boot scene's `.ogt` source rewritten from the world by
//! `og_cook::scene_text` — each field line of a scene entity whose value
//! changed, and a line for each field `set`/`patch` tuned that the text does
//! not list. A field's value is read from the live component through its
//! registered layout, as raw lanes, so a value the cook would pack to the
//! same bytes is the same value. Resources have no place in scene text (the
//! cook reads `name`, `entity`, `stream` and `path`), so a tuned resource is
//! reported as skipped.

use super::codec::{Kind, Layout};
use og_cook::scene_text::{FieldValue, Tuned, rewrite};
use og_ecs::{Entity, World};
use og_probe::{SavedScene, SceneWriter};
use std::collections::BTreeSet;

pub(crate) struct RuntimeWriter {
    pub engine: u32,
}

/// The value of field `field` of `bytes` (a record of `layout`) as scene text
/// gives it; `None` for handles, which are the engine's and not the text's.
fn field_value(layout: &Layout, bytes: &[u8], field: &str) -> Option<FieldValue> {
    let (i, f) = layout.field(field)?;
    let at = layout.offset(i);
    let b = bytes.get(at..at + f.ty.size())?;
    let f32s = |n: usize| -> Vec<f64> {
        (0..n)
            .map(|k| f32::from_le_bytes(b[4 * k..4 * k + 4].try_into().expect("4 bytes")) as f64)
            .collect()
    };
    Some(match f.kind {
        Kind::F32
        | Kind::Vector2
        | Kind::Vector3
        | Kind::Quaternion
        | Kind::Color
        | Kind::Matrix4 => FieldValue::Lanes(f32s(b.len() / 4)),
        Kind::F64 => FieldValue::Lanes(vec![f64::from_le_bytes(b[..8].try_into().ok()?)]),
        Kind::U32 | Kind::Enum => {
            FieldValue::Lanes(vec![u32::from_le_bytes(b[..4].try_into().ok()?) as f64])
        }
        Kind::I32 => FieldValue::Lanes(vec![i32::from_le_bytes(b[..4].try_into().ok()?) as f64]),
        Kind::Bool => FieldValue::Bool(b.iter().any(|x| *x != 0)),
        Kind::Str => FieldValue::Text(og_core::unpack_str(b).to_string()),
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => return None,
    })
}

/// The component block names a scene text uses (`Transform`, `og.Light`).
fn blocks(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| {
            let code = l.split('#').next()?.trim();
            let head = code.strip_suffix('{')?.trim();
            (!head.is_empty() && !head.contains(char::is_whitespace)).then(|| head.to_string())
        })
        .collect()
}

impl SceneWriter for RuntimeWriter {
    fn save(&mut self, world: &World, tuned: &[String], write: bool) -> Result<SavedScene, String> {
        let eng = super::engine::engine(self.engine)?;
        let id = eng.state.borrow().scene_source.clone().ok_or(
            "no scene was loaded at boot (GameDesc.scene), so there is no scene text to write back",
        )?;
        let root = eng.assets.borrow().root.clone();
        let source = format!("assets/{id}");
        let path = std::path::Path::new(&root).join(&source);
        let text = if root.is_empty() {
            return Err(format!(
                "{source}: this host has no project directory to read the scene's text from"
            ));
        } else {
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?
        };
        let (names, layouts) = {
            let st = eng.state.borrow();
            (st.names.clone(), st.layouts.clone())
        };
        let used = blocks(&text);
        // `<entity>.<Component>.<field>` with the component spelled as the
        // text spells it; anything else is a resource or a runtime entity.
        let mut skipped = Vec::new();
        let mut fields: Vec<Tuned> = Vec::new();
        for t in tuned {
            let Some((entity, rest)) = t.split_once('.') else {
                continue;
            };
            let Some((component, field)) = rest.rsplit_once('.') else {
                skipped.push(format!("{t}: a resource; scene text has no resources"));
                continue;
            };
            if !names.contains_key(entity) {
                skipped.push(format!(
                    "{t}: '{entity}' is a resource or an entity without a name, neither of which scene text holds"
                ));
                continue;
            }
            let short = component.strip_prefix("og.").unwrap_or(component);
            let spelled = [
                component.to_string(),
                short.to_string(),
                format!("og.{short}"),
            ]
            .into_iter()
            .find(|c| used.contains(c))
            .unwrap_or_else(|| short.to_string());
            fields.push((entity.to_string(), spelled, field.to_string()));
        }
        let resolve = |name: &str| -> Option<&Layout> {
            layouts
                .get(name)
                .or_else(|| layouts.get(&format!("og.{name}")))
                .filter(|l| l.kind == super::codec::RecordKind::Component)
        };
        let current = |entity: &str, component: &str, field: &str| -> Option<FieldValue> {
            let bits = *names.get(entity)?;
            let layout = resolve(component)?;
            let sid = world.id_by_name(&layout.schema).ok()?;
            let bytes = world.get_bytes(Entity::from_bits(bits), sid).ok()?;
            field_value(layout, &bytes, field)
        };
        let mut r = rewrite(&text, &current, &fields);
        r.skipped.splice(0..0, skipped);
        let written = if write && !r.lines.is_empty() {
            std::fs::write(&path, &r.text).map_err(|e| format!("{}: {e}", path.display()))?;
            source.clone()
        } else {
            String::new()
        };
        Ok(SavedScene {
            source,
            written,
            text: r.text,
            lines: r.lines,
            skipped: r.skipped,
        })
    }
}
