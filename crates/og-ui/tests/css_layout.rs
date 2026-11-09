//! Standard CSS layout in documents (docs/ui.md §2, HTML blocks and
//! lists): the box model, flex, grid, positioning, display, multi-column
//! and writing modes, each built both ways a document is (immediate and
//! mounted, retained).
//!
//! Ways these could fail, each covered below:
//! - `width: auto` refused or shrink-wrapping a block, a bound alone
//!   (`max-width`) turning a block into a fit-content box, a percentage or
//!   content-keyword bound ignored, `box-sizing: content-box` not adding
//!   the padding, a padding longhand ignored or beating a later shorthand,
//!   `aspect-ratio: a / b` refused, `min-content`/`max-content` not
//!   sizing by the content, or the logical sizes not mapping to width and
//!   height → `the_box_model_sizes_as_css_does`
//! - flex factors laid out by the house fill instead of CSS's
//!   proportional shares (or shrinking by factor alone, not scaled by the
//!   basis), a reversed direction that reverses the order but not the
//!   side, `space-evenly`/`safe`/`align-self` ignored, `order` ignored, a
//!   `wrap-reverse` stacking lines from the top, a wrapping column that
//!   does not wrap, or the row and column gaps swapped →
//!   `flex_lays_out_as_css_flexbox_does`
//! - `auto-fit` keeping its empty tracks (so a centred grid does not
//!   centre its one item), a named, negative or spanning-to-a-name line
//!   resolved to the wrong track, a `1fr` track sized below its item's
//!   min-content (or the other `fr` not taking what is left), `auto`
//!   tracks not stretching under `normal` (or stretching under `start`),
//!   the `grid-template` areas or the `grid` auto-flow forms misread, a
//!   subgrid sizing its own tracks, masonry placing in rows, or
//!   `justify-items` ignored → `grid_lays_out_as_css_grid_does`
//! - `display: contents` keeping its box (its children not the flex
//!   container's items), inline-level boxes stacked as blocks (or a block
//!   among them not on a line of its own), a multi-keyword display refused,
//!   a flow-root letting its child's margin collapse through, a table's
//!   colspan or rowspan placing a cell in the wrong slot, a spanning cell's
//!   width not spread over its columns, a row box not over its row, or a
//!   bottom caption on top → `display_lays_out_as_css_display_does`

use crate::html_text::{Doc, both};
use og_core::glam::Vec2;
use og_ui::InputFrame;

fn wh(d: &Doc, id: &str) -> (f32, f32) {
    let e = d.get(id);
    (e.width, e.height)
}

#[test]
fn the_box_model_sizes_as_css_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Auto"></div>
  <div id="AutoMax"></div>
  <div id="PctMax"></div>
  <div id="Content"></div>
  <div id="Sides"><div id="SidesIn"></div></div>
  <div id="Later"><div id="LaterIn"></div></div>
  <div id="Ratio"></div>
  <div id="Logical"></div>
  <div id="Fit"><p id="FitText">alpha beta gamma delta</p></div>
  <p id="MinC">alpha beta</p>
  <p id="MaxC">alpha beta</p>
</div></div>
<style>
#Root { width: 300px; }
#Auto { width: 120px; height: 4px; }
#Auto { width: auto; }
#AutoMax { max-width: 120px; height: 4px; }
#PctMax { width: 100%; max-width: 50%; min-height: 6px; }
#Content { width: 100px; height: 10px; padding: 10px; box-sizing: content-box; }
#Sides { padding: 10px; padding-left: 30px; padding-block-end: 2px; }
#SidesIn, #LaterIn { height: 4px; }
#Later { padding-left: 30px; padding: 4px; }
#Ratio { width: 160px; aspect-ratio: 16 / 9; }
#Logical { inline-size: 50px; block-size: 20px; max-block-size: 12px; }
#Fit { width: fit-content(60px); }
#MinC { width: min-content; }
#MaxC { width: max-content; }
</style>"#;
    both(text, &[], |d| {
        assert_eq!(wh(d, "Auto"), (300.0, 4.0), "auto fills a block's width");
        assert_eq!(
            d.get("AutoMax").width,
            120.0,
            "a bound alone keeps the fill"
        );
        assert_eq!(wh(d, "PctMax"), (150.0, 6.0));
        assert_eq!(wh(d, "Content"), (120.0, 30.0));
        let (sides, inner) = (d.get("Sides"), d.get("SidesIn"));
        assert_eq!((inner.x - sides.x, inner.y - sides.y), (30.0, 10.0));
        assert_eq!(inner.width, 300.0 - 40.0);
        assert_eq!(sides.height, 10.0 + 4.0 + 2.0);
        let (later, inner) = (d.get("Later"), d.get("LaterIn"));
        assert_eq!(
            inner.x - later.x,
            4.0,
            "a later shorthand resets the longhand"
        );
        assert_eq!(wh(d, "Ratio"), (160.0, 90.0));
        assert_eq!(wh(d, "Logical"), (50.0, 12.0));
        assert_eq!(
            d.get("Fit").width,
            60.0,
            "fit-content(60px) caps its content"
        );
        let (min, max) = (d.get("MinC"), d.get("MaxC"));
        assert!(min.width < max.width, "{} {}", min.width, max.width);
        assert_eq!((min.height, max.height), (40.0, 20.0));
    });
}

