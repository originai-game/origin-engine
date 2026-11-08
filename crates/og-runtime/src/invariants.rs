// The default invariants and og.assert run outside publish builds only.
#![cfg_attr(feature = "publish", allow(dead_code))]

//! The default invariants from reflection (IDL `assert`), checked after
//! every tick outside publish builds: no NaN or infinity in a reflected
//! float, a number inside its field's min..max where it declares one, no
//! Entity field naming a despawned entity, and, where the world names
//! og.WorldBounds, every root's Transform.position inside them.
//!
//! They only read, and they follow what changed: an archetype none of whose
//! columns of a component was written since the last check (the world's
//! write clock: a view taken for writing, a row set, spawned or moved) is
//! skipped, and of one that was, a component whose changes the world
//! tracks is read on the rows that changed since; Entity fields are read on
//! every row when the world's structure changed (a despawn dangles a
//! reference without touching it), and everything after a restore. Nothing
//! of this is world state: tracking stays the game's, and snapshots and
//! hashes are as without the checks. A resource is read again once written. A failure is an `always`
//! site of origin `default` in the probe's assertions, named for the rule
//! and placed at the record and field (`finite Health.hp`, `Health.hp`).

use crate::engine::Engine;
use og_core::FieldType;
use og_core::reflect::Scalar;
use og_ecs::{Entity, SchemaId, World};
use og_probe::assertions::{Origin, Seen};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// og.assert's bounds resource.
pub(crate) const BOUNDS: &str = "og.WorldBounds";

struct FieldCheck {
    index: usize,
    name: String,
    ty: FieldType,
    min: Option<f64>,
    max: Option<f64>,
    entity: bool,
}

struct RecordCheck {
    id: SchemaId,
    name: String,
    resource: bool,
    layout: og_core::Layout,
    fields: Vec<FieldCheck>,
}

#[derive(Default)]
struct Plan {
    /// (layouts, registered types) the plan was made for.
    made_for: (usize, usize),
    records: Vec<RecordCheck>,
    fields: u64,
    /// The world's change tick, write clock, structural and restore
    /// counts at the last check.
    since: Option<u64>,
    clock: Option<u64>,
    structural: u64,
    restores: u64,
}

thread_local! {
    static PLANS: RefCell<BTreeMap<u32, Plan>> = RefCell::default();
}

/// Whether a field of `schema` holds an entity: the IDL's column for an
/// engine record (a module's derived layout reads every u64 as one), the
/// layout's kind for a game's.
fn is_entity(schema: &str, field: &crate::codec::FieldLayout) -> bool {
    match og_api::host::RECORDS.iter().find(|d| d.schema == schema) {
        Some(d) => d
            .fields
            .iter()
            .any(|f| f.name == field.name && f.column == "Entity"),
        None => field.kind == crate::codec::Kind::Entity,
    }
}

fn make_plan(eng: &Engine, world: &World, plan: &mut Plan) {
    let st = eng.state.borrow();
    plan.records.clear();
    plan.fields = 0;
    for layout in st.layouts.values() {
        let resource = match layout.kind {
            crate::codec::RecordKind::Component => false,
            crate::codec::RecordKind::Resource => true,
            crate::codec::RecordKind::Event => continue,
        };
        let Ok(id) = world.id_by_name(&layout.schema) else {
            continue;
        };
        let Ok(schema) = world.schema(id) else {
            continue;
        };
        let mut fields = Vec::new();
        for (index, f) in schema.fields.iter().enumerate() {
            let Some(fl) = layout.fields.iter().find(|l| l.name == f.name) else {
                continue;
            };
            let float = matches!(f.ty.scalar(), Scalar::F32 | Scalar::F64);
            let numeric = float || matches!(f.ty.scalar(), Scalar::I32 | Scalar::U32);
            let ranged = numeric && f.ty != FieldType::Enum && (f.min.is_some() || f.max.is_some());
            let entity = f.ty.scalar() == Scalar::U64 && is_entity(&layout.schema, fl);
            if float || ranged || entity {
                fields.push(FieldCheck {
                    index,
                    name: f.name.clone(),
                    ty: f.ty,
                    min: if ranged { f.min } else { None },
                    max: if ranged { f.max } else { None },
                    entity,
                });
            }
        }
        if fields.is_empty() {
            continue;
        }
        plan.fields += fields.len() as u64;
        plan.records.push(RecordCheck {
            id,
            name: layout.schema.clone(),
            resource,
            layout: og_core::Layout::of(schema),
            fields,
        });
    }
}

