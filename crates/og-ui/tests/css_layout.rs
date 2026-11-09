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
//! - columns laid out unbalanced or at the container's width, a spanning
//!   child kept in a column (or the set below it not starting again),
//!   `column-width` not fitting as many columns as there is room for,
//!   `column-fill: auto` balancing, `break-before: column` ignored, or a
//!   paragraph moved whole instead of broken between its lines →
//!   `multi_column_lays_out_as_css_multicol_does`
//! - `vertical-align` ignored or measured against the wrong box: a
//!   `<sup>`/`<sub>` raised by its own smaller size, a raised span that
//!   does not grow its line, a length or percentage on an inline-block not
//!   taken from the line's baseline, `middle` centred in the line instead
//!   of on the parent's x-height → `vertical_align_places_inline_boxes_as_css_does`
//! - a float not taken out of the flow, not shifted to its side (or a
//!   second one not beside the first, or one that does not fit not moved
//!   below), the lines beside a float not shortened, a formatting context
//!   box overlapping a float (not narrowed beside it, or a fixed-width one
//!   not moved below), `clear` ignored, a float in a nested block not
//!   reaching the boxes after it, a flow-root not holding its floats, or a
//!   `<p>` only as wide as its text → `floats_lay_out_as_css_floats_do`
//! - `inline-start` not following the containing block's direction, `left`
//!   turned round under a right-to-left surface, or a box narrowed beside
//!   floats not mirrored whole in a right-to-left block →
//!   `floats_take_their_sides_from_the_direction`

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

#[test]
fn the_root_element_fills_its_viewport() {
    // The root is a block of the viewport: a wider child's min-content does
    // not widen it, so `max-width: 100%` holds the child to the root's
    // content box and its aspect ratio follows the clamped width.
    let text = r#"<meta name="component" content="t">
<div><div id="Pad" style="padding:16px"><div id="Card" style="width:1000px;max-width:100%"><div id="Media" style="aspect-ratio:16/9"></div></div></div></div>"#;
    both(text, &[], |d| {
        assert_eq!(d.get("Pad").width, 800.0);
        assert_eq!(wh(d, "Media"), (768.0, 432.0));
    });
}

#[test]
fn multi_column_lays_out_as_css_multicol_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Mc"><div id="A"></div><div id="B"></div><div id="C"></div><div id="D"></div><div id="E"></div><div id="F"></div><div id="Span"></div><div id="G"></div><div id="H"></div></div>
  <div id="W"><div id="W1"></div><div id="W2"></div><div id="W3"></div></div>
  <div id="Fa"><div id="F1"></div><div id="F2"></div><div id="F3"></div></div>
  <div id="Bk"><div id="X"></div><div id="Y"></div><div id="Z"></div></div>
  <div id="Tx"><p id="P">one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty</p></div>
