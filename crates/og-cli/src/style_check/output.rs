use super::*;

// ── output ──────────────────────────────────────────────────────────────────

fn asset_json(a: &Asset) -> Value {
    json!({
        "asset": a.id,
        "role": a.role.name(),
        "optedOut": a.off,
        "measures": a.measures,
        "checks": a.checks.iter().map(|c| json!({
            "name": c.name, "value": c.value, "limit": c.limit, "ok": c.ok,
        })).collect::<Vec<_>>(),
        "notes": a.notes,
        "ok": a.ok(),
    })
}

fn frame_json(f: &Frame) -> Value {
    json!({
        "screen": f.screen,
        "viewport": f.viewport,
        "label": f.label(),
        "size": f.size,
        "tier": f.tier,
        "path": f.path.display().to_string(),
        "colours": f.colours,
        "colours95": f.colours95,
        "paletteHit": f.palette_hit,
        "edges": { "density": f.edges.density, "hard": f.edges.hard, "contrast": f.edges.contrast },
        "texelsPerPixel": f.grid,
        "camera": f.camera,
        "cameraChecks": f.camera_checks.iter().map(|c| json!({
            "name": c.name, "value": c.value, "limit": c.limit, "ok": c.ok,
        })).collect::<Vec<_>>(),
    })
}

/// What a check captured: the screens it ran and the scenes none reaches.
#[derive(Clone, Debug, Default)]
pub struct Coverage {
    pub screens: Vec<String>,
    pub viewports: Vec<String>,
    pub uncovered: Vec<String>,
}

/// The whole check as one JSON document.
pub fn to_json(m: &Measured, frames: &[Frame], frame_errors: &[String], cov: &Coverage) -> Value {
    let c = &m.contract;
    let slug = c.slug();
    json!({
        "contract": {
            "look": c.look,
            "extends": c.extends,
            "sets": c.sets,
            "palette": c.palette.as_ref().map(|(id, p)| json!({
                "id": id,
                "colours": p.len(),
                "hex": p.iter().map(|q| format!("#{:02x}{:02x}{:02x}", q[0], q[1], q[2])).collect::<Vec<_>>(),
            })),
            "art": c.art,
            "style": slug.as_ref().map(|s| json!({ "slug": s })),
        },
        "coverage": {
            "screens": cov.screens,
            "viewports": cov.viewports,
            "uncoveredScenes": cov.uncovered,
        },
        "assets": m.assets.iter().map(asset_json).collect::<Vec<_>>(),
        "optOuts": m.opt_outs.iter().map(|(a, w)| json!({ "asset": a, "why": w })).collect::<Vec<_>>(),
        "texelsPerPixel": m.grid,
        "densitySpread": m.density_spread,
        "frames": frames.iter().map(frame_json).collect::<Vec<_>>(),
        "frameErrors": frame_errors,
        "summary": {
            "assets": m.assets.len(),
            "offContract": m.off_contract().len(),
            "optOuts": m.opt_outs.len(),
        },
    })
}

/// One line per asset, its numbers and each check (✓ or ✗).
fn asset_line(a: &Asset) -> String {
    let mut parts = Vec::new();
    let m = &a.measures;
    if let Some(n) = m["faces"].as_u64() {
        parts.push(format!("faces {n}"));
    }
    if let Some(f) = m["flatShare"].as_f64() {
        parts.push(format!("flat {}", pct(f)));
    }
    if let Some(d) = m["density"]["median"].as_f64() {
        parts.push(format!("density {d:.1}/m"));
    }
    if let Some(n) = m["colours"].as_u64() {
        parts.push(format!("colours {n}"));
    }
    if let Some(h) = m["paletteHit"].as_f64() {
        parts.push(format!("palette {}", pct(h)));
    }
    if let Some(k) = m["grid"]["texelsPerPixel"].as_array() {
        let k: Vec<String> = k.iter().map(Value::to_string).collect();
        parts.push(format!("grid {}", k.join("x")));
    }
    if let Some(l) = m["line"].as_u64() {
        parts.push(format!("line {l}"));
    }
    let mut s = format!(
        "  {} {:<7} {:<44} {}",
        if a.ok() { "ok " } else { "off" },
        a.role.name(),
        a.id,
        parts.join("  ")
    );
    if a.off.is_some() {
        s.push_str("  (opted out)");
    }
    for c in &a.checks {
        s.push_str(&format!(
            "\n        {} {}: {} against {}",
            if c.ok { "✓" } else { "✗" },
            c.name,
            c.value,
            c.limit
        ));
    }
    for n in &a.notes {
        s.push_str(&format!("\n        · {n}"));
    }
    s
}

