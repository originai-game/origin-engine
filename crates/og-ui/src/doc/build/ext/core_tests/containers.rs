use crate::doc::build::ext::*;
use crate::{InputFrame, Ui, UiDesc, Viewport};
use serde_json::json;

const DOC: &str = r#"<meta name="screen" content="q" data-layer="hud">
<row id="Root">
  <column id="Side"><panel id="SideBadge"/></column>
  <column id="Main">
<column id="Inner">
  <panel id="MainBadge"/>
  <column id="Wide"><panel id="WideDot"/></column>
  <panel id="Hint"/>
</column>
  </column>
</row>
<style>
#Root { width: stretch; gap: 0px; padding: 0px; }
#Side { width: 150px; height: 120px; padding: 0px; container: side rail / inline-size; }
#Main { width: stretch; height: 200px; padding: 0px; container-type: size; }
#Inner { width: stretch; padding: 0px; gap: 0px; }
#Wide { display: none; padding: 0px; }
#WideDot { width: 6px; height: 4px; }
#Hint { height: 5px; }
@container (width > 300px) { #Wide { display: initial; } }
@container (width <= 300px) { #Hint { display: none; } }
panel { width: 10px; height: 10px; }
#MainBadge { width: var(--badge, 10px); }
@container (width > 300px) { #Inner { --badge: 40px; } }
@container rail (width < 200px) { #SideBadge { height: 33px; } }
@container (height >= 200px) { #MainBadge { height: 21px; } }
@container (width > 300px) { #SideBadge { width: 27px; } }
</style>"#;

fn frame(ui: &mut Ui, width: f32) {
    ui.begin(InputFrame::default(), Viewport::new(width, 480.0));
    ui.end();
}

fn size(ui: &Ui, path: &str) -> (f32, f32) {
    let i = (0..ui.tree.nodes.len())
        .find(|&i| ui.tree.path_of(i) == path)
        .unwrap_or_else(|| panic!("no {path}"));
    (
        ui.tree.nodes[i].placed.rect.width,
        ui.tree.nodes[i].placed.rect.height,
    )
}

#[test]
fn container_conditions_settle_on_the_first_frame_and_on_resize() {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(crate::doc::UiDocument::parse("q.svelte", DOC).unwrap());
    ui.push_screen("q", json!({})).unwrap();
    let side = "q/Root/Side/SideBadge";
    let main = "q/Root/Main/Inner/MainBadge";
    // 640 wide: Main is 490, a size container 200 high; Side is a
    // 150-wide inline-size container named side and rail.
    frame(&mut ui, 640.0);
    assert_eq!(size(&ui, "q/Root/Main"), (490.0, 200.0));
    // SideBadge: rail answers width < 200, and the unnamed width > 300
    // reads its nearest container, Side (150), and fails. Its height
    // query finds no size container among its ancestors: unknown, false.
    assert_eq!(size(&ui, side), (10.0, 33.0));
    // MainBadge: --badge set on Inner by Main's width, height by Main's
    // block size, both settled before this first frame was painted.
    assert_eq!(size(&ui, main), (40.0, 21.0));
    // Wide was built hidden (unknown on the first build) and revealed by
    // settlement with its child laid out: 21 + 4 + 5.
    assert_eq!(size(&ui, "q/Root/Main/Inner/Wide/WideDot"), (6.0, 4.0));
    assert_eq!(size(&ui, "q/Root/Main/Inner").1, 30.0);
    let saved = ui.save_state();
    frame(&mut ui, 400.0);
    assert_eq!(size(&ui, "q/Root/Main").0, 250.0);
    assert_eq!(size(&ui, main), (10.0, 21.0));
    // Both conditional elements now take no room.
    assert_eq!(size(&ui, "q/Root/Main/Inner").1, 21.0);
    assert_eq!(size(&ui, side), (10.0, 33.0));
    frame(&mut ui, 900.0);
    assert_eq!(size(&ui, main), (40.0, 21.0));
    assert_eq!(size(&ui, "q/Root/Main/Inner").1, 30.0);
    ui.restore_state(&saved).unwrap();
    frame(&mut ui, 400.0);
    assert_eq!(size(&ui, main), (10.0, 21.0));
    frame(&mut ui, 640.0);
    assert_eq!(size(&ui, main), (40.0, 21.0));
    assert!(ui.out.warnings.is_empty(), "{:?}", ui.out.warnings);
}
