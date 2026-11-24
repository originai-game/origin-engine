//! What the inspector shows and writes, with no UI in it: each reflected
//! field's editor and its shown text, the command text an edit writes
//! through the probe's `set`, the hierarchy by og.Parent, the session's
//! undo history and the toggle key's rule.

use crate::host::codec::Kind;
use og_core::reflect::{FieldSchema, FieldType, Layout};
use og_ecs::{Entity, World};
use std::collections::{BTreeMap, BTreeSet};

/// How a field is edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editor {
    /// A number; `int` refuses a fraction.
    Number {
        int: bool,
    },
    Bool,
    /// A variant by name.
    Enum,
    /// `n` numbers side by side (x, y, z, w).
    Vector(usize),
    /// A quaternion as Euler angles in degrees (three.js's XYZ order).
    Euler,
    /// A colour as `#rrggbb` (sRGB; stored linear).
    Color,
    Text,
    /// An entity: shown by its probe id, a link that selects it.
    Entity,
    /// An asset handle, a matrix or a list: shown, not edited.
    ReadOnly,
}

/// The editor a field takes from its type and, when the host declared
/// its layout, its column kind (which tells a colour from a vector and a
/// handle from a number).
pub fn editor(ty: FieldType, kind: Option<Kind>) -> Editor {
    match kind {
        Some(Kind::Color) if matches!(ty, FieldType::Vec3 | FieldType::Vec4) => {
            return Editor::Color;
        }
        Some(Kind::Entity) if !ty.is_list() => return Editor::Entity,
        Some(k) if k.is_handle() => return Editor::ReadOnly,
        _ => {}
    }
    match ty {
        FieldType::F32 | FieldType::F64 => Editor::Number { int: false },
        FieldType::I32 | FieldType::U32 | FieldType::I64 | FieldType::U64 => {
            Editor::Number { int: true }
        }
        FieldType::Vec2 => Editor::Vector(2),
        FieldType::Vec3 => Editor::Vector(3),
        FieldType::Vec4 => Editor::Vector(4),
        FieldType::Quat => Editor::Euler,
        FieldType::Bool => Editor::Bool,
        FieldType::Enum => Editor::Enum,
        FieldType::Str(_) | FieldType::Ease => Editor::Text,
        FieldType::Mat4 | FieldType::List(..) => Editor::ReadOnly,
    }
}

fn word<const N: usize>(b: &[u8], at: usize) -> [u8; N] {
    b.get(at..at + N)
        .and_then(|s| s.try_into().ok())
        .unwrap_or([0; N])
}

/// A field's numbers, lane by lane (a bool 0 or 1, an enum its index).
pub fn lanes(ty: FieldType, b: &[u8]) -> Vec<f64> {
    match ty {
        FieldType::F32 => vec![f64::from(f32::from_le_bytes(word(b, 0)))],
        FieldType::F64 => vec![f64::from_le_bytes(word(b, 0))],
        FieldType::I32 => vec![f64::from(i32::from_le_bytes(word(b, 0)))],
        FieldType::U32 | FieldType::Enum => vec![f64::from(u32::from_le_bytes(word(b, 0)))],
        FieldType::I64 => vec![i64::from_le_bytes(word(b, 0)) as f64],
        FieldType::U64 => vec![u64::from_le_bytes(word(b, 0)) as f64],
        FieldType::Bool => vec![f64::from(b.first().copied().unwrap_or(0) != 0)],
        FieldType::Vec2 | FieldType::Vec3 | FieldType::Vec4 | FieldType::Quat | FieldType::Mat4 => {
            (0..ty.lanes())
                .map(|i| f64::from(f32::from_le_bytes(word(b, 4 * i))))
                .collect()
        }
        FieldType::Str(_) | FieldType::Ease | FieldType::List(..) => Vec::new(),
    }
}

