//! Tween calls and squash and flash presets.

use super::*;
use crate::wire::{FlashDesc, OneOf, SquashDesc, TweenDesc, TweenFromDesc, TweenGroupDesc};

// ── tweens ─────────────────────────────────────────────────────────────────

/// A token or schema name of a component.
pub(super) fn token_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("name").and_then(|n| n.as_str()).map(str::to_string),
        _ => None,
    }
}

/// The schema id index and schema of a component, by name.
pub(super) fn component_schema(eng: &Engine, name: &str) -> Result<(u32, og_core::Schema), String> {
    let st = eng.state.borrow();
    let layout = st
        .layouts
        .get(name)
        .filter(|l| l.kind == super::codec::RecordKind::Component)
        .ok_or_else(|| format!("'{name}' is not a registered component"))?;
    let index = st
        .names_by_id
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| format!("'{name}' is not registered in this world"))?;
    Ok((index as u32, layout.schema()))
}

/// A desc's `entity`: 0 (absent) is no entity.
fn entity_of(e: u64) -> Result<u64, String> {
    if e == 0 {
        return Err("desc.entity is the entity to tween".into());
    }
    Ok(e)
}

/// Whether `e` is alive and has component `index` (inside a system too:
/// archetype membership needs no declaration).
pub(super) fn has(eng: &Engine, e: u64, index: u32) -> Result<bool, String> {
    let id = SchemaId::from_index(index);
    let ent = Entity::from_bits(e);
    let types = with_world(
        eng,
        |c| {
            if c.is_alive(ent) {
                c.components_of(ent)
            } else {
                Ok(Vec::new())
            }
        },
        |w| {
            if w.is_alive(ent) {
                w.components_of(ent)
            } else {
                Ok(Vec::new())
            }
        },
    )?;
    Ok(types.contains(&id))
}

/// An Ease's text; empty (absent) is `default`.
pub(super) fn ease_of(n: &str, default: Ease) -> Result<Ease, String> {
    if n.is_empty() {
        return Ok(default);
    }
    Ease::from_text(n).ok_or_else(|| {
        format!(
            "invalid ease '{n}': expected a GSAP name ({}), cubic-bezier(x1,y1,x2,y2) with four finite controls and x1,x2 in [0,1], or steps(n[,position]) with integer n >= 1 and position start/end/jump-start/jump-end/jump-none/jump-both (jump-none requires n > 1)",
            og_tween::EASE_NAMES.join(", ")
        )
    })
}

/// A lane value (`to`, `from`): one number for every lane, or one per lane.
pub(super) fn lanes(v: Option<OneOf<f32, Vec<f32>>>) -> Option<Vec<f32>> {
    v.map(|v| match v {
        OneOf::A(n) => vec![n],
        OneOf::B(list) => list,
    })
}

/// A required lane value `what` that was left out.
pub(super) fn required(what: &str) -> String {
    format!("{what}: expected a number or a list of numbers, got nothing")
}

/// A TweenDesc as a leaf (checked against the entity and the field).
fn leaf_of(eng: &Engine, d: TweenDesc) -> Result<Leaf, String> {
    let entity = entity_of(d.entity)?;
    let to = lanes(d.to).ok_or_else(|| required("desc.to"))?;
    let name = match &d.component {
        None => "og.Transform".to_string(),
        Some(c) => token_name(c).ok_or("desc.component is a component token, such as og.Sprite")?,
    };
    let (index, schema) = component_schema(eng, &name)?;
    if !has(eng, entity, index)? {
        return Err(format!("entity {entity} has no {name} (or is not alive)"));
    }
    if d.field.is_empty() {
        return Err("desc.field names the field, such as `position.y`".into());
    }
    let target = Target::resolve(&schema, index, entity, &d.field).map_err(err)?;
    let mut leaf = Leaf::new(target, to)
        .duration(d.duration)
        .delay(d.delay)
        .ease(ease_of(&d.ease, Ease::QuadOut)?)
        .relative(d.relative)
        .loops(d.repeat, d.yoyo);
    if let Some(from) = lanes(d.from) {
        leaf = leaf.from(from);
    }
    Ok(leaf)
}

fn start(eng: &Engine, node: Node, unscaled: bool, tag: u32) -> Result<Value, String> {
    let feel = feel_of(eng)?;
    let h = feel.tweens.start(node, unscaled, tag).map_err(err)?;
    Ok(json!(h as f64))
}

pub fn og_tween_to(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let d: TweenDesc = crate::wire::arg(desc.as_deref(), "desc")?;
        let (unscaled, tag) = (d.unscaled, d.tag);
        let leaf = leaf_of(&eng, d)?;
        start(&eng, Node::Leaf(leaf), unscaled, tag)
    })
}

pub fn og_tween_from(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let f: TweenFromDesc = crate::wire::arg(desc.as_deref(), "desc")?;
        if f.from.is_none() {
            return Err(required("desc.from"));
        }
        let d = TweenDesc {
            entity: f.entity,
            component: f.component,
            field: f.field,
            duration: f.duration,
            delay: f.delay,
            ease: f.ease,
            repeat: f.repeat,
            yoyo: f.yoyo,
            unscaled: f.unscaled,
            tag: f.tag,
            to: f.from.clone(),
            from: f.from,
            ..TweenDesc::default()
        };
        let (unscaled, tag) = (d.unscaled, d.tag);
        let mut leaf = leaf_of(&eng, d)?;
        let entity = Entity::from_bits(leaf.target.entity);
        let component = SchemaId::from_index(leaf.target.component);
        let bytes = with_world(
            &eng,
            |ctx| ctx.get_bytes(entity, component),
            |world| world.get_bytes(entity, component),
        )?;
        leaf.to = leaf.target.read(&bytes).ok_or("cannot read tween target")?;
        leaf.relative = false;
        start(&eng, Node::Leaf(leaf), unscaled, tag)
    })
}