/// The check as lines for a reader.
pub fn report(m: &Measured, frames: &[Frame], frame_errors: &[String], cov: &Coverage) -> String {
    let c = &m.contract;
    let mut s = format!(
        "contract: {} sets {}\n",
        c.look,
        if c.sets.is_empty() {
            "no art resource: nothing to hold the assets to".into()
        } else {
            c.sets.join(", ")
        }
    );
    if let Some((id, p)) = &c.palette {
        s.push_str(&format!("palette: {id} ({} colours)\n", p.len()));
    }
    let off = m.off_contract().len();
    s.push_str(&format!(
        "assets: {} measured, {} off the contract{}\n",
        m.assets.len(),
        off,
        m.grid
            .map(|k| format!("; sprites at {k} texel(s) per logical pixel"))
            .unwrap_or_default()
    ));
    for a in &m.assets {
        s.push_str(&asset_line(a));
        s.push('\n');
    }
    if let Some(sp) = m.density_spread {
        s.push_str(&format!("texel density spread across models: x{sp:.2}\n"));
    }
    s.push_str(&format!("opt-outs: {}\n", m.opt_outs.len()));
    for (a, w) in &m.opt_outs {
        s.push_str(&format!("  {a}: {w}\n"));
    }
    if frames.is_empty() && frame_errors.is_empty() {
        s.push_str("frames: none: name the screens in project.ogt (screens { title \"<commands>\" play \"...\" results \"...\" }) or pass --title, --play, --results\n");
    } else {
        s.push_str("frames:\n");
    }
    for f in frames {
        s.push_str(&format!(
            "  {:<16} {}{}\n          colours {} (95% in {}){}  edges {} of pixels, {} hard, contrast {:.2}  grid {} px\n",
            f.label(),
            f.path.display(),
            f.tier.as_ref().map(|t| format!("  ({}×{}, {t})", f.size[0], f.size[1])).unwrap_or_else(|| format!("  ({}×{})", f.size[0], f.size[1])),
            f.colours,
            f.colours95,
            f.palette_hit.map(|h| format!("  palette {}", pct(h))).unwrap_or_default(),
            pct(f.edges.density),
            pct(f.edges.hard),
            f.edges.contrast,
            f.grid
        ));
        for c in &f.camera_checks {
            s.push_str(&format!(
                "          {} camera {}: {} against {}\n",
                if c.ok { "✓" } else { "✗" },
                c.name,
                c.value,
                c.limit
            ));
        }
    }
    for e in frame_errors {
        s.push_str(&format!("  failed: {e}\n"));
    }
    if !cov.uncovered.is_empty() {
        s.push_str(&format!(
            "coverage: no capture script for the scene(s) {}: add <name> \"<commands>\" to project.ogt's screens {{}} for each\n",
            cov.uncovered.join(", ")
        ));
    }
    s
}

/// The summary `og build --publish` prints as warnings: one line per asset
/// off the contract (at most `max`), and how many more.
pub fn warnings(m: &Measured, max: usize) -> Vec<String> {
    let off = m.off_contract();
    if off.is_empty() {
        return Vec::new();
    }
    let mut out = vec![format!(
        "style: {} of {} asset(s) off the look's art contract ({}); `og style check-project` says what",
        off.len(),
        m.assets.len(),
        m.contract.look
    )];
    for a in off.iter().take(max) {
        let failed: Vec<String> = a
            .checks
            .iter()
            .filter(|c| !c.ok)
            .map(|c| format!("{} {} against {}", c.name, c.value, c.limit))
            .collect();
        out.push(format!("style: {}: {}", a.id, failed.join("; ")));
    }
    if off.len() > max {
        out.push(format!("style: and {} more", off.len() - max));
    }
    out
}

/// `og build --publish`'s summary of the asset part of the check, after
/// its cook: warnings only, empty when the project has no look or every
/// asset meets the contract.
pub fn publish_warnings(root: &Path, cooked: &og_cook::Report) -> Vec<String> {
    match contract(root).and_then(|c| match c {
        Some(c) => measure(root, c, opt_outs_of(cooked)).map(Some),
        None => Ok(None),
    }) {
        Ok(Some(m)) => warnings(&m, 8),
        Ok(None) => Vec::new(),
        Err(e) => vec![format!("style: the assets could not be measured: {e}")],
    }
}