#[test]
fn flex_lays_out_as_css_flexbox_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Row"><div id="A"></div><div id="B"></div><div id="C"></div></div>
  <div id="Rev"><div id="R1"></div><div id="R2"></div></div>
  <div id="Even"><div id="E1"></div><div id="E2"></div></div>
  <div id="Ord"><div id="O1"></div><div id="O2"></div></div>
  <div id="Shrink"><div id="S1"></div><div id="S2"></div></div>
  <div id="Safe"><div id="F1"></div></div>
  <div id="Self"><div id="L1"></div><div id="L2"></div></div>
  <div id="Wrap"><div id="W1"></div><div id="W2"></div><div id="W3"></div></div>
  <div id="Col"><div id="C1"></div><div id="C2"></div><div id="C3"></div></div>
</div></div>
<style>
#Root { width: 300px; }
#Row, #Rev, #Even, #Ord, #Shrink, #Safe, #Self, #Wrap, #Col { display: flex; }
#Row { column-gap: 10px; row-gap: 99px; }
#A { flex: 1; height: 4px; }
#B { flex: 2; height: 4px; }
#C { flex: none; width: 40px; height: 4px; }
#Rev { flex-direction: row-reverse; justify-content: flex-start; }
#R1, #R2 { width: 50px; height: 4px; }
#Even { justify-content: space-evenly; }
#E1, #E2 { width: 60px; height: 4px; }
#O1 { order: 2; width: 10px; height: 4px; }
#O2 { width: 20px; height: 4px; }
#Shrink { width: 200px; }
#S1 { flex: 0 1 auto; width: 150px; height: 4px; }
#S2 { flex: 0 3 auto; width: 150px; height: 4px; }
#Safe { justify-content: safe center; width: 100px; }
#F1 { flex: none; width: 160px; height: 4px; }
#Self { height: 40px; align-items: flex-start; }
#L1 { width: 10px; height: 10px; align-self: flex-end; }
#L2 { width: 10px; height: 10px; }
#Wrap { flex-flow: row wrap-reverse; width: 100px; row-gap: 6px; }
#W1, #W2, #W3 { width: 60px; height: 10px; }
#Col { flex-flow: column wrap; height: 50px; column-gap: 8px; }
#C1, #C2, #C3 { width: 30px; height: 20px; }
</style>"#;
    both(text, &[], |d| {
        let at = |id: &str, parent: &str| {
            let (e, p) = (d.get(id), d.get(parent));
            (e.x - p.x, e.y - p.y)
        };
        assert_eq!(
            (d.get("A").width, d.get("B").width, d.get("C").width),
            (80.0, 160.0, 40.0)
        );
        assert_eq!((at("B", "Row").0, at("C", "Row").0), (90.0, 260.0));
        assert_eq!((at("R1", "Rev").0, at("R2", "Rev").0), (250.0, 200.0));
        assert_eq!((at("E1", "Even").0, at("E2", "Even").0), (60.0, 180.0));
        assert_eq!((at("O2", "Ord").0, at("O1", "Ord").0), (0.0, 20.0));
        // 100px of overflow shared by factor times basis: 1:3.
        assert_eq!((d.get("S1").width, d.get("S2").width), (125.0, 75.0));
        assert_eq!(
            at("F1", "Safe").0,
            0.0,
            "safe keeps the overflow at the start"
        );
        assert_eq!((at("L1", "Self").1, at("L2", "Self").1), (30.0, 0.0));
        assert_eq!(d.get("Wrap").height, 42.0);
        assert_eq!(
            [at("W1", "Wrap").1, at("W2", "Wrap").1, at("W3", "Wrap").1],
            [32.0, 16.0, 0.0]
        );
        assert_eq!(
            (at("C2", "Col"), at("C3", "Col")),
            ((0.0, 20.0), (38.0, 0.0))
        );
    });
}

