//! A development host's default style (`og_ui::set_base_css`): a UI and a
//! pack read after it wear it, CSS that does not read changes nothing, and
//! `None` puts the compiled default back, and only the documents a changed
//! rule reaches read again. Its own binary: the default style is the
//! process's, so no other test may read it meanwhile (and its tests take
//! [`ONE`] in turn).

use og_ui::{DEFAULT_CSS, Ui, set_base_css, style_pack::StylePack};

static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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

/// An edited default style reads again only the documents it dresses
/// otherwise: none for a token, those a changed role rule reaches for its
/// body, every one for a changed selector.
#[test]
fn an_edited_default_style_restyles_only_the_documents_it_reaches() {
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    set_base_css(None).unwrap();
    let mut ui = Ui::new(Default::default());
    for (name, body) in [
        (
            "framed",
            "<meta name=\"parts\" content=\"framed.frame\"><div part=\"framed.frame\"><p>x</p></div>",
        ),
        ("plain", "<p>x</p>"),
    ] {
        let text = format!("<meta name=\"component\" content=\"{name}\">{body}");
        let doc = ui
            .parse_document(&format!("ui/{name}.svelte"), &text, Vec::new())
            .unwrap();
        ui.define_document(doc);
    }
    let restyled = |ui: &Ui| {
        let mut r = ui.restyled_documents();
        r.sort();
        r
    };
    assert!(restyled(&ui).is_empty(), "nothing changed");

    let token = DEFAULT_CSS.replace("--ink: var(--color-fg);", "--ink: #ff0000;");
    assert!(set_base_css(Some(&token)).unwrap());
    assert!(restyled(&ui).is_empty(), "a token is the root cascade's");

    let frame = "    padding: var(--frame-pad);\n    border: var(--frame-rim-width)";
    assert!(DEFAULT_CSS.contains(frame));
    let body = token.replacen(
        frame,
        "    padding: 3px;\n    border: var(--frame-rim-width)",
        1,
    );
    assert!(set_base_css(Some(&body)).unwrap());
    assert_eq!(
        restyled(&ui),
        ["framed"],
        "the frame rule reaches only the framed one"
    );

    let selector = body.replacen("hr[vertical] {", "hr[vertical], p.zz {", 1);
    assert_ne!(selector, body);
    assert!(set_base_css(Some(&selector)).unwrap());
    assert_eq!(
        restyled(&ui),
        ["framed", "plain"],
        "a new selector may reach any"
    );
    set_base_css(None).unwrap();
}
