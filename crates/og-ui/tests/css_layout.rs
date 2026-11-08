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
