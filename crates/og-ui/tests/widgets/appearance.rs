use super::*;

#[test]
fn anchors_sit_in_the_safe_area_and_touch_controls_keep_44px() {
    let mut rig = Rig::new(JoystickResponse::default());
    rig.vp = Viewport {
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
    let b = &mut |ui: &mut Ui, s: &mut JoystickResponse| {
        ui.anchor(
            "Score",
            AnchorDesc {
                anchor: Anchor::TopRight,
                ..AnchorDesc::default()
            },
            |ui| {
                ui.label("Points", "1200", LabelDesc::default());
            },
        );
        ui.anchor(
            "Pad",
            AnchorDesc {
                anchor: Anchor::BottomLeft,
                ..AnchorDesc::default()
            },
            |ui| {
                *s = ui.joystick(
                    "Stick",
                    JoystickDesc {
                        size: 60.0,
                        ..JoystickDesc::default()
                    },
                );
            },
        );
        ui.anchor(
            "Buttons",
            AnchorDesc {
                anchor: Anchor::BottomRight,
                ..AnchorDesc::default()
            },
            |ui| {
                ui.action_button(
                    "Jump",
                    "Jump",
                    &ActionButtonDesc {
                        size: 20.0,
                        ..ActionButtonDesc::default()
                    },
                );
                ui.dpad(
                    "Dpad",
                    DpadDesc {
                        size: 60.0,
                        ..DpadDesc::default()
                    },
                );
            },
        );
    };
    rig.idle(1, b);
    assert_eq!(rig.ui.breakpoint(), Breakpoint::PhonePortrait);
    let gutter = rig.ui.sizes().gutter;
    let score = rig.ui.frame().element("Score").expect("score").clone();
    assert!(
        (score.y - (47.0 + gutter)).abs() < 0.5,
        "under the notch: {score:?}"
    );
    assert!((score.x + score.width - (390.0 - gutter)).abs() < 0.5);
    let pad = rig.ui.frame().element("Pad").expect("pad").clone();
    assert!(
        (pad.y + pad.height - (844.0 - 34.0 - gutter)).abs() < 0.5,
        "above the home indicator: {pad:?}"
    );
    for id in ["Stick", "Jump", "Dpad/Up", "Dpad/Left"] {
        let e = rig.ui.frame().element(id).expect(id);
        assert!(
            e.width >= 44.0 && e.height >= 44.0,
            "{id} is {}×{}",
            e.width,
            e.height
        );
    }
    // Drag the stick to the right edge.
    let c = rig.center("Stick");
    let r = rig.ui.frame().element("Stick").unwrap().width / 2.0;
    rig.step(press(c), b);
    rig.step(
        InputFrame {
            pointer: c + Vec2::new(r * 2.0, 0.0),
            pointer_inside: true,
            down: true,
            ..InputFrame::default()
        },
        b,
    );
    assert!(
        rig.state.active && (rig.state.x - 1.0).abs() < 1e-4 && rig.state.y.abs() < 1e-4,
        "{:?}",
        rig.state
    );
    rig.step(release(c), b);
    assert!(!rig.state.active && rig.state.x == 0.0);
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
    let mut rig = Rig::new(());
    // One og-ui unit at every size: the band, not the unit, is the subject.
    let mut t = *rig.ui.theme();
    t.unit_pin = 1.0;
    rig.ui.set_theme(t);
    let b = &mut |ui: &mut Ui, _: &mut ()| {
        for (id, anchor) in [("Vitals", Anchor::TopLeft), ("Map", Anchor::TopRight)] {
            ui.anchor(
                id,
                AnchorDesc {
                    anchor,
                    width: Size::Fixed(240.0),
                    height: Size::Fixed(48.0),
                    ..AnchorDesc::default()
                },
                |_| {},
            );
        }
    };
    let at = |rig: &Rig<()>, id: &str| rig.ui.frame().element(id).expect(id).clone();
    // 16:10: narrower than the band, so the corners are the safe area's.
    rig.vp = Viewport::new(1680.0, 1050.0);
    rig.idle(1, b);
    let (vitals, map) = (at(&rig, "Vitals"), at(&rig, "Map"));
    let gutter = vitals.x;
    assert!(gutter > 0.0 && gutter < 64.0, "{vitals:?}");
    assert!(
        (1680.0 - (map.x + map.width) - gutter).abs() < 0.5,
        "{map:?}"
    );
    assert_eq!(rig.ui.hud_area(), Rect::new(0.0, 0.0, 1680.0, 1050.0));

    // 32:9 with a safe-area inset on the left (a notch-like 100 px).
    rig.vp = Viewport {
        safe_area: Insets {
            left: 100.0,
            ..Insets::default()
        },
        ..Viewport::new(5120.0, 1440.0)
    };
    rig.idle(1, b);
    let band = 1440.0 * 21.0 / 9.0;
    let left = 100.0 + (5020.0 - band) / 2.0;
    assert_eq!(rig.ui.hud_area(), Rect::new(left, 0.0, band, 1440.0));
    let (wide_vitals, wide_map) = (at(&rig, "Vitals"), at(&rig, "Map"));
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
    rig.ui.set_hud_max_aspect(0.0);
    rig.idle(1, b);
    assert!((at(&rig, "Vitals").x - (100.0 + gutter)).abs() < 0.5);
    assert!((5120.0 - (at(&rig, "Map").x + 240.0) - gutter).abs() < 0.5);

    // A dialog's backdrop covers the whole surface, band or not.
    rig.ui.set_hud_max_aspect(21.0 / 9.0);
    let d = &mut |ui: &mut Ui, _: &mut ()| {
        ui.modal("Pause", true, &ModalDesc::default(), |_| {});
    };
    rig.idle(2, d);
    let scrim = at(&rig, "Pause");
    assert_eq!((scrim.x, scrim.width), (0.0, 5120.0), "{scrim:?}");
}
