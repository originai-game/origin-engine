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

use crate::html_text::{Doc, both};

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