/// What a value broke: the rule, the value, what was expected.
struct Broken {
    rule: &'static str,
    value: Value,
    expected: Value,
    message: String,
}

fn number(x: f64) -> Value {
    if x.is_nan() {
        json!("NaN")
    } else if x.is_infinite() {
        json!(if x > 0.0 { "Infinity" } else { "-Infinity" })
    } else {
        json!(x)
    }
}

/// The lanes of one value of `ty` as f64 (a list's elements up to its count).
fn lanes(ty: FieldType, bytes: &[u8], out: &mut Vec<f64>) {
    out.clear();
    let read = |s: Scalar, b: &[u8]| -> f64 {
        match s {
            Scalar::F32 => f64::from(f32::from_le_bytes(b[..4].try_into().unwrap())),
            Scalar::F64 => f64::from_le_bytes(b[..8].try_into().unwrap()),
            Scalar::I32 => f64::from(i32::from_le_bytes(b[..4].try_into().unwrap())),
            Scalar::U32 => f64::from(u32::from_le_bytes(b[..4].try_into().unwrap())),
            _ => 0.0,
        }
    };
    let s = ty.scalar();
    let w = match s {
        Scalar::F64 | Scalar::I64 | Scalar::U64 => 8,
        Scalar::Bool | Scalar::Utf8 => 1,
        _ => 4,
    };
    if ty.is_list() {
        let count = (read(s, bytes) as usize).min(ty.lanes() - 1);
        for i in 0..count {
            out.push(read(s, &bytes[(i + 1) * w..]));
        }
    } else {
        for i in 0..ty.lanes() {
            out.push(read(s, &bytes[i * w..]));
        }
    }
}

/// The u64 handles of one value (a list's elements up to its count).
fn handles(ty: FieldType, bytes: &[u8], out: &mut Vec<u64>) {
    out.clear();
    let at = |i: usize| u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap());
    if ty.is_list() {
        let count = (at(0) as usize).min(ty.lanes() - 1);
        out.extend((1..=count).map(at));
    } else {
        out.push(at(0));
    }
}

fn check(
    world: &World,
    f: &FieldCheck,
    bytes: &[u8],
    scratch: &mut (Vec<f64>, Vec<u64>),
) -> Option<Broken> {
    if f.entity {
        handles(f.ty, bytes, &mut scratch.1);
        let dead = scratch
            .1
            .iter()
            .copied()
            .find(|&b| b != 0 && !world.is_alive(Entity::from_bits(b)))?;
        let e = Entity::from_bits(dead);
        return Some(Broken {
            rule: "live entity",
            value: json!(format!("#{}.{}", e.index(), e.generation())),
            expected: json!("a live entity, or 0"),
            message: "names an entity that was despawned".into(),
        });
    }
    lanes(f.ty, bytes, &mut scratch.0);
    if let Some(&x) = scratch.0.iter().find(|x| !x.is_finite()) {
        return Some(Broken {
            rule: "finite",
            value: number(x),
            expected: json!("a finite number"),
            message: format!("{} is not finite", number(x)),
        });
    }
    let below = |x: f64| f.min.is_some_and(|m| x < m);
    let above = |x: f64| f.max.is_some_and(|m| x > m);
    let &x = scratch.0.iter().find(|&&x| below(x) || above(x))?;
    Some(Broken {
        rule: "in range",
        value: number(x),
        expected: json!({ "min": f.min, "max": f.max }),
        message: if below(x) {
            format!(
                "{x} is below the field's minimum {}",
                f.min.unwrap_or_default()
            )
        } else {
            format!(
                "{x} is above the field's maximum {}",
                f.max.unwrap_or_default()
            )
        },
    })
}

