//! HTML's `<input type="color" bind:value={$c}>` (docs/ui.md §2): a swatch
//! that opens a picker against it (a saturation and value square over the
//! hue, a hue slider and the hex), writing `#rrggbb` back as HTML's value
//! is. Ported from the engine package's TS `colorPicker`.

use super::*;
use crate::paint::{ColorRef, ElementDraw, Gradient, Paint, Stop};

/// `#rgb`, `#rrggbb` or `#rrggbbaa` as 0–1 channels.
pub(crate) fn parse_hex(s: &str) -> Option<[f32; 3]> {
    let h = s.trim().strip_prefix('#')?;
    let digits: Vec<u8> = h
        .chars()
        .map(|c| c.to_digit(16).map(|d| d as u8))
        .collect::<Option<_>>()?;
    let [r, g, b] = match digits.len() {
        3 => [digits[0] * 17, digits[1] * 17, digits[2] * 17],
        6 | 8 => [
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
        ],
        _ => return None,
    };
    Some([r, g, b].map(|c| f32::from(c) / 255.0))
}

pub(crate) fn to_hex(c: [f32; 3]) -> String {
    let [r, g, b] = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Hue in degrees, saturation and value 0–1.
pub(crate) fn rgb_to_hsv([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let d = max - r.min(g).min(b);
    let h = if d <= 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h, if max > 0.0 { d / max } else { 0.0 }, max]
}

pub(crate) fn hsv_to_rgb([h, s, v]: [f32; 3]) -> [f32; 3] {
    let f = |n: f32| {
        let k = (n + h / 60.0).rem_euclid(6.0);
        v - v * s * k.min(4.0 - k).clamp(0.0, 1.0)
    };
    [f(5.0), f(3.0), f(1.0)]
}

fn srgba(c: [f32; 3], alpha: f32) -> crate::Srgba {
    crate::Srgba {
        rgb: c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
        alpha,
    }
}

impl Ui {
    /// Builds a colour input; answers its response (`changed` when it
    /// wrote a colour back).
    pub(super) fn color_input(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
    ) -> crate::ui::Response {
        let t = self.env.theme;
        let disabled = r.flag("disabled");
        let rgb = r.str("colorValue").and_then(parse_hex).unwrap_or([0.0; 3]);
        let swatch = self.leaf(
            "color",
            crate::Id::from(id),
            Element {
                width: Size::Fixed(self.by_text(44.0)),
                height: Size::Fixed(self.by_text(28.0)),
                background: Some(srgba(rgb, 1.0)),
                radius: t.radius.control,
                interactive: !disabled,
                ..Element::default()
            },
        );
        let key = self.nodes[swatch].key;
        self.nodes[swatch].a11y.role = Some("button");
        self.nodes[swatch].a11y.value = Some(to_hex(rgb));
        let pressed = self.respond(key);
        let open_key = crate::id::child_key(key, "~open");
        let mut open = self.has_anim(open_key) && self.anim(open_key).h != 0;
        if pressed.clicked && !disabled {
            open = !open;
        }
        // While the colour is grey it has no hue of its own: the last one
        // is kept (widget memory).
        let hue_key = crate::id::child_key(key, "~hue");
        let [mut h, mut s, mut v] = rgb_to_hsv(rgb);
        if s < 1e-4 || v < 1e-4 {
            h = f32::from_bits(self.anim(hue_key).h as u32);
        }
        let drag_key = crate::id::child_key(key, "~drag");
        let mut changed = false;
        let mut hex_out = None;
        let path = self.path_of(swatch).replace('/', "_");
        let content = Element {
            direction: Direction::Column,
            width: Size::Fixed(self.by_text(240.0)),
            gap: t.space.sm,
            padding: Insets::all(t.space.sm),
            frame: Some(crate::style_file::StylePart::Panel),
            ..Element::default()
        };
        let out = self.floating(&format!("{path}~picker"), open, swatch, content, |ui| {
            // The square: the hue, white across, black down; a ring at the colour.
            ui.open(
                "Square",
                Element {
                    width: Size::grow(),
                    height: Size::Fixed(ui.by_text(150.0)),
                    radius: t.radius.control,
                    interactive: true,
                    ..Element::default()
                },
            );
            let square = ui.parent();
            let square_key = ui.nodes[square].key;
            ui.nodes[square].a11y.role = Some("slider");
            ui.nodes[square].a11y.name = Some("Saturation and brightness".into());
            let lit = |c: [f32; 3], a: f32| ColorRef::Literal(srgba(c, a));
            let across = |from: [f32; 2], to: [f32; 2], a: ColorRef, z: ColorRef| {
                Paint::Gradient(Gradient {
                    from,
                    to,
                    stops: vec![
                        Stop {
                            offset: 0.0,
                            color: a,
                        },
                        Stop {
                            offset: 1.0,
                            color: z,
                        },
                    ],
                    ..Gradient::default()
                })
            };
            ui.set_draw(ElementDraw {
                fills: vec![
                    Paint::Solid(lit(hsv_to_rgb([h, 1.0, 1.0]), 1.0)),
                    across(
                        [0.0, 0.5],
                        [1.0, 0.5],
                        lit([1.0; 3], 1.0),
                        lit([1.0; 3], 0.0),
                    ),
                    across(
                        [0.5, 0.0],
                        [0.5, 1.0],
                        lit([0.0; 3], 0.0),
                        lit([0.0; 3], 1.0),
                    ),
                ],
                ..ElementDraw::default()
            });
            // Last tick's box: where a held press maps to a colour.
            let bounds = ui.last_bounds(square_key);
            let mut drag = ui.has_anim(drag_key) && ui.anim(drag_key).h != 0;
            if ui.interaction.input.pressed && ui.interaction.state.hot == square_key {
                drag = true;
            }
            if !ui.interaction.input.down {
                drag = false;
            }
            ui.set_anim(
                drag_key,
                crate::ui::Anim {
                    h: u64::from(drag),
                    ..Default::default()
                },
            );
            if drag && let Some(rect) = bounds {
                let p = ui.interaction.input.pointer;
                s = ((p.x - rect.x) / rect.width.max(1.0)).clamp(0.0, 1.0);
                v = 1.0 - ((p.y - rect.y) / rect.height.max(1.0)).clamp(0.0, 1.0);
                changed = true;
            }
            if let Some(rect) = bounds {
                let ring = ui.by_text(14.0);
                let i = ui.leaf(
                    "ring",
                    crate::Id::from("Ring"),
                    Element {
                        width: Size::Fixed(ring),
                        height: Size::Fixed(ring),
                        radius: ring / 2.0,
                        background: Some(srgba(hsv_to_rgb([h, s, v]), 1.0)),
                        ..Element::default()
                    },
                );
                ui.nodes[i].props.position = crate::element::Position::Absolute;
                ui.nodes[i].props.insets.start = Some(s * rect.width - ring / 2.0);
                ui.nodes[i].props.insets.top = Some((1.0 - v) * rect.height - ring / 2.0);
            }
            ui.close();
            let hue = ui.value_slider(
                "Hue",
                h,
                &crate::widgets::ValueSliderDesc {
                    min: 0.0,
                    max: 360.0,
                    label: "Hue".into(),
                    ..Default::default()
                },
            );
            if hue.changed {
                h = hue.value;
                changed = true;
            }
            let mut hex = to_hex(hsv_to_rgb([h, s, v]));
            let field = ui.text_field(
                "Hex",
                &mut hex,
                &crate::ui::FieldDesc {
                    max_chars: 7,
                    width: Some(Size::grow()),
                    ..Default::default()
                },
            );
            if field.changed
                && let Some(c) = parse_hex(&hex)
            {
                hex_out = Some(c);
            }
        });
        if out.dismissed {
            open = false;
        }
        self.set_anim(
            open_key,
            crate::ui::Anim {
                h: u64::from(open),
                ..Default::default()
            },
        );
        self.set_anim(
            hue_key,
            crate::ui::Anim {
                h: u64::from(h.to_bits()),
                ..Default::default()
            },
        );
        let next = match hex_out {
            Some(c) => Some(c),
            None if changed => Some(hsv_to_rgb([h, s, v])),
            None => None,
        };
        let mut resp = pressed;
        if let Some(c) = next
            && to_hex(c) != to_hex(rgb)
        {
            self.write_back(b, n, "colorValue", json!(to_hex(c)));
            resp.changed = true;
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_and_hsv_round_trip() {
        assert_eq!(parse_hex("#f80"), Some([1.0, 136.0 / 255.0, 0.0]));
        assert_eq!(parse_hex("#00ff0080"), Some([0.0, 1.0, 0.0]));
        assert_eq!(parse_hex("red"), None);
        for hex in ["#3366cc", "#ff0000", "#00ff7f", "#808080", "#000000"] {
            let c = parse_hex(hex).unwrap();
            assert_eq!(to_hex(hsv_to_rgb(rgb_to_hsv(c))), hex);
        }
        let [h, s, v] = rgb_to_hsv(parse_hex("#3366cc").unwrap());
        assert!((h - 220.0).abs() < 0.5 && (s - 0.75).abs() < 0.01 && (v - 0.8).abs() < 0.01);
    }
}
