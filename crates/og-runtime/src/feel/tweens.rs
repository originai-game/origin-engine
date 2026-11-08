//! Tween calls and squash and flash presets.

use super::*;

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

pub(super) fn entity_of(v: &Value) -> Result<u64, String> {
    let e = v.as_f64().ok_or("desc.entity is the entity to tween")?;
    if e <= 0.0 || e.fract() != 0.0 {
        return Err(format!("{e} is not an entity"));
    }
    Ok(e as u64)
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

pub(super) fn ease_of(v: &Value, default: Ease) -> Result<Ease, String> {
    match v.as_str() {
        None => Ok(default),
        Some(n) => Ease::from_text(n).ok_or_else(|| {
            format!(
                "invalid ease '{n}': expected a GSAP name ({}), cubic-bezier(x1,y1,x2,y2) with four finite controls and x1,x2 in [0,1], or steps(n[,position]) with integer n >= 1 and position start/end/jump-start/jump-end/jump-none/jump-both (jump-none requires n > 1)",
                og_tween::EASE_NAMES.join(", ")
            )
        }),
    }
}

pub(super) fn floats(v: &Value, what: &str) -> Result<Vec<f32>, String> {
    match v {
        Value::Number(n) => Ok(vec![n.as_f64().unwrap_or(0.0) as f32]),
        Value::Array(a) => a
            .iter()
            .map(|x| {
                x.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| format!("{what} holds numbers"))
            })
            .collect(),
        _ => Err(format!("{what} is a number or a list of numbers")),
    }
}

/// A TweenDesc as a leaf (checked against the entity and the field).
fn leaf_of(eng: &Engine, d: &Value) -> Result<Leaf, String> {
    let entity = entity_of(&d["entity"])?;
    let name = if d["component"].is_null() {
        "og.Transform".to_string()
    } else {
        token_name(&d["component"])
            .ok_or("desc.component is a component token, such as og.Sprite")?
    };
    let (index, schema) = component_schema(eng, &name)?;
    if !has(eng, entity, index)? {
        return Err(format!("entity {entity} has no {name} (or is not alive)"));
    }
    let field = d["field"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("desc.field names the field, such as `position.y`")?;
    let target = Target::resolve(&schema, index, entity, field).map_err(err)?;
    let mut leaf = Leaf::new(target, floats(&d["to"], "desc.to")?)
        .duration(f(d, "duration", 0.3))
        .delay(f(d, "delay", 0.0))
        .ease(ease_of(&d["ease"], Ease::QuadOut)?)
        .relative(on(d, "relative", false))
        .loops(
            d["repeat"].as_i64().unwrap_or(0) as i32,
            on(d, "yoyo", false),
        );
    if !d["from"].is_null() {
        leaf = leaf.from(floats(&d["from"], "desc.from")?);
    }
    Ok(leaf)
}

fn start(eng: &Engine, node: Node, unscaled: bool, tag: u32) -> Result<Value, String> {
    let feel = feel_of(eng)?;
    let h = feel.tweens.start(node, unscaled, tag).map_err(err)?;
    Ok(json!(h as f64))
}

pub fn og_tween_to(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let leaf = leaf_of(&eng, &d)?;
        start(
            &eng,
            Node::Leaf(leaf),
            on(&d, "unscaled", false),
            d["tag"].as_u64().unwrap_or(0) as u32,
        )
    }))
}

pub fn og_tween_from(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let mut d = parse(desc.as_deref())?;
        if !d.is_object() {
            return Err("desc must contain entity, field and from".into());
        }
        d["to"] = d["from"].clone();
        let mut leaf = leaf_of(&eng, &d)?;
        let entity = Entity::from_bits(leaf.target.entity);
        let component = SchemaId::from_index(leaf.target.component);
        let bytes = with_world(
            &eng,
            |ctx| ctx.get_bytes(entity, component),
            |world| world.get_bytes(entity, component),
        )?;
        leaf.to = leaf.target.read(&bytes).ok_or("cannot read tween target")?;
        leaf.relative = false;
        start(
            &eng,
            Node::Leaf(leaf),
            on(&d, "unscaled", false),
            d["tag"].as_u64().unwrap_or(0) as u32,
        )
    }))
}

pub fn og_tween_from_to(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        if d["from"].is_null() {
            return Err("fromTo requires desc.from and desc.to".into());
        }
        let leaf = leaf_of(&eng, &d)?;
        start(
            &eng,
            Node::Leaf(leaf),
            on(&d, "unscaled", false),
            d["tag"].as_u64().unwrap_or(0) as u32,
        )
    }))
}

fn group(world: u32, kind: GroupKind, list: &str, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let list = parse(Some(list))?;
        let d = parse(desc.as_deref())?;
        let leaves = list
            .as_array()
            .ok_or("the tweens are a list of TweenDesc")?
            .iter()
            .enumerate()
            .map(|(i, t)| leaf_of(&eng, t).map_err(|e| format!("tween {i}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;
        let node = Node::group(kind, leaves, d["repeat"].as_i64().unwrap_or(0) as i32);
        start(
            &eng,
            node,
            on(&d, "unscaled", false),
            d["tag"].as_u64().unwrap_or(0) as u32,
        )
    }))
}

pub fn og_tween_sequence(world: u32, steps: String, desc: Option<String>) -> String {
    group(world, GroupKind::Sequence, &steps, desc)
}

pub fn og_tween_parallel(world: u32, tweens: String, desc: Option<String>) -> String {
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

pub fn og_tween_squash(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let entity = entity_of(&d["entity"])?;
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
        let impact = v3(&d["impact"], style);
        let rest = v3(&d["rest"], Vec3::ONE);
        let tick = 1.0 / eng.state.borrow().rate.max(1) as f32;
        let squash = og_tween::presets::Squash {
            impact: impact.to_array(),
            rest: rest.to_array(),
            duration: f(&d, "duration", 0.18),
            ease: ease_of(&d["ease"], Ease::BackOut)?,
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
    }))
}

pub fn og_tween_flash(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let entity = entity_of(&d["entity"])?;
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
        let c = rgb(&d["color"], Vec3::ONE);
        let tick = 1.0 / eng.state.borrow().rate.max(1) as f32;
        let flash = og_tween::presets::Flash {
            color: c.to_array(),
            amount: f(&d, "amount", 1.0),
            duration: f(&d, "duration", 0.12),
            ease: ease_of(&d["ease"], Ease::QuadIn)?,
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
    }))
}
