//! The scene texts a district expands to, written through `.ogt`'s own
//! writer so every one reads back.

use glam::{DQuat, DVec3};
use og_core::ogt::{Block, Document, Entry, Number, Value};

use super::parse::PropCollider;

/// A number as the text keeps it: to a tenth of a millimetre, so the
/// text stays short and the same on every host.
fn num(x: f64) -> Value {
    let r = (x * 10_000.0).round() / 10_000.0;
    let r = if r == 0.0 { 0.0 } else { r };
    Value::Number(Number::from_f64(r).expect("a finite number"))
}

fn nums(key: &str, xs: &[f64]) -> Entry {
    xs.iter().fold(Entry::new(key), |e, x| e.with(num(*x)))
}

fn string(key: &str, s: &str) -> Entry {
    Entry::new(key).with(Value::String(s.to_string()))
}

fn word(key: &str, w: &str) -> Entry {
    Entry::new(key).with(Value::Word(w.to_string()))
}

fn block(key: &str, entries: Vec<Entry>) -> Entry {
    Entry::new(key).with_block(Block { entries })
}

fn transform(p: DVec3, q: DQuat) -> Entry {
    let mut e = vec![nums("position", &p.to_array())];
    if q != DQuat::IDENTITY {
        e.push(nums("quaternion", &q.to_array()));
    }
    block("Transform", e)
}

/// An entity that collides as `c`, standing on its origin.
fn collider_entity(name: &str, c: &PropCollider) -> Entry {
    let (lift, fields) = match c {
        PropCollider::Box(s) => (s[1] / 2.0, vec![word("shape", "box"), nums("size", s)]),
        PropCollider::Capsule(r, h) => (
            r + h / 2.0,
            vec![
                word("shape", "capsule"),
                nums("radius", &[*r]),
                nums("height", &[*h]),
            ],
        ),
    };
    let mut e = Entry::new("entity").with(Value::String(name.to_string()));
    e.block = Some(Block {
        entries: vec![
            transform(DVec3::new(0.0, lift, 0.0), DQuat::IDENTITY),
            block("Collider", fields),
        ],
    });
    e
}

/// An entity placing `model` at `p` turned by `q`, with its collider.
pub(super) fn placed(
    name: &str,
    model: &str,
    p: DVec3,
    q: DQuat,
    c: Option<&PropCollider>,
) -> Entry {
    let mut entries = vec![
        string("model", model),
        word("static", "true"),
        transform(p, q),
    ];
    if let Some(c) = c {
        // Under its prop's name: names are unique in the whole world.
        entries.push(collider_entity(&format!("{name}/collider"), c));
    }
    let mut e = Entry::new("entity").with(Value::String(name.to_string()));
    e.block = Some(Block { entries });
    e
}

/// An entity that only collides: a box of `c` centred at `p`.
pub(super) fn collider(name: &str, p: DVec3, q: DQuat, c: &PropCollider) -> Entry {
    let PropCollider::Box(s) = c else {
        unreachable!("a building collides as a box")
    };
    let mut e = Entry::new("entity").with(Value::String(name.to_string()));
    e.block = Some(Block {
        entries: vec![
            transform(p, q),
            block("Collider", vec![word("shape", "box"), nums("size", s)]),
        ],
    });
    e
}

/// A streaming cell placing `scene` over a `size`-metre square at `at`,
/// `height` tall, loaded within `load` metres.
pub(super) fn cell(name: &str, scene: &str, at: DVec3, size: f64, height: f64, load: f64) -> Entry {
    let mut e = Entry::new("entity").with(Value::String(name.to_string()));
    e.block = Some(Block {
        entries: vec![
            string("scene", scene),
            transform(at, DQuat::IDENTITY),
            block("Volume", vec![nums("size", &[size, 2.0 * height, size])]),
            block("StreamCell", vec![nums("load", &[load])]),
        ],
    });
    e
}

/// A scene's text: its name, its `hlod` block, its cell group (its name
/// and the assets it packs) and its entities.
pub(super) fn scene(
    id: &str,
    hlod: Option<&Entry>,
    group: Option<(&str, &[String])>,
    entities: &[Entry],
) -> Result<String, String> {
    let mut d = Document::new("scene", 1);
    d.body.push(string("name", id));
    if let Some(h) = hlod {
        let mut h = h.clone();
        h.line = 0;
        d.body.push(h);
    }
    if let Some((name, assets)) = group {
        let mut entries = vec![Entry::new("cell")];
        entries.extend(assets.iter().map(|a| string("asset", a)));
        d.body.push(
            Entry::new("stream")
                .with(Value::String(name.to_string()))
                .with_block(Block { entries }),
        );
    }
    d.body.entries.extend(entities.iter().cloned());
    d.to_text().map_err(|e| e.to_string())
}
