//! `src/theme.rs` is exactly what the generator writes from the vendored
//! platform tokens: a hand edit, a stale file or a token file changed without
//! regenerating all fail here. (`scripts/gen-theme.sh --check` additionally
//! compares the vendored copy with a platform checkout.)

#[path = "../scripts/theme_codegen.rs"]
mod theme_codegen;

use std::path::PathBuf;

#[test]
fn theme_rs_is_generated_from_the_vendored_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = std::fs::read_to_string(root.join("tokens/tokens.json")).expect("tokens.json");
    let committed = std::fs::read_to_string(root.join("src/theme.rs")).expect("theme.rs");
    let generated = theme_codegen::generate(&tokens).expect("tokens generate");
    if committed != generated {
        let line = committed
            .lines()
            .zip(generated.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(committed.lines().count().min(generated.lines().count()));
        panic!(
            "src/theme.rs differs from the generator's output at line {}:\n  committed: {:?}\n  generated: {:?}\nrun crates/og-ui/scripts/gen-theme.sh (never edit theme.rs by hand)",
            line + 1,
            committed.lines().nth(line),
            generated.lines().nth(line)
        );
    }
    // The theme the toolkit uses is the generated one.
    let dark = og_ui::Theme::dark();
    assert_eq!(dark.palette, og_ui::theme::DARK);
    assert_eq!(og_ui::Theme::light().palette, og_ui::theme::LIGHT);
    assert_eq!(dark.space, og_ui::theme::SPACE);
    assert_eq!(dark.type_ramp, og_ui::theme::TYPE_RAMP);
}

/// The edge light (`Theme::edge`, the tokens' `shadow.edge`): a card in the
/// dark token look is drawn as its fill with both layers over it, the fill
/// with the hairline over it below its top 0.5 px, and the plain fill
/// inside the hairline; in light a hairline only; a skinned card and a
/// translucent fill take none.
///
/// Ways it could fail: the layers drawn outside the card, in the wrong
/// order (the highlight under the hairline), on a translucent surface
/// (twice its alpha), or not at all.
#[test]
fn a_dark_card_draws_the_edge_light_inside_its_edge() {
    use og_ui::*;
    let quads = |theme: Theme, bg: Option<Srgba>| -> Vec<(Rect, [f32; 4])> {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_theme(theme);
        for _ in 0..2 {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.panel(
                "Card",
                PanelDesc {
                    width: Size::px(200.0),
                    height: Size::px(100.0),
                    background: bg,
                    ..PanelDesc::default()
                },
                |_| {},
            );
            ui.end();
        }
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Quad { rect, color, .. } => Some((*rect, *color)),
                _ => None,
            })
            .collect()
    };
    let dark = Theme::dark();
    let fill = dark.palette.surface;
    let q = quads(dark, None);
    assert_eq!(q.len(), 3, "fill and two layers: {q:?}");
    let [(outer, top), (lower, ring), (inner, plain)] = [q[0], q[1], q[2]];
    let edge = og_ui::theme::SHADOW_EDGE_DARK;
    let lin = |c: Srgba| c.linear();
    assert_eq!(
        top,
        lin(edge[1].color.over(edge[0].color.over(fill))),
        "top: both layers"
    );
    assert_eq!(
        ring,
        lin(edge[1].color.over(fill)),
        "the hairline below the top"
    );
    assert_eq!(plain, lin(fill), "the fill inside");
    assert_eq!((lower.y - outer.y, lower.height), (0.5, outer.height - 0.5));
    assert_eq!(
        (inner.x - outer.x, inner.width),
        (0.5, outer.width - 1.0),
        "the hairline is 0.5 px"
    );
    // Light: the hairline.
    let light = Theme::light();
    let q = quads(light, None);
    assert_eq!(q.len(), 2, "{q:?}");
    assert_eq!(
        q[0].1,
        lin(og_ui::theme::SHADOW_EDGE_LIGHT[0]
            .color
            .over(light.palette.surface))
    );
    // A translucent card: none.
    assert_eq!(quads(dark, Some(fill.fade(0.5))).len(), 1);
}
