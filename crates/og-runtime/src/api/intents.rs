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
    let record = wire::RecordedIntent {
        tick: tick as f64,
        name: name.to_string(),
        layout_hash: hash.to_string(),
        payload: payload.iter().map(|&b| u32::from(b)).collect(),
    };
    eng.state
        .borrow_mut()
        .recorded_intents
        .push(wire::ToJson::to_json(&record));
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
        let record: wire::RecordedIntent = wire::arg(Some(&intent), "intent")?;
        if record.tick < 0.0 || record.tick.fract() != 0.0 {
            return Err("recorded intent needs an integer tick".into());
        }
        let tick = record.tick as u64;
        if eng.world()?.time().tick != tick {
            return Err(format!(
                "recorded intent tick {tick} differs from world tick {}",
                eng.world()?.time().tick
            ));
        }
        let invalid = || "invalid recorded intent name, layoutHash or payload".to_string();
        let name = record.name;
        let layout_hash: u64 = record.layout_hash.parse().map_err(|_| invalid())?;
        let payload = record
            .payload
            .into_iter()
            .map(|b| u8::try_from(b).map_err(|_| invalid()))
            .collect::<Result<Vec<u8>, String>>()?;
        event_layout(&eng, &name)?;
        og_input::InputLayer::new()
            .inject_intent(eng.world()?, &name, layout_hash, &payload)
            .map_err(err)
    })())
}
