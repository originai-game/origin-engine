//! A development host's default style (`og_ui::set_base_css`): a UI and a
//! pack read after it wear it, CSS that does not read changes nothing, and
//! `None` puts the compiled default back. Its own binary: the default style
//! is the process's, so no other test may read it meanwhile.

use og_ui::{DEFAULT_CSS, Ui, set_base_css, style_pack::StylePack};

fn ink(ui: &Ui) -> [u8; 3] {
    ui.theme().palette.ink.rgb
}

fn packed() -> Ui {
    let pack = StylePack::parse(
        "styles/plain",
        vec![(
            "styles/plain/base.css".into(),
            ":root { --gap-test: 1px; }".into(),
        )],
    )
    .unwrap();
    let mut ui = Ui::new(Default::default());
    ui.set_style_pack(&pack).unwrap();
    ui
}

#[test]
fn a_replaced_default_style_dresses_what_is_read_after_it() {
    let compiled = ink(&Ui::new(Default::default()));
    assert_ne!(compiled, [255, 0, 0]);
    let edited = DEFAULT_CSS.replace("--ink: var(--color-fg);", "--ink: #ff0000;");
    assert_ne!(edited, DEFAULT_CSS);

    assert!(set_base_css(Some(&edited)).unwrap());
    assert!(
        !set_base_css(Some(&edited)).unwrap(),
        "the same text again changes nothing"
    );
    assert_eq!(ink(&Ui::new(Default::default())), [255, 0, 0]);
    assert_eq!(
        ink(&packed()),
        [255, 0, 0],
        "a pack lays its rules over the replaced default"
    );

    // CSS that does not read is refused and leaves the replaced default.
    assert!(set_base_css(Some("@layer og-default { :root { --ink: red; }")).is_err());
    assert_eq!(ink(&Ui::new(Default::default())), [255, 0, 0]);

    assert!(set_base_css(None).unwrap());
    assert_eq!(ink(&Ui::new(Default::default())), compiled);
    assert_eq!(ink(&packed()), compiled);
}