/// A number as the panel shows it: at most four decimals, no trailing
/// zeros, and no `-0`.
pub fn number_text(v: f64) -> String {
    let r = (v * 10_000.0).round() / 10_000.0;
    let r = if r == 0.0 { 0.0 } else { r };
    let s = format!("{r:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

fn srgb_byte(linear: f64) -> u8 {
    (og_core::color::linear_to_srgb_f64(linear.clamp(0.0, 1.0)) * 255.0).round() as u8
}

fn linear_of(byte: u8) -> f64 {
    og_core::color::srgb_to_linear_f64(f64::from(byte) / 255.0)
}

/// The quaternion `x, y, z, w` as Euler degrees, in three.js's XYZ order
/// (`Euler.setFromQuaternion`).
pub fn euler_degrees(q: &[f64]) -> [f64; 3] {
    let at = |i: usize, d: f64| q.get(i).copied().unwrap_or(d);
    let (mut x, mut y, mut z, mut w) = (at(0, 0.0), at(1, 0.0), at(2, 0.0), at(3, 1.0));
    let n = (x * x + y * y + z * z + w * w).sqrt();
    if n > 0.0 {
        (x, y, z, w) = (x / n, y / n, z / n, w / n);
    }
    let m11 = 1.0 - 2.0 * (y * y + z * z);
    let m12 = 2.0 * (x * y - z * w);
    let m13 = 2.0 * (x * z + y * w);
    let m22 = 1.0 - 2.0 * (x * x + z * z);
    let m23 = 2.0 * (y * z - x * w);
    let m32 = 2.0 * (y * z + x * w);
    let m33 = 1.0 - 2.0 * (x * x + y * y);
    let ey = m13.clamp(-1.0, 1.0).asin();
    let (ex, ez) = if m13.abs() < 0.999_999_9 {
        ((-m23).atan2(m33), (-m12).atan2(m11))
    } else {
        (m32.atan2(m22), 0.0)
    };
    [ex, ey, ez].map(f64::to_degrees)
}

/// Euler degrees in three.js's XYZ order as the quaternion `x, y, z, w`
/// (`Quaternion.setFromEuler`).
pub fn quaternion(degrees: [f64; 3]) -> [f64; 4] {
    let [(s1, c1), (s2, c2), (s3, c3)] = degrees.map(|d| (d.to_radians() / 2.0).sin_cos());
    [
        s1 * c2 * c3 + c1 * s2 * s3,
        c1 * s2 * c3 - s1 * c2 * s3,
        c1 * c2 * s3 + s1 * s2 * c3,
        c1 * c2 * c3 - s1 * s2 * s3,
    ]
}

/// The text of each part of a field as the panel shows it: one per lane of
/// a vector, three angles for a rotation, one for everything else.
/// `id_of` names an entity by its probe id.
pub fn parts(
    editor: Editor,
    field: &FieldSchema,
    bytes: &[u8],
    id_of: &dyn Fn(Entity) -> String,
) -> Vec<String> {
    let l = lanes(field.ty, bytes);
    match editor {
        Editor::Number { .. } => vec![number_text(l.first().copied().unwrap_or(0.0))],
        Editor::Vector(_) => l.iter().map(|v| number_text(*v)).collect(),
        Editor::Euler => euler_degrees(&l).iter().map(|v| number_text(*v)).collect(),
        Editor::Bool => vec![(l.first().copied().unwrap_or(0.0) != 0.0).to_string()],
        Editor::Enum => {
            let i = l.first().copied().unwrap_or(0.0) as usize;
            vec![
                field
                    .variants
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| i.to_string()),
            ]
        }
        Editor::Color => {
            let [r, g, b] = [0, 1, 2].map(|i| srgb_byte(l.get(i).copied().unwrap_or(0.0)));
            let mut s = format!("#{r:02x}{g:02x}{b:02x}");
            if l.len() == 4 && l[3] < 1.0 {
                s.push_str(&format!(
                    "{:02x}",
                    (l[3].clamp(0.0, 1.0) * 255.0).round() as u8
                ));
            }
            vec![s]
        }
        Editor::Text if field.ty == FieldType::Ease => vec![
            og_core::ease::Ease::from_bytes(bytes)
                .expect("validated ease")
                .to_text(),
        ],
        Editor::Text => vec![og_core::unpack_str(bytes).to_string()],
        Editor::Entity => {
            let bits = match bytes.len() {
                8.. => u64::from_le_bytes(word(bytes, 0)),
                _ => u64::from(u32::from_le_bytes(word(bytes, 0))),
            };
            vec![if bits == 0 {
                "none".into()
            } else {
                id_of(Entity::from_bits(bits))
            }]
        }
        Editor::ReadOnly => {
            if field.ty.is_list() {
                let n = og_core::list_elements(field.ty, bytes).count();
                vec![format!("{n} of {}", field.ty.name())]
            } else if field.ty == FieldType::Mat4 {
                vec!["matrix".into()]
            } else {
                vec![format!("handle {}", u64::from_le_bytes(word(bytes, 0)))]
            }
        }
    }
}

/// A field's value now as `set` writes it, exactly (the undo of an edit
/// writes it back): every lane, a rotation's quaternion, a colour linear.
pub fn current(editor: Editor, field: &FieldSchema, bytes: &[u8]) -> String {
    let join = |l: &[f64]| {
        l.iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    match editor {
        Editor::Text | Editor::Enum | Editor::Bool => {
            parts(editor, field, bytes, &|_| String::new())
                .into_iter()
                .next()
                .unwrap_or_default()
        }
        _ => join(&lanes(field.ty, bytes)),
    }
}

/// The command text (`set`'s right side) for a field whose shown parts
/// are `shown` after part `index` became `typed`. Refused with the reason
/// for a number that does not parse or a colour that is not `#rrggbb`.
pub fn command(
    editor: Editor,
    shown: &[String],
    index: usize,
    typed: &str,
) -> Result<String, String> {
    let typed = typed.trim();
    let mut p: Vec<String> = shown.to_vec();
    if let Some(slot) = p.get_mut(index) {
        *slot = typed.to_string();
    }
    let num = |t: &str| -> Result<f64, String> {
        t.trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("'{t}' is not a number"))
    };
    match editor {
        Editor::Number { int } => {
            let v = num(typed)?;
            if int && v.fract() != 0.0 {
                return Err(format!("'{typed}' is not a whole number"));
            }
            Ok(number_text(v))
        }
        Editor::Vector(_) => Ok(p
            .iter()
            .map(|t| num(t).map(number_text))
            .collect::<Result<Vec<_>, _>>()?
            .join(",")),
        Editor::Euler => {
            let d: Vec<f64> = p.iter().map(|t| num(t)).collect::<Result<_, _>>()?;
            let at = |i: usize| d.get(i).copied().unwrap_or(0.0);
            let q = quaternion([at(0), at(1), at(2)]);
            Ok(q.iter()
                .map(|v| format!("{}", *v as f32))
                .collect::<Vec<_>>()
                .join(","))
        }
        Editor::Bool => match typed {
            "true" | "false" => Ok(typed.to_string()),
            _ => Err(format!("'{typed}' is not true or false")),
        },
        Editor::Enum | Editor::Text => Ok(typed.to_string()),
        Editor::Color => {
            let hex = typed.strip_prefix('#').unwrap_or(typed);
            let [r, g, b, a] = og_core::color::parse_hex(hex)
                .filter(|_| hex.len() == 6 || hex.len() == 8)
                .ok_or_else(|| format!("'{typed}' is not a colour (#rrggbb)"))?;
            let mut lanes = vec![linear_of(r), linear_of(g), linear_of(b)];
            // A colour of four lanes keeps its alpha unless one was typed.
            let was = shown
                .first()
                .map(|s| s.trim_start_matches('#'))
                .filter(|s| s.len() == 8)
                .and_then(og_core::color::parse_hex)
                .map(|[_, _, _, a]| a);
            match (hex.len(), was) {
                (8, _) => lanes.push(f64::from(a) / 255.0),
                (_, Some(a)) => lanes.push(f64::from(a) / 255.0),
                _ => {}
            }
            Ok(lanes
                .iter()
                .map(|v| format!("{}", *v as f32))
                .collect::<Vec<_>>()
                .join(","))
        }
        Editor::Entity | Editor::ReadOnly => Err("this field is shown, not edited".into()),
    }
}

/// The parent of an entity (og.Parent's `entity`), if it has a live one.
pub fn parent_of(world: &World, e: Entity) -> Option<Entity> {
    let id = world.id_by_name("og.Parent").ok()?;
    let schema = world.schema(id).ok()?;
    let i = schema.field_index("entity")?;
    let bytes = world.get_bytes(e, id).ok()?;
    let range = Layout::of(schema).range(i);
    let bits = u64::from_le_bytes(bytes.get(range)?.try_into().ok()?);
    let p = Entity::from_bits(bits);
    (bits != 0 && p != e && world.is_alive(p)).then_some(p)
}

/// One row of the hierarchy.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub entity: Entity,
    pub depth: u32,
    /// Its probe id: its name, or `#<index>.<generation>`.
    pub label: String,
    pub named: bool,
    pub children: usize,
}