#[test]
fn grid_lays_out_as_css_grid_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Fill"><div id="F1"></div><div id="F2"></div><div id="F3"></div><div id="F4"></div></div>
  <div id="Fit"><div id="T1"></div></div>
  <div id="Named"><div id="N1"></div><div id="N2"></div><div id="N3"></div></div>
  <div id="Fr"><div id="R1"></div><div id="R2"></div></div>
  <div id="Auto"><div id="A1"></div><div id="A2"></div></div>
  <div id="AutoStart"><div id="S1"></div><div id="S2"></div></div>
  <div id="Areas"><div id="H"></div><div id="M"></div></div>
  <div id="Flow"><div id="W1"></div><div id="W2"></div><div id="W3"></div></div>
  <div id="Parent"><div id="Sub"><div id="U1"></div><div id="U2"></div><div id="U3"></div></div></div>
  <div id="Mason"><div id="K1"></div><div id="K2"></div><div id="K3"></div></div>
  <div id="Items"><div id="I1"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Fill, #Fit, #Named, #Fr, #Auto, #AutoStart, #Areas, #Flow, #Parent, #Sub, #Mason, #Items { display: grid; }
#Fill { width: 320px; grid-template-columns: repeat(auto-fill, 100px); gap: 10px; }
#Fill > div, #T1 { height: 5px; }
#Fit { width: 320px; grid-template-columns: repeat(auto-fit, 100px); column-gap: 10px; justify-content: center; }
#Named { width: 180px; grid-template-columns: [a] 50px [b] 60px [c] 70px [d]; }
#Named > div { height: 5px; }
#N1 { grid-column: b / d; }
#N2 { grid-column: -2 / -1; }
#N3 { grid-column-start: span 2; grid-column-end: c; }
#Fr { width: 100px; grid-template-columns: 1fr 1fr; }
#R1 { width: 80px; height: 5px; }
#R2 { height: 5px; }
#Auto, #AutoStart { width: 200px; grid-template-columns: auto 50px; }
#AutoStart { justify-content: start; }
#A1, #S1 { width: 20px; height: 5px; justify-self: start; }
#A2, #S2 { height: 5px; }
#Areas { width: 200px; grid-template: "h h" 30px "s m" 40px / 60px 1fr; }
#H { grid-area: h; }
#M { grid-area: m; }
#Flow { width: 200px; grid: auto-flow 25px / 1fr 1fr; }
#Parent { width: 180px; grid-template-columns: 40px 60px 80px; }
#Sub { grid-column: 1 / 4; grid-template-columns: subgrid; }
#Sub > div { height: 5px; }
#Mason { width: 100px; grid-template-columns: 50px 50px; grid-template-rows: masonry; }
#K1 { height: 30px; }
#K2, #K3 { height: 10px; }
#Items { width: 100px; grid-template-columns: 100px; justify-items: center; }
#I1 { width: 20px; height: 5px; }
</style>"#;
    both(text, &[], |d| {
        let at = |id: &str, of: &str| {
            let (e, p) = (d.get(id), d.get(of));
            (e.x - p.x, e.y - p.y)
        };
        // 320px holds three 100px tracks with 10px gaps.
        assert_eq!(at("F3", "Fill"), (220.0, 0.0));
        assert_eq!(at("F4", "Fill"), (0.0, 15.0));
        assert_eq!(at("T1", "Fit").0, 110.0, "auto-fit's empty tracks collapse");
        assert_eq!((at("N1", "Named").0, d.get("N1").width), (50.0, 130.0));
        assert_eq!((at("N2", "Named").0, d.get("N2").width), (110.0, 70.0));
        assert_eq!((at("N3", "Named").0, d.get("N3").width), (0.0, 110.0));
        // The 1fr holding an 80px item keeps it; the other takes the rest.
        assert_eq!((at("R2", "Fr").0, d.get("R2").width), (80.0, 20.0));
        assert_eq!(at("A2", "Auto").0, 150.0, "normal stretches auto tracks");
        assert_eq!(at("S2", "AutoStart").0, 20.0);
        assert_eq!(wh(d, "H"), (200.0, 30.0));
        assert_eq!(
            (at("M", "Areas"), wh(d, "M")),
            ((60.0, 30.0), (140.0, 40.0))
        );
        assert_eq!(
            (at("W3", "Flow"), wh(d, "W3")),
            ((0.0, 25.0), (100.0, 25.0))
        );
        assert_eq!(
            (at("U2", "Sub").0, d.get("U3").width),
            (40.0, 80.0),
            "a subgrid takes its parent's tracks"
        );
        assert_eq!(
            at("K3", "Mason"),
            (50.0, 10.0),
            "masonry packs the shortest track"
        );
        assert_eq!(d.get("Mason").height, 30.0);
        assert_eq!(at("I1", "Items").0, 40.0);
    });
}