</div></div>
<style>
#Root { width: 400px; }
#Mc { columns: 3; column-gap: 20px; width: 340px; column-rule: 2px solid #e33f; }
#A, #B, #C, #D, #E, #F { height: 30px; }
#Span { column-span: all; height: 10px; }
#G, #H { height: 20px; }
#W { column-width: 100px; column-gap: 10px; width: 320px; }
#W1, #W2, #W3, #X, #Y, #Z { height: 10px; }
#Fa { column-count: 2; column-fill: auto; column-gap: 10px; width: 210px; height: 50px; }
#F1, #F2, #F3 { height: 20px; }
#Bk { columns: 2; column-gap: 0; width: 200px; }
#Y { break-before: column; }
#Tx { columns: 2; column-gap: 0; width: 160px; }
p { margin: 0; }
</style>"#;
    both(text, &[], |d| {
        let at = |id: &str, of: &str| {
            let (e, p) = (d.get(id), d.get(of));
            (e.x - p.x, e.y - p.y)
        };
        // Three columns of (340 − 2·20) / 3 = 100, two blocks in each.
        for (id, xy) in [
            ("A", (0.0, 0.0)),
            ("B", (0.0, 30.0)),
            ("C", (120.0, 0.0)),
            ("D", (120.0, 30.0)),
            ("E", (240.0, 0.0)),
            ("F", (240.0, 30.0)),
        ] {
            assert_eq!((at(id, "Mc"), d.get(id).width), (xy, 100.0), "{id}");
        }
        // The spanning block below the balanced set, a new set under it.
        assert_eq!(
            (at("Span", "Mc"), wh(d, "Span")),
            ((0.0, 60.0), (340.0, 10.0))
        );
        assert_eq!((at("G", "Mc"), at("H", "Mc")), ((0.0, 70.0), (120.0, 70.0)));
        assert_eq!(d.get("Mc").height, 90.0);
        // column-width: as many 100px columns as fit, widened to fill.
        assert_eq!(
            [at("W2", "W"), at("W3", "W")],
            [(110.0, 0.0), (220.0, 0.0)],
            "column-width"
        );
        assert_eq!(d.get("W").height, 10.0);
        // Filled in turn to the 50px height: two blocks, then the next column.
        assert_eq!(at("F3", "Fa"), (110.0, 0.0), "column-fill: auto");
        // break-before starts a column: X alone, Y and Z in the next.
        assert_eq!(
            [at("Y", "Bk"), at("Z", "Bk")],
            [(100.0, 0.0), (100.0, 10.0)],
            "break-before: column"
        );
        // Fifteen 20px lines at the 80px column width break between lines
        // 8 and 9 (balanced): the box is the first column's fragment and
        // the container that tall; the rest are in the second column.
        let (p, tx) = (d.get("P"), d.get("Tx"));
        assert_eq!((p.width, p.height, tx.height), (80.0, 160.0, 160.0));
    });
}

#[test]
fn vertical_align_places_inline_boxes_as_css_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <p id="Plain">x</p>
  <p id="Sup">x<sup>2</sup></p>
  <p id="Sub">x<sub>i</sub></p>
  <p id="Part">x<span part="lift">y</span></p>
  <div id="Line">ab<div id="Mid" class="ib"></div><div id="Up" class="ib"></div><div id="Pc" class="ib"></div><div id="Tp" class="ib"></div></div>
