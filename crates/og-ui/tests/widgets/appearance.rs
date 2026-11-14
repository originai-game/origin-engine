use super::*;
use serde_json::json;

#[test]
fn anchors_sit_in_the_safe_area_and_touch_controls_keep_44px() {
    let vp = Viewport {
        width: 390.0,
        height: 844.0,
        dpr: 3.0,
        safe_area: Insets {
            left: 0.0,
            top: 47.0,
            right: 0.0,
            bottom: 34.0,
        },
        coarse_pointer: true,
        ..Viewport::default()
    };
    // The corner groups a HUD places at its insets, a gutter in.
    let mut d = mount_at(
        r#"<meta name="component" content="t">
<div id="Root">
  <column id="Score"><text id="Points">1200</text></column>
  <column id="Pad"><joystick id="Stick" action="move"/></column>
  <column id="Buttons"><dpad id="Dpad" action="look"/></column>
</div>
<style>
#Score { position: fixed; top: calc(env(-og-hud-inset-top) + 16px); right: calc(env(-og-hud-inset-right) + 16px); }
#Pad { position: fixed; left: calc(env(-og-hud-inset-left) + 16px); bottom: calc(env(-og-hud-inset-bottom) + 16px); }
#Buttons { position: fixed; right: calc(env(-og-hud-inset-right) + 16px); bottom: calc(env(-og-hud-inset-bottom) + 16px); }
</style>"#,
        &[],
        vp,
    );
    assert_eq!(d.ui.breakpoint(), Breakpoint::PhonePortrait);
    let gutter = 16.0;
    let score = d.get("Score");
    assert!(
        (score.y - (47.0 + gutter)).abs() < 0.5,
        "under the notch: {score:?}"
    );
    assert!((score.x + score.width - (390.0 - gutter)).abs() < 0.5);
    let pad = d.get("Pad");
    assert!(
        (pad.y + pad.height - (844.0 - 34.0 - gutter)).abs() < 0.5,
        "above the home indicator: {pad:?}"
    );
    for id in ["Stick", "Dpad/Up", "Dpad/Left"] {
        let e = d.get(id);
        assert!(
            e.width >= 44.0 && e.height >= 44.0,
            "{id} is {}×{}",
            e.width,
            e.height
        );
    }
    // Drag the stick to the right edge: its knob leans all the way right,
    // level (its vector (1, 0)).
    let c = center(&d, "Stick");
    let size = d.get("Stick").width;
    let r = size / 2.0;
    let knob = |d: &Doc| {
        let color = d.ui.theme().palette.knob.linear();
        d.ui.frame().items.iter().find_map(|i| match &i.cmd {
            DrawCmd::Shape { shape, .. } if shape.paint == og_draw::UiPaint::Solid(color) => {
                Some(shape.center)
            }
            _ => None,
        })
    };
    d.step(press(c));
    d.step(InputFrame {
        pointer: c + Vec2::new(r * 2.0, 0.0),
        pointer_inside: true,
        down: true,
        ..InputFrame::default()
    });
    // The knob's travel: the base's radius less its 2 px rim, a pixel and
    // half the knob's rim disc (0.4 of the size, 2 px wider).
    let travel = r - 3.0 - (size * 0.4 + 2.0) / 2.0;
    let k = knob(&d).expect("the stick is held");
    assert!(
        (k.x - (c.x + travel)).abs() < 0.5 && (k.y - c.y).abs() < 0.5,
        "{k:?} about {c:?}, travel {travel}"
    );
    d.step(release(c));
    idle(&mut d, 1);
    assert_eq!(knob(&d), None, "released");
}

/// An ultrawide HUD: on a 32:9 surface (5120×1440 CSS px) the corner groups
/// lay out in a centred 21:9 band (UiDesc.hudMaxAspect's default), at the
/// same size they have at 16:10, with the band's edges where the corners'
/// gutter starts; 0 puts them at the screen's edges again; a dialog keeps
/// the whole surface; a surface narrower than the band (16:10) is the
/// whole safe area.
///
/// Ways it could fail: the corners at the far edges of a 32:9 screen (the
/// player turns their head to read them); a band centred on the screen but
/// not on the safe area; the HUD stretched to the width (a bar 3× wider);
/// the band applied below 21:9, moving every 16:9 game's corners; a modal
/// squeezed into the band.
#[test]
fn hud_corners_sit_in_a_centred_band_on_an_ultrawide() {
    let mut d = Doc::new(
        r#"<meta name="component" content="t">
<div id="Root">
  <div id="Vitals"></div>
  <div id="Map"></div>
  <dialog id="Pause" modal bind:open={$pause} aria-label="Pause"><button id="Resume">Resume</button></dialog>
</div>
<style>
#Vitals, #Map { position: fixed; top: calc(env(-og-hud-inset-top) + 16px); width: 240px; height: 48px; }
#Vitals { left: calc(env(-og-hud-inset-left) + 16px); }
#Map { right: calc(env(-og-hud-inset-right) + 16px); }
</style>"#,
        &[("pause", json!(false))],
    );
    // One og-ui unit at every size: the band, not the unit, is the subject.
    let mut t = *d.ui.theme();
    t.unit_pin = 1.0;
    d.ui.set_theme(t);
    // 16:10: narrower than the band, so the corners are the safe area's.
    d.viewport = Viewport::new(1680.0, 1050.0);
    idle(&mut d, 2);
    let (vitals, map) = (d.get("Vitals"), d.get("Map"));
    let gutter = vitals.x;
    assert!(gutter > 0.0 && gutter < 64.0, "{vitals:?}");
    assert!(
        (1680.0 - (map.x + map.width) - gutter).abs() < 0.5,
        "{map:?}"
    );
    assert_eq!(d.ui.hud_area(), Rect::new(0.0, 0.0, 1680.0, 1050.0));

    // 32:9 with a safe-area inset on the left (a notch-like 100 px).
    d.viewport = Viewport {
        safe_area: Insets {
            left: 100.0,
            ..Insets::default()
        },
        ..Viewport::new(5120.0, 1440.0)
    };
    idle(&mut d, 2);
    let band = 1440.0 * 21.0 / 9.0;
    let left = 100.0 + (5020.0 - band) / 2.0;
    assert_eq!(d.ui.hud_area(), Rect::new(left, 0.0, band, 1440.0));
    let (wide_vitals, wide_map) = (d.get("Vitals"), d.get("Map"));
    assert!(
        (wide_vitals.x - (left + gutter)).abs() < 0.5,
        "top left of the band, not the screen: {wide_vitals:?}"
    );
    assert!(
        (left + band - (wide_map.x + wide_map.width) - gutter).abs() < 0.5,
        "top right of the band: {wide_map:?}"
    );
    assert_eq!(
        (wide_vitals.width, wide_vitals.height),
        (vitals.width, vitals.height),
        "the HUD never stretches"
    );

    // 0: the whole safe area again.
    d.ui.set_hud_max_aspect(0.0);
    idle(&mut d, 2);
    assert!((d.get("Vitals").x - (100.0 + gutter)).abs() < 0.5);
    assert!((5120.0 - (d.get("Map").x + 240.0) - gutter).abs() < 0.5);

    // A dialog's backdrop covers the whole surface, band or not.
    d.ui.set_hud_max_aspect(21.0 / 9.0);
    d.set("pause", json!(true));
    idle(&mut d, 2);
    // The modal's scrim stands in the top layer, under the dialog's id.
    let scrim = d.ui.element("Pause").cloned().expect("the scrim");
    assert_eq!((scrim.x, scrim.width), (0.0, 5120.0), "{scrim:?}");
}