pub fn og_tween_from_to(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let d: TweenDesc = crate::wire::arg(desc.as_deref(), "desc")?;
        if d.from.is_none() {
            return Err(required("desc.from"));
        }
        let (unscaled, tag) = (d.unscaled, d.tag);
        let leaf = leaf_of(&eng, d)?;
        start(&eng, Node::Leaf(leaf), unscaled, tag)
    })
}

fn group(
    world: u32,
    kind: GroupKind,
    list: &str,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let list: Vec<TweenDesc> = crate::wire::arg(Some(list), "tweens")?;
        let d: TweenGroupDesc = crate::wire::arg(desc.as_deref(), "desc")?;
        let leaves = list
            .into_iter()
            .enumerate()
            .map(|(i, t)| leaf_of(&eng, t).map_err(|e| format!("tween {i}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;
        let node = Node::group(kind, leaves, d.repeat);
        start(&eng, node, d.unscaled, d.tag)
    })
}

pub fn og_tween_sequence(
    world: u32,
    steps: String,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    group(world, GroupKind::Sequence, &steps, desc)
}

pub fn og_tween_parallel(
    world: u32,
    tweens: String,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    group(world, GroupKind::Parallel, &tweens, desc)
}

pub fn og_tween_cancel(world: u32, tween: f64) -> bool {
    engine(world)
        .and_then(|e| feel_of(&e))
        .is_ok_and(|f| f.tweens.cancel(tween as u64))
}

pub fn og_tween_cancel_entity(world: u32, entity: f64) -> u32 {
    engine(world)
        .and_then(|e| feel_of(&e))
        .map_or(0, |f| f.tweens.cancel_entity(entity as u64))
}

pub fn og_tween_active(world: u32, tween: f64) -> bool {
    engine(world)
        .and_then(|e| feel_of(&e))
        .is_ok_and(|f| f.tweens.is_active(tween as u64))
}

/// A preset shows its impact whole on the tick it is started in: every
/// tween in it waits one tick (it then holds its start value), so the
/// recovery begins on the next.
fn hold_first_tick(node: Node, tick: f32) -> Node {
    match node {
        Node::Leaf(l) => {
            let d = l.delay;
            Node::Leaf(l.delay(d + tick))
        }
        Node::Group(mut g) => {
            for c in &mut g.children {
                c.delay += tick;
            }
            Node::Group(g)
        }
        // Presets are leaves and groups.
        timeline @ Node::Timeline(_) => timeline,
    }
}

pub fn og_tween_squash(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let d: SquashDesc = crate::wire::arg(desc.as_deref(), "desc")?;
        let entity = entity_of(d.entity)?;
        let (index, schema) = component_schema(&eng, "og.Transform")?;
        if !has(&eng, entity, index)? {
            return Err(format!(
                "entity {entity} has no og.Transform (or is not alive)"
            ));
        }
        let target = Target::resolve(&schema, index, entity, "scale").map_err(err)?;
        // The components left out take the art contract's squash
        // (ArtMotion.squash; the preset's own without it).
        let style = crate::art::squash_impact(crate::art::squash_amount(&eng));
        // `impact` is open (a partial vector), read by hand.
        let impact = v3(&d.impact, style);
        let tick = 1.0 / eng.state.borrow().rate.max(1) as f32;
        let squash = og_tween::presets::Squash {
            impact: impact.to_array(),
            rest: d.rest.to_array(),
            duration: d.duration,
            ease: ease_of(&d.ease, Ease::BackOut)?,
        };
        let feel = feel_of(&eng)?;
        if let Some(old) = lock(&feel.shared).squash.remove(&entity) {
            feel.tweens.cancel(old);
        }
        let h = feel
            .tweens
            .start(
                hold_first_tick(og_tween::presets::squash(target, &squash), tick),
                false,
                0,
            )
            .map_err(err)?;
        lock(&feel.shared).squash.insert(entity, h);
        Ok(json!(h as f64))
    })
}

pub fn og_tween_flash(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let d: FlashDesc = crate::wire::arg(desc.as_deref(), "desc")?;
        let entity = entity_of(d.entity)?;
        let mut found = None;
        // A glTF instance flashes alone: og.Model's flash rides in its
        // instances' records (kestrel-stair #11).
        for name in ["og.Sprite", "og.Mesh", "og.Model"] {
            let (index, schema) = component_schema(&eng, name)?;
            if has(&eng, entity, index)? {
                found = Some((index, schema));
                break;
            }
        }
        let (index, schema) = found.ok_or_else(|| {
            format!("entity {entity} has none of og.Sprite, og.Mesh and og.Model (or is not alive)")
        })?;
        let amount = Target::resolve(&schema, index, entity, "flash").map_err(err)?;
        let color = Target::resolve(&schema, index, entity, "flashColor").map_err(err)?;
        let tick = 1.0 / eng.state.borrow().rate.max(1) as f32;
        let flash = og_tween::presets::Flash {
            color: [d.color.r, d.color.g, d.color.b],
            amount: d.amount,
            duration: d.duration,
            ease: ease_of(&d.ease, Ease::QuadIn)?,
        };
        let feel = feel_of(&eng)?;
        if let Some(old) = lock(&feel.shared).flash.remove(&entity) {
            feel.tweens.cancel(old);
        }
        let h = feel
            .tweens
            .start(
                hold_first_tick(og_tween::presets::flash(amount, Some(color), &flash), tick),
                false,
                0,
            )
            .map_err(err)?;
        lock(&feel.shared).flash.insert(entity, h);
        Ok(json!(h as f64))
    })
}
