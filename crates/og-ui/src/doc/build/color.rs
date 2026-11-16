//! HTML's `<input type="color" bind:value={$c}>` (docs/ui.md §2): a swatch
//! that opens a picker against it (a saturation and value square over the
//! hue, a hue slider and the hex), writing `#rrggbb` back as HTML's value
//! is. Ported from the engine package's TS `colorPicker`.

use super::*;
use crate::paint::{ColorRef, ElementDraw, Gradient, Paint, Stop};
use ::color::{Hwb, OpaqueColor, Srgb};

/// `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa` as 0–1 channels (the alpha
/// dropped).
pub(crate) fn parse_hex(s: &str) -> Option<[f32; 3]> {
    let c = crate::style_file::parse_color(s.trim())?;
    Some(c.rgb.map(|c| f32::from(c) / 255.0))
}

pub(crate) fn to_hex(c: [f32; 3]) -> String {
    let [r, g, b] = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    og_core::color::hex_string([r, g, b, 255])
}

/// Hue in degrees, saturation and value 0–1: HWB's (`color`'s) restated,
/// value the whiteness's complement of blackness.
pub(crate) fn rgb_to_hsv(rgb: [f32; 3]) -> [f32; 3] {
    let [h, w, b] = OpaqueColor::<Srgb>::new(rgb).convert::<Hwb>().components;
    let v = 1.0 - b / 100.0;
    [h, if v > 0.0 { 1.0 - w / 100.0 / v } else { 0.0 }, v]
}

pub(crate) fn hsv_to_rgb([h, s, v]: [f32; 3]) -> [f32; 3] {
    let hwb = [h, (1.0 - s) * v * 100.0, (1.0 - v) * 100.0];
    OpaqueColor::<Hwb>::new(hwb).convert::<Srgb>().components
}

fn srgba(c: [f32; 3], alpha: f32) -> crate::Srgba {
    crate::Srgba {
        rgb: c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
        alpha,
        p3: None,
    }
}

impl Ui {
    /// Builds a colour input; answers its response (`changed` when it
    /// wrote a colour back).
    pub(crate) fn color_input(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
    ) -> crate::ui::Response {
        let t = *self.env.theme();
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
        let key = self.tree.nodes()[swatch].key;
        self.tree.semantics_mut(swatch).role = Some("button");
        self.tree.semantics_mut(swatch).value = Some(to_hex(rgb));
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
        let path = self.tree.path_of(swatch).replace('/', "_");
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
            let square = ui.tree.parent();
            let square_key = ui.tree.nodes()[square].key;
            ui.tree.semantics_mut(square).role = Some("slider");
            ui.tree.semantics_mut(square).name = Some("Saturation and brightness".into());
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
            if ui.interaction.input().pressed && ui.interaction.state().hot == square_key {
                drag = true;
            }
            if !ui.interaction.input().down {
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
                let p = ui.interaction.input().pointer;
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
                ui.tree.props_mut(i).position = crate::element::Position::Absolute;
                ui.tree.props_mut(i).insets.start = Some(s * rect.width - ring / 2.0);
                ui.tree.props_mut(i).insets.top = Some((1.0 - v) * rect.height - ring / 2.0);
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
            self.docs.write_back(b, n, "colorValue", json!(to_hex(c)));
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
        assert_eq!(parse_hex("#f808"), Some([1.0, 136.0 / 255.0, 0.0]));
        assert_eq!(parse_hex("f80"), None);
        assert_eq!(parse_hex("red"), None);
        for hex in ["#3366cc", "#ff0000", "#00ff7f", "#808080", "#000000"] {
            let c = parse_hex(hex).unwrap();
            assert_eq!(to_hex(hsv_to_rgb(rgb_to_hsv(c))), hex);
        }
        let [h, s, v] = rgb_to_hsv(parse_hex("#3366cc").unwrap());
        assert!((h - 220.0).abs() < 0.5 && (s - 0.75).abs() < 0.01 && (v - 0.8).abs() < 0.01);
    }
}
