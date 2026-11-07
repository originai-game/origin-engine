//! Reflected gameplay inputs. Presentation never supplies a simulation snapshot.
use super::*;

pub(crate) fn send_intent(
    eng: &Engine,
    ctx: Option<&og_ecs::SystemContext<'_>>,
    name: &str,
    payload: &[u8],
) -> Result<(), String> {
    let (layout, id) = if ctx.is_some() {
        running_record(eng, name, RecordKind::Event)?
    } else {
        event_layout(eng, name)?
    };
    let schema = layout.schema();
    let hash = schema.layout_hash();
    og_input::InputLayer::validate_intent(&schema, hash, payload).map_err(err)?;
    let tick = if let Some(ctx) = ctx {
        ctx.send_bytes(id, payload).map_err(err)?;
        ctx.time().tick + 1
    } else {
        let world = eng.world()?;
        og_input::InputLayer::new()
            .inject_intent(world, name, hash, payload)
            .map_err(err)?;
        world.time().tick
    };
    eng.state.borrow_mut().recorded_intents.push(json!({
        "tick": tick, "name": name, "layoutHash": hash.to_string(), "payload": payload,
    }));
    Ok(())
}

pub fn og_input_intent(world: u32, event: String, payload: String) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let frame = running(world);
        let (layout, _) = if frame.is_some() {
            running_record(&eng, &event, RecordKind::Event)?
        } else {
            event_layout(&eng, &event)?
        };
        let bytes = layout.encode(None, Some(&parse(Some(&payload))?))?;
        // SAFETY: the frame's context remains live throughout this host call.
        let ctx = frame.as_ref().map(|frame| unsafe { frame.ctx() });
        send_intent(&eng, ctx.as_deref(), &event, &bytes)
    })())
}

pub fn og_input_take_intents(world: u32) -> String {
    engine(world).map_or_else(
        |_| "[]".into(),
        |eng| {
            Value::Array(std::mem::take(&mut eng.state.borrow_mut().recorded_intents)).to_string()
        },
    )
}

pub fn og_input_replay_intent(world: u32, intent: String) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let mut record = parse(Some(&intent))?;
        let tick = record
            .get("tick")
            .and_then(Value::as_u64)
            .ok_or("recorded intent needs an integer tick")?;
        if eng.world()?.time().tick != tick {
            return Err(format!(
                "recorded intent tick {tick} differs from world tick {}",
                eng.world()?.time().tick
            ));
        }
        record
            .as_object_mut()
            .ok_or("recorded intent must be an object")?
            .insert("kind".into(), Value::String("intent".into()));
        let Some(og_input::InjectedInput::Intent {
            name,
            layout_hash,
            payload,
        }) = og_input::InjectedInput::from_json(&record)
        else {
            return Err("invalid recorded intent name, layoutHash or payload".into());
        };
        event_layout(&eng, &name)?;
        og_input::InputLayer::new()
            .inject_intent(eng.world()?, &name, layout_hash, &payload)
            .map_err(err)
    })())
}
