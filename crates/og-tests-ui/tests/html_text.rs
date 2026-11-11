//! HTML text: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/html_text.rs.

use crate::support;
use og_ui::reactive::{SourceId, wire};
use og_ui::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use support::*;

const PAGE: &str = r#"<meta name="component" content="t">
<main id="Page">
  <header><h1 i18n="h1">Patch notes</h1><p i18n="intro">Season two brings new maps.</p></header>
  <nav aria-label="Sections"><a href="maps" i18n="maps">Maps</a><a href="heroes" i18n="heroes">Heroes</a><a href="shop" i18n="shop">Shop</a></nav>
  <section aria-label="Changes">
    <h2 i18n="h2">Changes</h2>
    <ul>
      <li i18n="a">Faster matchmaking</li>
      <li><span i18n="b">New modes</span>
        <ul><li i18n="c">Duel</li><li><span i18n="d">Siege</span><ul><li i18n="e">Night</li></ul></li></ul>
      </li>
    </ul>
    <h3 i18n="h3">Steps</h3>
    <ol>{#each $steps as s (s.id)}<li>{s.id}</li>{/each}</ol>
    <ol id="Roman"><li i18n="r1">Prologue</li><li i18n="r2">Ascent</li><li i18n="r3">Fall</li><li i18n="r4">Epilogue</li></ol>
    <p>Read the <a href="rules">rules</a> and <b>enjoy</b> the <i>season</i>.</p>
  </section>
</main>
<style>
main { padding: 24px; gap: 12px; }
nav { display: flex; gap: 16px; }
#Roman { list-style-type: upper-roman; }
</style>"#;

fn steps(ids: &[&str]) -> Value {
    json!(ids.iter().map(|id| json!({ "id": id })).collect::<Vec<_>>())
}

#[test]
#[ignore = "requires GPU"]
fn html_text_matches_its_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/t.svelte", PAGE).unwrap());
        let source = SourceId::new("test", "steps");
        let reply = wire::decode(
            &ui.apply_reactive(
                &wire::encode(&json!([{"op": "collection", "source": source, "key": "id",
                "items": steps(&["Download", "Install", "Play"])}]))
                .unwrap(),
            ),
        )
        .unwrap();
        assert_eq!(reply[0]["ok"], true, "{reply}");
        ui.mount_document(
            SourceId::new("test", "t"),
            "t",
            BTreeMap::from([("steps".to_string(), source)]),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let vp = Viewport::new(width as f32, height as f32);
        for _ in 0..3 {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        let frame = render_ui(&ui, &mut scene, width, height);
        assert_eq!(frame.stats.missing_glyphs, 0);
        let name = format!("html-text-{width}");
        matches_reference(&name, &frame.image, &frame.info, Against::Recorded);
    }
}
