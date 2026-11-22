//! og.render.floatingText.

use super::*;

pub fn og_render_floating_text(world: u32, desc: Option<String>) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let d: crate::host::wire::FloatingTextDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        let text = d.text;
        let (family, named) = match d.font {
            None => {
                if !eng.render.borrow().ui_faces.any() {
                    return Err("desc.font is null and the UI has no body font: load one with og.load.font and pass it".into());
                }
                ("sans-serif".to_string(), None)
            }
            Some(h) => {
                let i = crate::world::assets::index_of(h as f64).ok_or_else(|| format!("{h} is not a font handle"))?;
                let named = eng.assets.borrow().fonts.get(i).map(|f| f.id.clone());
                if named.is_none() {
                    return Err(format!("font {h} is not alive"));
                }
                let family = crate::present::faces::family_of(&eng, &mut eng.render.borrow_mut().ui_faces, i)
                    .ok_or_else(|| format!("font {h} ships no file to lay text out from"))?;
                (family, named)
            }
        };
        let elapsed = with_world(&eng, |c| Ok(c.time_scale().elapsed), |w| Ok(w.time_scale().elapsed))?;
        // Its missing glyphs warned of now, on the CPU, renderer or not.
        let tick = with_world(&eng, |c| Ok(c.time().tick), |w| Ok(w.time().tick))?;
        let missing = {
            let mut rs = eng.render.borrow_mut();
            crate::present::faces::lacking(&mut rs, &text)
        };
        crate::present::faces::warn_missing(&eng, tick, named.as_deref(), &missing);
        let crit = d.crit;
        let scale = if crit { d.crit_scale.max(1.0) } else { 1.0 };
        let feel = feel_of(&eng)?;
        let mut sh = lock(&feel.shared);
        // The drift: a hash of the spawn count, so equal runs drift equally.
        let n = sh.spawned;
        sh.spawned += 1;
        let h = (n.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as u32;
        let unit = (h % 2001) as f32 / 1000.0 - 1.0;
        sh.floating.push(Floating {
            text,
            family,
            position: d.position,
            drift: unit * d.spread.max(0.0),
            start: elapsed,
            size: d.size.max(0.0) * scale,
            color: Vec3::new(d.color.r, d.color.g, d.color.b),
            outline: d.outline.clamp(0.0, 1.0),
            outline_color: Vec3::new(d.outline_color.r, d.outline_color.g, d.outline_color.b),
            rise: d.rise,
            lifetime: d.lifetime.max(0.05),
            fade: d.fade.clamp(0.0, 1.0),
            pop: d.pop.max(0.0) * if crit { 1.5 } else { 1.0 },
            layer: d.layer,
            facing: facing_named(Some(d.facing.as_str())),
        });
        Ok(())
    })
}