</div></div>
<style>
#Root { width: 300px; }
.ib { display: inline-block; width: 10px; height: 10px; }
#Mid { vertical-align: middle; }
#Up { vertical-align: 4px; }
#Pc { vertical-align: -50%; }
#Tp { vertical-align: top; }
#Part::part(lift) { vertical-align: 6px; }
</style>"#;
    for mounted in [false, true] {
        let mut scene = og_render::RenderScene::new();
        let font = crate::support::add_test_font(&mut scene);
        let d = Doc::with_font(text, &[], mounted, font);
        let base = |t: &str| {
            d.ui.frame()
                .items
                .iter()
                .find_map(|i| match &i.cmd {
                    og_ui::DrawCmd::Text {
                        text,
                        baseline,
                        size,
                        ..
                    } if text == t => Some((*baseline, *size)),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no text {t}"))
        };
        // Against the parent's size (14px), not the smaller sup's own.
        assert_eq!(base("x").1, 14.0);
        let rise = |t: &str, parent: &str| {
            let p = d.get(parent);
            let x =
                d.ui.frame()
                    .items
                    .iter()
                    .find_map(|i| match &i.cmd {
                        og_ui::DrawCmd::Text { text, baseline, .. }
                            if text == "x" && *baseline > p.y && *baseline < p.y + p.height =>
                        {
                            Some(*baseline)
                        }
                        _ => None,
                    })
                    .expect("its x");
            x - base(t).0
        };
        assert!(
            (rise("2", "Sup") - 14.0 / 3.0).abs() <= 0.5,
            "super: a third up"
        );
        assert!(
            (rise("i", "Sub") + 14.0 / 5.0).abs() <= 0.5,
            "sub: a fifth down"
        );
        assert_eq!(rise("y", "Part"), 6.0, "::part's length");
        assert!(base("2").1 < 14.0, "a sup is smaller");
        let line = d.get("Plain").height;
        assert!(d.get("Sup").height > line, "the line grows to hold the sup");
        assert_eq!(d.get("Part").height, line + 6.0);
        // Inline-blocks against the line's baseline: their bottom edge is
        // their baseline, having no text.
        let b = base("ab").0;
        let bottom = |id: &str| d.get(id).y + d.get(id).height;
        assert!((bottom("Up") - (b - 4.0)).abs() <= 0.5, "4px up");
        assert!(
            (bottom("Pc") - (b + line / 2.0)).abs() <= 0.5,
            "-50% of the line height"
        );
        let mid = d.get("Mid").y + 5.0;
        assert!(
            (b - mid) > 0.15 * 14.0 && (b - mid) < 0.35 * 14.0,
            "middle: half an x-height over the baseline ({})",
            b - mid
        );
        assert_eq!(d.get("Tp").y, d.get("Line").y, "top: the line's top");
    }
}

#[test]
fn direction_lays_out_as_css_writing_modes_does() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Row" dir="rtl"><div id="R1"></div><div id="R2"></div></div>
  <div id="Ltr"><div id="L1"></div></div>
  <div id="Log"><div id="G1"></div></div>
  <div id="Abs"><div id="Ab"></div></div>
  <div id="Back" dir="ltr"><div id="B1"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Row { display: flex; width: 300px; }
#R1, #R2 { width: 50px; height: 10px; }
#R1 { margin-inline-start: 5px; }
#Ltr, #Log, #Back { width: 300px; }
#L1 { width: 40px; height: 10px; margin-left: 20px; }
#Log { direction: rtl; }
#G1 { width: 40px; height: 10px; margin-inline-end: 30px; }
#Abs { position: relative; width: 300px; height: 20px; direction: rtl; }
#Ab { position: absolute; left: 10px; width: 20px; height: 5px; }
#Back { display: flex; }
#B1 { width: 40px; height: 10px; }
</style>"#;
    let at = |d: &Doc, id: &str, of: &str| d.get(id).x - d.get(of).x;
    both(text, &[], |d| {
        for rtl_surface in [false, true] {
            if rtl_surface {
                d.ui.set_direction(og_i18n::Direction::Rtl);
                for _ in 0..2 {
                    d.step(InputFrame::default());
                }
            }
            let s = if rtl_surface {
                "rtl surface"
            } else {
                "ltr surface"
            };
            // A right-to-left flex row runs from the right; its start margin
            // is on the right.
            assert_eq!(
                (at(d, "R1", "Row"), at(d, "R2", "Row")),
                (245.0, 195.0),
                "{s}"
            );
            // A block sits at its container's inline start; a physical margin
            // stays on its side, a logical one follows the direction.
            let l1 = if rtl_surface { 260.0 } else { 20.0 };
            assert_eq!(at(d, "L1", "Ltr"), l1, "{s}");
            assert_eq!(at(d, "G1", "Log"), 260.0, "{s}");
            // `left` is the left, whatever the direction.
            assert_eq!(at(d, "Ab", "Abs"), 10.0, "{s}");
            // A left-to-right element in a right-to-left surface.
            assert_eq!(at(d, "B1", "Back"), 0.0, "{s}");
        }
    });
}