/// After every tick: every default invariant, on what changed.
pub(crate) fn after_tick(eng: &Engine, world: &World) {
    let t0 = crate::probe::now_ms();
    // The tick whose systems produced this state, the one og.assert sees as og.Time.tick
    // (og.Time has moved on to the next tick by now).
    let tick = og_ecs::last_tick();
    let layouts = eng.state.borrow().layouts.len();
    let mut failures: Vec<(String, String, Seen)> = Vec::new();
    let mut values = 0u64;
    let fields = PLANS.with(|plans| {
        let mut plans = plans.borrow_mut();
        let plan = plans.entry(eng.id).or_default();
        let made_for = (layouts, world.type_count());
        if plan.made_for != made_for {
            make_plan(eng, world, plan);
            plan.made_for = made_for;
            plan.since = None;
            plan.clock = None;
        }
        let now = world.change_tick();
        let clock = world.write_clock();
        // After a restore every row is the snapshot's: read everything again.
        let restored = world.restores() != plan.restores;
        let since = plan.since.filter(|&s| s <= now && !restored);
        let written_after = plan.clock.filter(|_| !restored);
        let structural = world.structural_changes() != plan.structural || since.is_none();
        let mut scratch = (Vec::new(), Vec::new());
        for r in &plan.records {
            if r.resource {
                if written_after
                    .is_some_and(|w| world.resource_written(r.id).is_none_or(|at| at <= w))
                {
                    continue;
                }
                let Ok(bytes) = world.resource_bytes(r.id) else {
                    continue;
                };
                for f in &r.fields {
                    values += 1;
                    if let Some(b) = check(world, f, &bytes[r.layout.range(f.index)], &mut scratch)
                    {
                        failures.push((
                            format!("{} {}.{}", b.rule, r.name, f.name),
                            format!("{}.{}", r.name, f.name),
                            Seen {
                                entity: None,
                                component: Some(r.name.clone()),
                                field: Some(f.name.clone()),
                                value: b.value,
                                expected: b.expected,
                                message: b.message,
                            },
                        ));
                    }
                }
                continue;
            }
            let has_refs = r.fields.iter().any(|f| f.entity);
            // Every row when a reference may have dangled; else the changed ones.
            let (rows_since, written) = if has_refs && structural {
                (None, None)
            } else {
                (since, written_after)
            };
            world.scan_component(r.id, rows_since, written, None, |entities, cols, rows| {
                let mut row = |i: usize| {
                    for f in &r.fields {
                        let size = f.ty.size();
                        let col = cols[f.index];
                        if col.len() < (i + 1) * size {
                            continue;
                        }
                        values += 1;
                        if let Some(b) =
                            check(world, f, &col[i * size..(i + 1) * size], &mut scratch)
                        {
                            failures.push((
                                format!("{} {}.{}", b.rule, r.name, f.name),
                                format!("{}.{}", r.name, f.name),
                                Seen {
                                    entity: Some(og_probe::Names.id_of(world, entities[i])),
                                    component: Some(r.name.clone()),
                                    field: Some(f.name.clone()),
                                    value: b.value,
                                    expected: b.expected,
                                    message: b.message,
                                },
                            ));
                        }
                    }
                };
                match rows {
                    Some(rows) => rows.iter().for_each(|&i| row(i as usize)),
                    None => (0..entities.len()).for_each(&mut row),
                }
            });
        }
        bounds(eng, world, &mut failures, &mut values);
        plan.since = Some(now);
        plan.clock = Some(clock);
        plan.structural = world.structural_changes();
        plan.restores = world.restores();
        plan.fields
    });
    let mut a = eng.assertions.borrow_mut();
    for (name, location, seen) in failures {
        a.fail(Origin::Default, &name, &location, tick, seen);
    }
    a.add_cost(fields, values, (crate::probe::now_ms() - t0).max(0.0));
}