/// The hierarchy as rows, depth first, roots and siblings in entity
/// order. A collapsed entity's descendants are left out; with a filter,
/// the rows whose label holds it (any case) and their ancestors, all
/// expanded.
pub fn rows(
    entities: &[Entity],
    parent: &dyn Fn(Entity) -> Option<Entity>,
    label: &dyn Fn(Entity) -> (String, bool),
    collapsed: &BTreeSet<u64>,
    filter: &str,
) -> Vec<Row> {
    let mut sorted = entities.to_vec();
    sorted.sort_by_key(|e| (e.index(), e.generation()));
    let known: BTreeSet<u64> = sorted.iter().map(|e| e.to_bits()).collect();
    let mut children: BTreeMap<u64, Vec<Entity>> = BTreeMap::new();
    let mut roots = Vec::new();
    for &e in &sorted {
        match parent(e).filter(|p| known.contains(&p.to_bits())) {
            Some(p) => children.entry(p.to_bits()).or_default().push(e),
            None => roots.push(e),
        }
    }
    let filter = filter.trim().to_lowercase();
    // With a filter, the entities shown: the matches and their ancestors.
    let shown: Option<BTreeSet<u64>> = (!filter.is_empty()).then(|| {
        let mut s = BTreeSet::new();
        for &e in &sorted {
            if label(e).0.to_lowercase().contains(&filter) {
                let mut at = Some(e);
                let mut guard = 0;
                while let Some(x) = at {
                    if !s.insert(x.to_bits()) || guard > 256 {
                        break;
                    }
                    guard += 1;
                    at = parent(x).filter(|p| known.contains(&p.to_bits()));
                }
            }
        }
        s
    });
    let mut out = Vec::new();
    let mut stack: Vec<(Entity, u32)> = roots.into_iter().rev().map(|e| (e, 0)).collect();
    let mut seen = BTreeSet::new();
    while let Some((e, depth)) = stack.pop() {
        if !seen.insert(e.to_bits()) {
            continue;
        }
        if shown.as_ref().is_some_and(|s| !s.contains(&e.to_bits())) {
            continue;
        }
        let kids = children.get(&e.to_bits()).cloned().unwrap_or_default();
        let (text, named) = label(e);
        out.push(Row {
            entity: e,
            depth,
            label: text,
            named,
            children: kids.len(),
        });
        if shown.is_some() || !collapsed.contains(&e.to_bits()) {
            for k in kids.into_iter().rev() {
                stack.push((k, depth + 1));
            }
        }
    }
    out
}