#[test]
fn display_lays_out_as_css_display_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Flex"><div id="Box"><div id="A"></div><div id="B"></div></div></div>
  <div id="Line"><div id="I1"></div><div id="I2"></div><div id="Blk"></div><div id="I3"></div></div>
  <div id="Fr"><div id="M"></div></div>
  <div id="Tbl">
    <div id="R1"><div id="C1" class="td" colspan="2"></div><div id="C2" class="td" rowspan="2"></div></div>
    <div id="R2"><div id="D1" class="td"></div><div id="D2" class="td"></div></div>
    <div id="Cap"></div>
  </div>
</div></div>
<style>
#Root { width: 300px; }
#Flex { display: flex; }
#Box { display: contents; background: #f00; }
#A, #B { width: 40px; height: 4px; }
#Line { width: 200px; }
#I1, #I2, #I3 { display: inline-block; width: 50px; height: 10px; }
#I2 { display: inline flow-root; }
#Blk { height: 5px; }
#Fr { display: flow-root; }
#M { margin-top: 20px; height: 5px; }
#Tbl { display: table; }
#R1, #R2 { display: table-row; background: #00f; }
.td { display: table-cell; }
#C1 { width: 100px; height: 10px; }
#C2 { width: 20px; height: 30px; }
#D1 { width: 30px; height: 10px; }
#D2 { width: 40px; height: 10px; }
#Cap { display: table-caption; caption-side: bottom; height: 8px; }
</style>"#;
    both(text, &[], |d| {
        let at = |id: &str, of: &str| {
            let (e, p) = (d.get(id), d.get(of));
            (e.x - p.x, e.y - p.y)
        };
        assert_eq!(
            at("B", "Flex"),
            (40.0, 0.0),
            "contents' children are flex items"
        );
        assert_eq!(at("I2", "Line"), (50.0, 0.0));
        assert_eq!(
            (at("Blk", "Line"), d.get("Blk").width),
            ((0.0, 10.0), 200.0)
        );
        assert_eq!(
            at("I3", "Line"),
            (0.0, 15.0),
            "a block is a line of its own"
        );
        assert_eq!((at("M", "Fr").1, d.get("Fr").height), (20.0, 25.0));
        // Columns 30 and 40 grow to hold the 100px spanning cell: 45, 55.
        assert_eq!(at("C2", "Tbl"), (100.0, 0.0));
        assert_eq!(wh(d, "C2"), (20.0, 30.0));
        assert_eq!(at("D2", "Tbl"), (45.0, 15.0));
        assert_eq!((at("R2", "Tbl"), wh(d, "R2")), ((0.0, 15.0), (120.0, 15.0)));
        assert_eq!((at("Cap", "Tbl"), d.get("Cap").width), ((0.0, 30.0), 120.0));
        assert_eq!(wh(d, "Tbl"), (120.0, 38.0), "a table shrinks to fit");
    });
}

/// HTML's table elements are table parts by the UA stylesheet alone: no
/// author `display`, and the cells sit side by side in their rows.
#[test]
fn html_tables_lay_out_without_author_css() {
    let text = r#"<meta name="component" content="t">
<div><table id="Tbl">
  <caption id="Cap">Scores</caption>
  <thead><tr><th id="H1">Name</th><th id="H2">Pts</th></tr></thead>
  <tbody><tr><td id="A" colspan="2"></td></tr><tr><td id="B"></td><td id="C"></td></tr></tbody>
</table></div>
<style>
#A { width: 60px; height: 10px; }
#B, #C { width: 20px; height: 10px; }
</style>"#;
    both(text, &[], |d| {
        let (h1, h2, b, c, a, cap) = (
            d.get("H1"),
            d.get("H2"),
            d.get("B"),
            d.get("C"),
            d.get("A"),
            d.get("Cap"),
        );
        assert_eq!(h1.y, h2.y, "header cells share a row");
        assert!(h2.x > h1.x);
        assert_eq!((b.y, c.y > b.y), (c.y, false), "body cells share a row");
        assert!(c.x > b.x && a.y < b.y && a.width >= 60.0);
        assert!(cap.y < h1.y, "the caption is above the table");
    });
}

