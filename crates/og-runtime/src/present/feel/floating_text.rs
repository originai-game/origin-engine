//! og.render.floatingText.

use super::*;

pub fn og_render_floating_text(world: u32, desc: Option<String>) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let d: crate::host::wire::FloatingTextDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
        let text = d.text;
        let font = match d.font {
            None => {
                let has_ui = eng.render.borrow().ui.as_ref().is_some_and(|u| u.lock().fonts().body.handle.is_some());
                if !has_ui {
                    return Err("desc.font is null and the UI has no body font: load one with og.load.font and pass it".into());
                }
                u32::MAX
            }
            Some(h) => {
                let i = crate::world::assets::index_of(h as f64).ok_or_else(|| format!("{h} is not a font handle"))?;
                if i >= eng.render.borrow().fonts.len() {
                    return Err(format!("font {h} is not alive"));
                }
                i as u32
            }
        };
        let elapsed = with_world(&eng, |c| Ok(c.time_scale().elapsed), |w| Ok(w.time_scale().elapsed))?;
        // Its missing glyphs warned of now, on the CPU, renderer or not.
        let tick = with_world(&eng, |c| Ok(c.time().tick), |w| Ok(w.time().tick))?;
        let missing = {
            let mut rs = eng.render.borrow_mut();
            if font == u32::MAX {
                crate::present::glyphs::ui_lacking(&mut rs, &text)
            } else {
                crate::present::glyphs::lacking(&mut rs, font as usize, &text).into_iter().collect()
            }
        };
        crate::present::glyphs::warn_missing(&eng, tick, missing);
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
            font,
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