/// One edit: a `set` target and its command text before and after.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub target: String,
    pub before: String,
    pub after: String,
}

/// The session's edits, undone and redone in order.
#[derive(Clone, Debug, Default)]
pub struct History {
    done: Vec<Edit>,
    undone: Vec<Edit>,
}

impl History {
    /// A new edit: what was undone is gone.
    pub fn push(&mut self, edit: Edit) {
        self.done.push(edit);
        self.undone.clear();
    }

    /// One more step of the last edit's drag: folded into it when it has
    /// the same target, so one undo takes the whole drag back.
    pub fn merge(&mut self, edit: Edit) {
        match self.done.last_mut() {
            Some(last) if last.target == edit.target => {
                last.after = edit.after;
                self.undone.clear();
            }
            _ => self.push(edit),
        }
    }

    /// The edit to undo: its target and the text to write back.
    pub fn undo(&mut self) -> Option<(String, String)> {
        let e = self.done.pop()?;
        let out = (e.target.clone(), e.before.clone());
        self.undone.push(e);
        Some(out)
    }

    /// The edit to redo.
    pub fn redo(&mut self) -> Option<(String, String)> {
        let e = self.undone.pop()?;
        let out = (e.target.clone(), e.after.clone());
        self.done.push(e);
        Some(out)
    }

    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
}

/// Whether the game's action map binds `code` (a `KeyboardEvent.code`):
/// then the inspector's toggle takes it only with Ctrl.
pub fn game_binds(actions: &og_input::actions::ActionMap, code: &str) -> bool {
    let Some(index) = og_input::KEY_CODES.iter().position(|c| *c == code) else {
        return false;
    };
    actions.infos().iter().any(|a| {
        a.bindings.iter().any(|b| {
            b.controls()
                .iter()
                .any(|(_, c)| matches!(c, og_input::Control::Key(k) if usize::from(*k) == index))
        })
    })
}