/// og.WorldBounds: every root's Transform.position inside min..max.
fn bounds(
    eng: &Engine,
    world: &World,
    failures: &mut Vec<(String, String, Seen)>,
    values: &mut u64,
) {
    let Ok(id) = world.id_by_name(BOUNDS) else {
        return;
    };
    let Ok(bytes) = world.resource_bytes(id) else {
        return;
    };
    let Some(b) = eng
        .state
        .borrow()
        .layouts
        .get(BOUNDS)
        .map(|l| l.decode(&bytes))
    else {
        return;
    };
    if b["off"].as_bool() == Some(true) {
        return;
    }
    let corner = |v: &Value| ["x", "y", "z"].map(|k| v[k].as_f64().unwrap_or(0.0));
    let (lo, hi) = (corner(&b["min"]), corner(&b["max"]));
    let (Ok(transform), parent) = (
        world.id_by_name("og.Transform"),
        world.id_by_name("og.Parent").ok(),
    ) else {
        return;
    };
    let Ok(field) = world.field_by_id(transform, "position") else {
        return;
    };
    let index = field.field as usize;
    world.scan_component(transform, None, None, parent, |entities, cols, _| {
        let col = cols[index];
        for (i, &e) in entities.iter().enumerate() {
            let at = |k: usize| {
                f64::from(f32::from_le_bytes(
                    col[i * 12 + k * 4..i * 12 + k * 4 + 4].try_into().unwrap(),
                ))
            };
            let p = [at(0), at(1), at(2)];
            *values += 1;
            if (0..3).any(|k| p[k] < lo[k] || p[k] > hi[k]) {
                failures.push((
                    "inside og.WorldBounds".into(),
                    "og.Transform.position".into(),
                    Seen {
                        entity: Some(og_probe::Names.id_of(world, e)),
                        component: Some("og.Transform".into()),
                        field: Some("position".into()),
                        value: json!({ "x": p[0], "y": p[1], "z": p[2] }),
                        expected: json!({ "min": b["min"], "max": b["max"] }),
                        message: "the entity left og.WorldBounds".into(),
                    },
                ));
            }
        }
    });
}

/// An engine's plan, forgotten when it is destroyed.
pub(crate) fn forget(engine: u32) {
    PLANS.with(|p| p.borrow_mut().remove(&engine));
}

/// `og.assert.record`: a game's assertion call, recorded into the run's
/// report (never the world). Answers the report's epoch.
pub fn og_assert_record(world: u32, record: String) -> u32 {
    let Ok(eng) = crate::engine::engine(world) else {
        return 0;
    };
    let mut a = eng.assertions.borrow_mut();
    let Ok(r) = serde_json::from_str::<Value>(&record) else {
        return a.epoch();
    };
    if r.is_null() {
        return a.epoch();
    }
    let Some(kind) = r["kind"]
        .as_str()
        .and_then(og_probe::assertions::Kind::parse)
    else {
        return a.epoch();
    };
    let text = |k: &str| r[k].as_str().unwrap_or_default().to_owned();
    a.record(&og_probe::assertions::Call {
        kind,
        origin: Origin::Game,
        name: text("name"),
        location: text("location"),
        condition: r["condition"].as_bool().unwrap_or(true),
        key: text("key"),
        details: r.get("details").cloned().unwrap_or(Value::Null),
        tick: r["tick"].as_u64().unwrap_or(0),
        declared: r["declared"].as_bool().unwrap_or(false),
    });
    a.epoch()
}