#[test]
fn positioning_places_as_css_positioning_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Col"><div id="Rel"></div><div id="Next"></div></div>
  <div id="Stage">
    <div id="Anc"></div>
    <div id="Corner"></div>
    <div id="Fill"></div>
    <div id="Area"></div>
    <div id="Edge"></div>
    <div id="Flip"></div>
    <div id="Named"></div>
  </div>
  <div id="Port"><div id="Head"></div><div id="Body"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Rel { position: relative; left: 10px; top: 5px; height: 20px; }
#Next { height: 10px; }
#Stage { position: relative; display: flow-root; width: 300px; height: 200px; }
#Anc { anchor-name: --a; margin-left: 50px; margin-top: 150px; width: 40px; height: 20px; }
#Corner { position: absolute; right: 10px; bottom: 25%; width: 30px; height: 10px; }
#Fill { position: absolute; inset: 10px 20px; }
#Area { position: absolute; position-anchor: --a; position-area: top span-right; width: 60px; height: 30px; }
#Edge { position: absolute; left: anchor(--a right); top: calc(anchor(--a top) - 4px); width: anchor-size(--a width); height: 6px; }
#Flip { position: absolute; position-anchor: --a; position-area: bottom; position-try-fallbacks: flip-block; width: 10px; height: 40px; }
#Named { position: absolute; position-anchor: --a; top: anchor(bottom); left: anchor(left); position-try-fallbacks: --above; width: 10px; height: 40px; }
@position-try --above { top: auto; bottom: anchor(top); }
#Port { height: 50px; overflow-y: auto; scrollbar-width: none; }
#Head { position: sticky; top: 0; height: 10px; }
#Body { height: 100px; }
</style>"#;
    both(text, &[], |d| {
        let at = |id: &str, of: &str| {
            let (e, p) = (d.get(id), d.get(of));
            (e.x - p.x, e.y - p.y)
        };
        assert_eq!(
            at("Rel", "Col"),
            (10.0, 5.0),
            "relative moves by its insets"
        );
        assert_eq!(at("Next", "Col"), (0.0, 20.0), "its siblings keep its room");
        assert_eq!(
            at("Corner", "Stage"),
            (260.0, 140.0),
            "right and a % bottom"
        );
        assert_eq!(
            (at("Fill", "Stage"), wh(d, "Fill")),
            ((20.0, 10.0), (260.0, 180.0))
        );
        // The anchor is at (50, 150) 40×20; span-right starts at its left.
        assert_eq!(at("Area", "Stage"), (50.0, 120.0), "position-area");
        assert_eq!(
            (at("Edge", "Stage"), wh(d, "Edge")),
            ((90.0, 146.0), (40.0, 6.0))
        );
        // Below the anchor it would overflow the stage: flipped above it,
        // centred across it (span-all's room is the anchor's column).
        assert_eq!(at("Flip", "Stage"), (65.0, 110.0), "flip-block");
        assert_eq!(at("Named", "Stage"), (50.0, 110.0), "@position-try");
        let p = d.get("Port");
        d.step(InputFrame {
            wheel: Vec2::new(0.0, 30.0),
            ..InputFrame::hover(p.x + 10.0, p.y + 10.0)
        });
        for _ in 0..40 {
            d.step(InputFrame::hover(p.x + 10.0, p.y + 10.0));
        }
        let s = d.ui.scroll_state(&p.path).expect("a scroll container");
        assert_eq!(s.offset_y, 30.0);
        // Scrolled 30 its flow place is 20px above the view; sticky holds
        // it whole at the view's top (bounds are the part that shows).
        let h = d.get("Head");
        assert_eq!(
            (h.y - p.y, h.height, h.clipped),
            (0.0, 10.0, false),
            "sticky"
        );
    });
}
