//! Captures for agents: `capture`'s fit and region, `marks` (numbered
//! boxes with their map back to source) and the run `timeline`.
#![cfg(not(feature = "publish"))]

use crate::support::{Harness, base64_decode, build};
use og_ecs::World;
use og_probe::frame::{Rgba, decode_png};
use og_probe::{Capture, Frame, Pick, SceneView, Seen, Shown, UiElement, UiTree};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

/// A 256×144 frame: grey, with a red square at (40, 20, 32, 32) of the
/// surface's 128×72 CSS px (two frame pixels a CSS pixel).
struct PngCapture;

impl Capture for PngCapture {
    fn tier(&self) -> &str {
        "metal"
    }
    fn target_size(&self) -> (u32, u32) {
        (256, 144)
    }
    fn capture(&mut self, _world: &World, width: u32, height: u32) -> Option<Frame> {
        let (w, h) = if width == 0 {
            (256, 144)
        } else {
            (width, height)
        };
        let mut img = Rgba::filled(w, h, [90, 90, 90, 255]);
        let k = w as i64 / 128;
        img.fill(40 * k, 20 * k, 32 * k, 32 * k, [230, 20, 20, 255]);
        Some(Frame {
            width: w,
            height: h,
            png: img.png(),
        })
    }
}

/// A UI of a button over a label, with a screen stack the test changes.
struct Ui(Rc<RefCell<Vec<String>>>);

impl UiTree for Ui {
    fn elements(&self, _world: &World) -> Vec<UiElement> {
        let el = |id: &str, path: &str, kind: &str, r: [f32; 4]| UiElement {
            id: id.into(),
            path: path.into(),
            kind: kind.into(),
            x: r[0],
            y: r[1],
            width: r[2],
            height: r[3],
            ..UiElement::default()
        };
        let mut button = el("play", "Title/play", "button", [80.0, 40.0, 40.0, 16.0]);
        button.states = vec!["focusable".into()];
        button.text = "Play".into();
        button
            .detail
            .insert("source".into(), json!("ui/title.svelte:12"));
        button
            .detail
            .insert("part".into(), json!("Button.base[primary]"));
        button.detail.insert(
            "rules".into(),
            json!([{ "selector": "button::part(base)", "source": "" }, { "selector": "button.primary::part(base)", "source": "styles/clean/style.css:40" }]),
        );
        let mut label = el("name", "Title/name", "label", [8.0, 4.0, 60.0, 10.0]);
        label.text = "Origin".into();
        vec![
            el("Title", "Title", "layer", [0.0, 0.0, 128.0, 72.0]),
            label,
            button,
        ]
    }
    fn surface(&self, _world: &World) -> Option<(f32, f32, f32)> {
        Some((128.0, 72.0, 2.0))
    }
    fn screens(&self, _world: &World) -> Vec<String> {
        self.0.borrow().clone()
    }
}

/// The player as the one entity in view, over the red square.
struct View;

impl SceneView for View {
    fn seen(&mut self, world: &World, _cover: &[[f32; 4]]) -> Option<Seen> {
        Some(Seen {
            pass: (32, 18),
            camera: json!({}),
            entities: vec![Shown {
                entity: world.find("player")?,
                rect: [40.0, 20.0, 32.0, 32.0],
                drawn: [40.0, 20.0, 32.0, 32.0],
                point: [56.0, 36.0],
                depth: 3.5,
                coverage: 0.1,
                occlusion: 0.0,
                labels: vec!["mesh".into()],
            }],
            views: Vec::new(),
        })
    }
    fn pick(&mut self, _world: &World, _x: f32, _y: f32) -> Option<Pick> {
        None
    }
}

fn harness() -> (Harness, Rc<RefCell<Vec<String>>>) {
    let mut h = build(1, false);
    let screens = Rc::new(RefCell::new(vec!["title".to_string()]));
    h.probe.set_capture(Box::new(PngCapture));
    h.probe.set_ui(Box::new(Ui(screens.clone())));
    h.probe.set_scene_view(Box::new(View));
    (h, screens)
}

fn image(v: &Value) -> Rgba {
    decode_png(&base64_decode(v["png"].as_str().unwrap())).unwrap()
}

