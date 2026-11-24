//! Brain-owned CoC sources and observable capacity drops. Shader evaluation
//! never writes this state; the fixed camera tick owns every transition.

use super::*;
use crate::host::wire::ToJson;
use crate::present::lens::coc as wire_coc;
use og_render::{CocMix, CocSource, WeightedCoc};

pub(crate) const DROPPED: &str = "og.CameraCocDropped";

pub(super) fn settings(
    ctx: &SystemContext<'_>,
    l: &Layouts,
) -> Result<record::PostEffects, String> {
    let (Some(layout), Ok(id)) = (&l.post, ctx.id_by_name(POST)) else {
        return Ok(record::PostEffects::default());
    };
    match ctx.resource_bytes(id) {
        Ok(bytes) => layout.read(&bytes),
        Err(_) => Ok(record::PostEffects::default()),
    }
}

pub(super) fn primitive(
    camera: &record::Camera,
    post: &record::PostEffects,
    aspect: f32,
    focus: Option<(f32, f32)>,
) -> CocSource {
    let mut source = wire_coc::source(camera, post, aspect);
    if camera.lens_mode == crate::host::wire::CameraLensMode::Artistic
        && let Some((distance, range)) = focus
    {
        source.focus_distance = distance;
        source.focus_range = range;
    }
    source
}

pub(super) fn remember(brain: &mut Value, mix: CocMix) {
    brain["fromCocSources"] = wire_coc::write(mix).to_json();
}

pub(super) fn mix(
    from: CocMix,
    to: CocMix,
    lens: Lens,
    focus: Option<(f32, f32)>,
    weight: f32,
    (aspect, zoom): (f32, f32),
    excluded: &mut u32,
) -> (CocMix, Option<WeightedCoc>) {
    if weight <= 0.0 {
        return (from, None);
    }
    if weight >= 1.0 {
        return (to, None);
    }
    if !lens.resolved {
        let mut source = to.sources[0].source;
        if let Some(physical) = lens.physical {
            source = CocSource::physical(physical, aspect, zoom, source.max_blur);
        } else if let Some((distance, range)) = focus {
            source.focus_distance = distance;
            source.focus_range = range;
        }
        return (CocMix::one(source), None);
    }
    from.blend(to.sources[0].source, weight, excluded)
}

pub(super) fn report(
    ctx: &mut SystemContext<'_>,
    l: &Layouts,
    brain: &mut Value,
    dropped: WeightedCoc,
) -> og_core::Result<()> {
    let count = brain["cocDropCount"].as_u64().unwrap_or(0) as u32;
    let count = count.saturating_add(1);
    let frame = ctx.time().tick as u32;
    let source = wire_coc::entry(dropped).to_json();
    brain["cocDropCount"] = json!(count);
    brain["lastCocDrop"] = source.clone();
    brain["lastCocDropFrame"] = json!(frame);
    if let (Some(layout), Ok(id)) = (&l.coc_dropped, ctx.id_by_name(DROPPED)) {
        let event = json!({ "source": source, "frame": frame, "count": count });
        ctx.send_bytes(id, &layout.encode(None, Some(&event)).map_err(invalid)?)?;
    }
    Ok(())
}
