use crate::ScreenWipe;
use crate::Side;
use crate::UiDocument;
use crate::doc::build::ext::*;
use crate::doc::view_transition::*;

fn spec(css: &str) -> Result<Option<ViewTransition>, String> {
    UiDocument::parse(
        "t.svelte",
        &format!(
            "<meta name=\"screen\" content=\"s\"><column id=\"C\"/><style>@keyframes rise {{ from {{ translate: 0 40px; opacity: 0; }} }}\n{css}</style>"
        ),
    )
    .map(|d| d.layer.transition)
    .map_err(|e| e.join("\n"))
}

/// The UA's cross-fade by default; old and new take animation, the
/// group's timing is inherited, types gate typed rules, and the
/// built-in names stay unqualified.
#[test]
fn view_transitions_read_the_spec_pseudo_elements() {
    let t = spec("@view-transition { navigation: auto; }")
        .unwrap()
        .unwrap();
    assert_eq!(
        (t.old[0].name.as_str(), t.new[0].name.as_str()),
        (FADE_OUT, FADE_IN)
    );
    assert_eq!((t.old[0].duration_ms, t.new[0].duration_ms), (250.0, 250.0));
    let t = spec(
        "@view-transition { navigation: auto; types: slide; }
         ::view-transition-group(root) { animation-duration: 600ms; }
         ::view-transition-new(root) { animation-name: rise; }
         :root:active-view-transition-type(slide)::view-transition-old(root) { animation: 300ms -og-iris; }
         :active-view-transition-type(other)::view-transition-old(root) { animation: 1s -og-wipe; }
         ::view-transition-old(root) { animation: 900ms -og-dissolve; }
         ::view-transition { background-color: #102030; background-image: url(\"ui/covers/stars.png\"); }",
    )
    .unwrap()
    .unwrap();
    assert_eq!(t.new[0].name, "s/rise");
    assert_eq!(
        t.new[0].duration_ms, 600.0,
        "the group's duration is inherited"
    );
    // The typed rule outranks the later plain one; the other type's does not apply.
    assert_eq!(
        (t.old[0].name.as_str(), t.old[0].duration_ms),
        ("-og-iris", 300.0)
    );
    assert_eq!(t.color.unwrap().rgb, [0x10, 0x20, 0x30]);
    assert_eq!(t.image.as_deref(), Some("ui/covers/stars.png"));
    assert_eq!(
        cover("-og-wipe-from-right"),
        Some((ScreenWipe::Wipe, Side::Right))
    );
    // Pseudo-element rules without the opt-in transition nothing.
    assert!(
        spec("::view-transition-new(root) { animation: 1s rise; }")
            .unwrap()
            .is_none()
    );
    for (css, wanted) in [
        (
            "@view-transition { navigation: auto; } ::view-transition-old(card.) { animation: 1s rise; }",
            "is not a class",
        ),
        (
            "@view-transition { navigation: auto; } ::view-transition-old(1up) { animation: 1s rise; }",
            "its argument is",
        ),
        ("#C { view-transition-name: root; }", "the layer's own name"),
        (
            "#C { view-transition-class: none card; }",
            "view-transition-class takes",
        ),
        (
            "@view-transition { navigation: auto; } ::view-transition-new(root) { opacity: 0; }",
            "takes animation",
        ),
        (
            "@view-transition { navigation: auto; } ::view-transition-new(root) { animation: 1s nowhere; }",
            "nowhere",
        ),
        (
            "@media (width > 1px) { @view-transition { navigation: auto; } }",
            "top level",
        ),
        (
            "@view-transition { navigation: auto; } ::view-transition-old(*) { overflow: clip; }",
            "takes animation",
        ),
        (
            "@view-transition { navigation: auto; } ::view-transition-group(*) { overflow: cut; }",
            "takes visible, hidden, clip",
        ),
    ] {
        let error = spec(css).unwrap_err();
        assert!(error.contains(wanted), "{css}: {error}");
    }
    // Named groups: `*` under a name under a class, whatever the order;
    // the images inherit the group's timing; the root takes `*` too.
    let t = spec(
        "@view-transition { navigation: auto; }
         ::view-transition-group(*) { animation-duration: 1s; }
         ::view-transition-group(*.card) { animation-duration: 700ms; }
         ::view-transition-group(hero) { animation-duration: 300ms; }
         ::view-transition-new(hero) { animation: 200ms rise; }
         ::view-transition-image-pair(.card.big) { animation-delay: 50ms; }",
    )
    .unwrap()
    .unwrap();
    let g = t.group("hero", &["card".into()]);
    assert_eq!(
        (g.group[0].name.as_str(), g.group[0].duration_ms),
        (GROUP_ANIM, 700.0)
    );
    assert_eq!(
        (g.old[0].name.as_str(), g.old[0].duration_ms),
        (FADE_OUT, 700.0)
    );
    assert_eq!(
        (g.new[0].name.as_str(), g.new[0].duration_ms),
        ("s/rise", 200.0)
    );
    assert_eq!(g.old[0].delay_ms, 0.0, "one class of two is no match");
    let g = t.group("hero", &["big".into(), "card".into()]);
    assert_eq!((g.pair.len(), g.old[0].delay_ms), (0, 50.0));
    assert_eq!(t.group("hero", &[]).group[0].duration_ms, 300.0);
    assert_eq!(t.group("other", &[]).new[0].duration_ms, 1000.0);
    assert_eq!(t.new[0].duration_ms, 1000.0, "the root is one of *");
    // `overflow` cascades like the rest, axis by axis; the pair's is
    // its own (not inherited from the group).
    let t = spec(
        "@view-transition { navigation: auto; }
         ::view-transition-group(*) { overflow: clip; }
         ::view-transition-group(hero) { overflow-x: visible; }
         ::view-transition-image-pair(.card) { overflow: hidden visible; }",
    )
    .unwrap()
    .unwrap();
    let g = t.group("hero", &["card".into()]);
    assert_eq!((g.group_clip, g.pair_clip), ([false, true], [true, false]));
    let g = t.group("other", &[]);
    assert_eq!((g.group_clip, g.pair_clip), ([true, true], [false, false]));
    // The children of a group take overflow and their own animations
    // (nothing of the group's timing inherits into them).
    let t = spec(
        "@view-transition { navigation: auto; }
         ::view-transition-group(*) { animation-duration: 1s; }
         ::view-transition-group-children(hero) { overflow-y: clip; animation: 300ms rise; }",
    )
    .unwrap()
    .unwrap();
    let g = t.group("hero", &[]);
    assert_eq!(g.children_clip, [false, true]);
    assert_eq!(
        g.children
            .iter()
            .map(|a| (a.name.as_str(), a.duration_ms))
            .collect::<Vec<_>>(),
        [("s/rise", 300.0)]
    );
    assert!(t.group("other", &[]).children.is_empty());
    let error = spec(
        "@view-transition { navigation: auto; } ::view-transition-group-children(*) { opacity: 0; }",
    )
    .unwrap_err();
    assert!(error.contains("nested groups"), "{error}");
    let error = UiDocument::parse(
        "c.svelte",
        "<meta name=\"component\" content=\"c\"><column/><style>@view-transition { navigation: auto; }</style>",
    )
    .unwrap_err()
    .join("\n");
    assert!(error.contains("a view transition is a screen's"), "{error}");
}
