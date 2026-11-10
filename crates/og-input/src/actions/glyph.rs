use super::*;

/// The glyphs of the first of `bindings` with a control on `device`'s
/// class, each with its control's name (a composite's parts in prompt
/// order). og-ui's UI actions draw theirs with it too.
pub fn binding_glyphs(bindings: &[Binding], device: Device) -> Vec<(Glyph, String)> {
    let class = device.class();
    for b in bindings {
        let controls = b.controls();
        if controls.is_empty() || controls.iter().any(|(_, c)| c.class() != class) {
            continue;
        }
        let mut ordered = controls;
        let rank = |p: Option<&str>| {
            ["up", "left", "down", "right", "negative", "positive"]
                .iter()
                .position(|x| Some(*x) == p)
                .unwrap_or(0)
        };
        ordered.sort_by_key(|(p, _)| rank(*p));
        return ordered
            .into_iter()
            .map(|(_, c)| (c.glyph(device), c.name()))
            .collect();
    }
    Vec::new()
}

/// One glyph for a list of a binding's glyphs: a single control's own, a
/// composite's texts together with a set's icon where one names them
/// (`key.wasd`, `key.arrows`, `xbox.dpad`), else its first part's.
pub fn combined_glyph(parts: &[(Glyph, String)], device: Device) -> Option<Glyph> {
    match parts {
        [] => None,
        [(g, _)] => Some(g.clone()),
        _ => {
            let names: Vec<&str> = parts.iter().map(|(_, n)| n.as_str()).collect();
            let texts: Vec<&str> = parts.iter().map(|(g, _)| g.text.as_str()).collect();
            let icon = match names.as_slice() {
                // The set's WASD icon only where the layout prints W A S D
                // there: AZERTY's Z Q S D draw as their first key's.
                ["KeyW", "KeyA", "KeyS", "KeyD"] if texts == ["W", "A", "S", "D"] => {
                    "key.wasd".to_string()
                }
                ["ArrowUp", "ArrowLeft", "ArrowDown", "ArrowRight"] => "key.arrows".to_string(),
                ["pad.up", "pad.left", "pad.down", "pad.right"] => {
                    Control::Axis2(Axis2::Dpad).glyph(device).icon
                }
                _ => parts[0].0.icon.clone(),
            };
            Some(Glyph {
                icon,
                text: parts
                    .iter()
                    .map(|(g, _)| g.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            })
        }
    }
}