fn pixel(img: &Rgba, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

#[test]
fn capture_fits_the_long_side_and_cuts_a_region_at_full_resolution() {
    let (mut h, _) = harness();
    // The default fit leaves a frame under 1280 as rendered.
    let whole = h.result("capture", json!({}));
    assert_eq!(
        (whole["width"].as_u64(), whole["height"].as_u64()),
        (Some(256), Some(144))
    );
    assert_eq!(whole["scale"], json!(2.0));
    // fit 128 halves it; the scale says one PNG pixel a CSS pixel now.
    let fitted = h.result("capture", json!({ "fit": 128 }));
    assert_eq!(
        (fitted["width"].as_u64(), fitted["height"].as_u64()),
        (Some(128), Some(72))
    );
    assert_eq!(fitted["scale"], json!(1.0));
    assert_eq!(pixel(&image(&fitted), 56, 36), [230, 20, 20]);
    // A region of the surface comes at the frame's resolution: 16×8 CSS px
    // at 2 pixels each, its corner at the red square's.
    let r = h.result(
        "capture",
        json!({ "region": { "x": 40, "y": 20, "width": 16, "height": 8 } }),
    );
    assert_eq!(
        (r["width"].as_u64(), r["height"].as_u64()),
        (Some(32), Some(16))
    );
    assert_eq!(
        r["region"],
        json!({ "x": 40.0, "y": 20.0, "width": 16.0, "height": 8.0 })
    );
    let img = image(&r);
    assert_eq!(pixel(&img, 0, 0), [230, 20, 20]);
    // A region off the surface is a refusal; a malformed one an error.
    assert_eq!(
        h.result(
            "capture",
            json!({ "region": { "x": 500, "y": 0, "width": 4, "height": 4 } })
        ),
        Value::Null
    );
    let e = h.call(
        "capture",
        json!({ "region": { "x": 0, "y": 0, "width": 0, "height": 4 } }),
    );
    assert!(e.get("error").is_some(), "{e}");
}

#[test]
fn marks_number_the_ui_then_the_entities_and_point_back_to_source() {
    let (mut h, _) = harness();
    let m = h.result("marks", json!({ "fit": 128 }));
    assert_eq!(
        (m["width"].as_u64(), m["height"].as_u64()),
        (Some(128), Some(72))
    );
    let marks = m["marks"].as_array().unwrap();
    // The bare layer gets no mark; the label (y 4) reads before the button
    // (y 40); the entity comes after the UI.
    let order: Vec<(&str, &str)> = marks
        .iter()
        .map(|k| (k["kind"].as_str().unwrap(), k["path"].as_str().unwrap()))
        .collect();
    assert_eq!(
        order,
        [
            ("ui", "Title/name"),
            ("ui", "Title/play"),
            ("entity", "player")
        ]
    );
    assert_eq!(
        marks
            .iter()
            .map(|k| k["n"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let play = &marks[1];
    assert_eq!(play["source"], json!("ui/title.svelte:12"));
    assert_eq!(play["part"], json!("Button.base[primary]"));
    assert_eq!(play["rule"]["source"], json!("styles/clean/style.css:40"));
    assert_eq!(play["point"], json!({ "x": 100.0, "y": 48.0 }));
    // The rect is in the fitted PNG's pixels: one a CSS pixel at fit 128.
    assert_eq!(
        play["rect"],
        json!({ "x": 80, "y": 40, "width": 40, "height": 16 })
    );
    let player = &marks[2];
    assert_eq!(player["what"], json!("mesh"));
    assert_eq!(player["depth"], json!(3.5));
    assert_eq!(
        player["rect"],
        json!({ "x": 40, "y": 20, "width": 32, "height": 32 })
    );
    // The boxes are drawn: the button's edge is amber, the entity's cyan,
    // and the inside of the entity's box is still the frame.
    let img = image(&m);
    assert_eq!(pixel(&img, 100, 55), [255, 196, 0]);
    assert_eq!(pixel(&img, 41, 50), [0, 214, 255]);
    assert_eq!(pixel(&img, 56, 40), [230, 20, 20]);
    // The full-size frame doubles every rect; max caps and counts the rest.
    let full = h.result("marks", json!({ "fit": 0 }));
    assert_eq!(
        full["marks"][1]["rect"],
        json!({ "x": 160, "y": 80, "width": 80, "height": 32 })
    );
    let capped = h.result("marks", json!({ "max": 1, "entities": false }));
    assert_eq!(capped["marks"].as_array().unwrap().len(), 1);
    assert_eq!(capped["omitted"], json!(1));
}

#[test]
fn the_timeline_lists_inputs_steps_screens_and_expectations_by_tick() {
    let (mut h, screens) = harness();
    h.result(
        "input",
        json!({ "event": { "kind": "key", "key": "ArrowRight", "action": "down" } }),
    );
    h.probe.schedule(&h.world, "0.05:pause;0.1:resume").unwrap();
    for i in 0..10 {
        if i == 4 {
            screens.borrow_mut().push("shop".into());
        }
        h.probe.step(&mut h.world).unwrap();
    }
    h.result(
        "expect",
        json!({ "what": "entity:player:within:-100,-100,-100,100,100,100" }),
    );
    let t = h.result("timeline", json!({}));
    let events: Vec<(&str, &str)> = t["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["kind"].as_str().unwrap(), e["what"].as_str().unwrap()))
        .collect();
    assert_eq!(events[0], ("input", "key down ArrowRight"), "{t}");
    assert!(events.contains(&("step", "pause")), "{t}");
    assert!(events.contains(&("screen", "title > shop")), "{t}");
    assert!(
        events
            .iter()
            .any(|(k, w)| *k == "expect" && w.starts_with("pass entity:player")),
        "{t}"
    );
    // Ticks never go back; the screen change is seen after the tick it
    // happened in, with what it showed.
    let ticks: Vec<u64> = t["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["tick"].as_u64().unwrap())
        .collect();
    assert!(ticks.windows(2).all(|w| w[0] <= w[1]), "{ticks:?}");
    let screen = t["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["what"] == "title > shop")
        .unwrap();
    assert_eq!(screen["detail"]["shown"], json!(["shop"]));
    assert_eq!(t["screens"], json!(["title", "shop"]));
    // A read takes the events: the next one starts empty.
    assert_eq!(h.result("timeline", json!({}))["events"], json!([]));
}