#[test]
fn floats_lay_out_as_css_floats_do() {
    let words = "one two three four five six seven eight nine ten eleven twelve";
    let text = format!(
        r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Ref"><p id="RefP">{words}</p></div>
  <div id="Ctx">
    <div id="L"></div><div id="L2"></div><div id="Rt"></div>
    <p id="P">{words}</p>
    <div id="Fr"></div>
    <div id="Wide"></div>
    <div id="Clr"></div>
    <span id="S">fits</span>
  </div>
  <div id="Nb"><div id="In"></div></div>
  <div id="After"></div>
  <div id="Holds"><div id="Hf"></div></div>
</div></div>
<style>
#Root {{ width: 400px; }}
p {{ margin: 0; }}
#Ref {{ width: 160px; }}
#L {{ float: left; width: 100px; height: 300px; }}
#L2 {{ float: inline-start; width: 100px; height: 20px; }}
#Rt {{ float: right; width: 40px; height: 300px; }}
#Fr {{ display: flow-root; height: 10px; }}
#Wide {{ display: flow-root; width: 300px; height: 10px; }}
#Clr {{ clear: both; height: 10px; }}
#Nb {{ height: 0; }}
#In {{ float: left; width: 50px; height: 40px; }}
#After {{ display: flow-root; height: 10px; }}
#Hf {{ float: right; width: 30px; height: 25px; }}
#Holds {{ display: flow-root; }}
</style>"#
    );
    both(&text, &[], |d| {
        let at = |id: &str, of: &str| {
            let (e, p) = (d.get(id), d.get(of));
            (e.x - p.x, e.y - p.y)
        };
        // Floats side by side at their sides, out of the flow.
        assert_eq!(at("L", "Ctx"), (0.0, 0.0));
        assert_eq!(at("L2", "Ctx"), (100.0, 0.0), "beside the first");
        assert_eq!(at("Rt", "Ctx"), (360.0, 0.0));
        assert_eq!(at("P", "Ctx").1, 0.0, "the flow starts at the top");
        // The paragraph fills the line; under the short float its lines
        // have 400 − 200 − 40 = 160px, then 400 − 100 − 40 = 260px.
        let (p, rp) = (d.get("P"), d.get("RefP"));
        assert_eq!(p.width, 400.0, "a <p> fills its line");
        assert_eq!(rp.width, 160.0);
        assert!(
            p.height < rp.height && p.height > 20.0,
            "lines beside the floats: {} against {} at 160px",
            p.height,
            rp.height
        );
        // A flow-root narrows beside the floats; a wider one moves below.
        assert_eq!(at("Fr", "Ctx").0, 100.0);
        assert_eq!(d.get("Fr").width, 260.0);
        assert_eq!(at("Wide", "Ctx"), (0.0, 300.0), "below the floats");
        // clear: both, below every float; an inline text still fits.
        assert_eq!(at("Clr", "Ctx").1, 310.0);
        assert!(d.get("S").width < 100.0);
        // The block does not hold its float; the box after it keeps clear.
        assert_eq!(d.get("Nb").height, 0.0);
        assert_eq!(at("After", "Nb"), (50.0, 0.0));
        assert_eq!(d.get("After").width, 350.0);
        // A flow-root holds its floats (and is narrowed beside the one
        // from the block above that reaches it).
        assert_eq!((d.get("Holds").width, d.get("Holds").height), (350.0, 25.0));
        assert_eq!(at("Hf", "Holds"), (320.0, 0.0));
    });
}

#[test]
fn floats_take_their_sides_from_the_direction() {
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="C"><div id="L"></div><div id="S"></div><p id="P">one two three four five six seven</p></div>
  <div id="D" dir="rtl"><div id="L2"></div><div id="S2"></div><div id="F2"></div></div>
</div></div>
<style>
#Root { width: 300px; }
p { margin: 0; }
#L, #L2 { float: left; width: 50px; height: 20px; }
#S, #S2 { float: inline-start; width: 30px; height: 20px; }
#F2 { display: flow-root; height: 10px; }
</style>"#;
    both(text, &[], |d| {
        for rtl in [false, true] {
            if rtl {
                d.ui.set_direction(og_i18n::Direction::Rtl);
                d.step(InputFrame::default());
                d.step(InputFrame::default());
            }
            let at = |id: &str, of: &str| d.get(id).x - d.get(of).x;
            // `left` stays left; `inline-start` follows the block's direction
            // (the surface's in C, `dir="rtl"` in D).
            let s = if rtl { 270.0 } else { 50.0 };
            assert_eq!((at("L", "C"), at("S", "C")), (0.0, s), "rtl surface {rtl}");
            assert_eq!(
                (at("L2", "D"), at("S2", "D")),
                (0.0, 270.0),
                "rtl surface {rtl}"
            );
            // The flow-root between them, narrowed and mirrored whole.
            assert_eq!(
                (at("F2", "D"), d.get("F2").width),
                (50.0, 220.0),
                "rtl surface {rtl}"
            );
        }
    });
}
