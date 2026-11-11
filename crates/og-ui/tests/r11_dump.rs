//! Temporary identity harness for R11 (not committed).
use crate::html_text::Doc;
use crate::support;
use og_core::glam::Vec2;
use og_ui::*;
use serde_json::json;
use std::fmt::Write;

const DOC: &str = r#"<meta name="component" content="t">
<column id="Root" gap="8" padding="12">
  <button id="Plain" onclick={hit}>Plain</button>
  <button id="Primary" variant="primary" onclick={go}>Go</button>
  <button id="Off" disabled onclick={never}>Off</button>
  <button id="Danger" variant="danger" autofocus>Danger</button>
  <button id="Content" onclick={pack}><text id="Word">Buy</text><text id="Price">12</text></button>
  <button id="Tip" title="A tip" onclick={tip}>Tip</button>
  <button id="Hold" data-hold="100" onclick={holdc}>Hold</button>
  <button id="Styled" class="s" onclick={styled}>Styled</button>
  <button id="Quiet" variant="quiet" width="300">Quiet wide</button>
  <button id="Pop" popovertarget="Menu">Pop</button>
  <column id="Menu" popover><text id="In">inside</text><button id="Inner" onclick={inner}>Inner</button></column>
  <label><input type="checkbox" switch id="Check" bind:checked={$on}> Check</label>
  <label><input type="checkbox" id="Box" bind:checked={$on}> Box</label>
  <input type="range" id="Vol" min="0" max="10" bind:value={$vol} />
  <progress id="Prog" value="0.4" />
  <meter id="Hp" value="40" min="0" max="100" low="25" high="75" optimum="90" />
  <input type="text" id="Name" bind:value={$name}>
  <select id="Pick" bind:value={$pick}><option value="a">A</option><option value="b">B</option></select>
  <details id="More"><summary>More</summary><text id="Body">body</text><button id="Deep" onclick={deep}>Deep</button></details>
  {#if $on}<button id="Late" onclick={late}>Late</button>{/if}
</column>
<style>
.s { width: 200px; }
.s:hover { width: 220px; background: red; }
.s:active { opacity: 0.5; }
.s:focus-visible { outline: 2px solid blue; }
#Quiet { transition: opacity 200ms; }
#Quiet:hover { opacity: 0.6; }
</style>
"#;

fn pack() -> style_pack::StylePack {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/styles/fantasy"
    );
    let mut files = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "css") {
            files.push((
                format!(
                    "styles/fantasy/{}",
                    p.file_name().unwrap().to_string_lossy()
                ),
                std::fs::read_to_string(&p).unwrap(),
            ));
        }
    }
    files.sort();
    style_pack::StylePack::parse("styles/fantasy", files).unwrap()
}

fn centre(d: &Doc, id: &str) -> Option<Vec2> {
    d.el(id)
        .map(|e| Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0))
}

fn dump(out: &mut String, d: &mut Doc, label: &str) {
    let f = d.ui.frame();
    writeln!(out, "== {label} items={}", f.items.len()).unwrap();
    for it in &f.items {
        writeln!(out, "{it:?}").unwrap();
    }
    for w in &f.warnings {
        writeln!(out, "W {w}").unwrap();
    }
    for e in d.ui.probe_elements() {
        writeln!(out, "E {e:?}").unwrap();
    }
    for e in d.events.drain(..) {
        writeln!(out, "V {e:?}").unwrap();
    }
    writeln!(out, "F {:?}", d.ui.take_feedback()).unwrap();
    writeln!(out, "A {:?}", d.ui.a11y().nodes).unwrap();
    for name in ["on", "vol", "name", "pick"] {
        writeln!(out, "S {name}={}", d.read(name)).unwrap();
    }
    writeln!(out, "focus={}", d.ui.focus_path(0)).unwrap();
}

fn ptr(p: Vec2, down: bool, pressed: bool, released: bool) -> InputFrame {
    InputFrame {
        pointer: p,
        pointer_inside: true,
        down,
        pressed,
        released,
        ..Default::default()
    }
}

fn key(k: UiKey, code: &'static str) -> InputFrame {
    InputFrame {
        keys: vec![k],
        keys_pressed: vec![code],
        keys_held: vec![code],
        ..Default::default()
    }
}

fn run(out: &mut String, mounted: bool, dpr: f32, rtl: bool, coarse: bool, fantasy: bool) {
    let stores = [
        ("on", json!(false)),
        ("vol", json!(3.0)),
        ("name", json!("ab")),
        ("pick", json!("a")),
    ];
    let mut d = Doc::new(DOC, &stores, mounted);
    d.viewport.dpr = dpr;
    d.viewport.coarse_pointer = coarse;
    if rtl {
        d.ui.set_direction(og_i18n::Direction::Rtl);
    }
    if fantasy {
        d.ui.set_style_pack(&pack()).unwrap();
    }
    let tag = format!("m{mounted} d{dpr} r{rtl} c{coarse} f{fantasy}");
    let mut n = 0;
    let mut step = |d: &mut Doc, out: &mut String, i: InputFrame, what: &str| {
        d.step(i);
        n += 1;
        dump(out, d, &format!("{tag} #{n} {what}"));
    };
    for _ in 0..4 {
        step(&mut d, out, InputFrame::default(), "rest");
    }
    let ids = [
        "Plain", "Primary", "Off", "Danger", "Content", "Tip", "Hold", "Styled", "Quiet", "Pop",
        "Check", "Box", "Vol", "Name", "Pick", "More",
    ];
    for id in ids {
        let Some(p) = centre(&d, id) else { continue };
        for k in 0..3 {
            step(
                &mut d,
                out,
                ptr(p + Vec2::new(k as f32, 0.0), false, false, false),
                &format!("hover {id}"),
            );
        }
        step(
            &mut d,
            out,
            ptr(p, true, true, false),
            &format!("press {id}"),
        );
        for k in 0..8 {
            step(
                &mut d,
                out,
                ptr(p + Vec2::new(k as f32 * 3.0, 0.0), true, false, false),
                &format!("drag {id}"),
            );
        }
        step(
            &mut d,
            out,
            ptr(p, false, false, true),
            &format!("release {id}"),
        );
        for _ in 0..3 {
            step(
                &mut d,
                out,
                ptr(p, false, false, false),
                &format!("after {id}"),
            );
        }
        if id == "Tip" {
            for _ in 0..25 {
                step(&mut d, out, ptr(p, false, false, false), "tip wait");
            }
        }
        step(
            &mut d,
            out,
            ptr(Vec2::new(790.0, 890.0), false, false, false),
            &format!("leave {id}"),
        );
        for _ in 0..2 {
            step(&mut d, out, InputFrame::default(), "settle");
        }
        if id == "Pop" || id == "Pick" {
            step(&mut d, out, key(UiKey::Escape, "Escape"), "escape");
            step(&mut d, out, InputFrame::default(), "settle");
        }
    }
    for k in 0..18 {
        step(&mut d, out, key(UiKey::Tab, "Tab"), &format!("tab {k}"));
        step(&mut d, out, InputFrame::default(), "tab rest");
        if k % 2 == 0 {
            step(&mut d, out, key(UiKey::Enter, "Enter"), "enter");
        } else {
            step(&mut d, out, key(UiKey::Space, "Space"), "space");
        }
        step(&mut d, out, InputFrame::default(), "key rest");
        step(&mut d, out, key(UiKey::Right, "ArrowRight"), "right");
        step(&mut d, out, key(UiKey::Escape, "Escape"), "escape");
    }
    for k in 0..6 {
        let mut i = InputFrame::default();
        i.gamepad.pressed = if k % 2 == 0 {
            GamepadFrame::DOWN
        } else {
            GamepadFrame::A
        };
        i.gamepad.held = i.gamepad.pressed;
        step(&mut d, out, i, &format!("pad {k}"));
        step(&mut d, out, InputFrame::default(), "pad rest");
    }
    let path = d.get("Plain").path;
    for s in [
        Some("hover"),
        Some("pressed"),
        Some("focused"),
        Some("disabled"),
        None,
    ] {
        d.ui.force_state(&path, "interaction", s);
        step(&mut d, out, InputFrame::default(), &format!("force {s:?}"));
        step(&mut d, out, InputFrame::default(), "force rest");
    }
    if let Some(p) = centre(&d, "Primary") {
        step(&mut d, out, ptr(p, false, false, false), "pre-save hover");
        let snap = d.ui.save_state();
        writeln!(out, "SNAP {snap:?}").unwrap();
        step(&mut d, out, ptr(p, true, true, false), "press");
        d.ui.restore_state(&snap).unwrap();
        step(&mut d, out, ptr(p, false, false, false), "restored");
        step(&mut d, out, ptr(p, true, true, false), "press");
        step(&mut d, out, ptr(p, false, false, true), "release");
        step(&mut d, out, InputFrame::default(), "end");
        let snap = d.ui.save_state();
        writeln!(out, "SNAP {snap:?}").unwrap();
    }
}

/// The immediate builder's buttons under the same script.
fn immediate(out: &mut String, dpr: f32, rtl: bool, coarse: bool, fantasy: bool) {
    let mut ui = support::ui_with_font(support::test_font_metrics());
    if rtl {
        ui.set_direction(og_i18n::Direction::Rtl);
    }
    if fantasy {
        ui.set_style_pack(&pack()).unwrap();
    }
    let mut vp = Viewport::new(800.0, 900.0);
    vp.dpr = dpr;
    vp.coarse_pointer = coarse;
    let mut n = 0;
    let mut tick = |ui: &mut Ui, out: &mut String, i: InputFrame| {
        let mut clicks = Vec::new();
        ui.begin(i, vp);
        ui.open(
            "Col",
            Element {
                gap: 8.0,
                direction: Direction::Column,
                ..Element::default()
            },
        );
        let variants = [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Quiet,
            ButtonVariant::Danger,
            ButtonVariant::Ghost,
        ];
        for (k, v) in variants.into_iter().enumerate() {
            let r = ui.button(
                format!("B{k}").as_str(),
                &format!("Button {k}"),
                ButtonDesc {
                    variant: v,
                    disabled: k == 2,
                    square: k == 4,
                    autofocus: k == 3,
                    ..Default::default()
                },
            );
            clicks.push((k, r));
        }
        let r = ui.button(
            "Small",
            "Small",
            ButtonDesc {
                size: ControlSize::Small,
                width: Some(Size::Fixed(40.0)),
                ..Default::default()
            },
        );
        clicks.push((9, r));
        ui.close();
        ui.end();
        n += 1;
        writeln!(out, "== imm d{dpr} r{rtl} c{coarse} f{fantasy} #{n}").unwrap();
        for it in &ui.frame().items {
            writeln!(out, "{it:?}").unwrap();
        }
        for e in ui.probe_elements() {
            writeln!(out, "E {e:?}").unwrap();
        }
        writeln!(out, "R {clicks:?}").unwrap();
        writeln!(out, "F {:?}", ui.take_feedback()).unwrap();
    };
    for _ in 0..3 {
        tick(&mut ui, out, InputFrame::default());
    }
    for id in ["B0", "B1", "B2", "B3", "B4", "Small"] {
        let Some(e) = ui.probe_elements().into_iter().find(|e| e.id == id) else {
            continue;
        };
        let p = Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0);
        for _ in 0..3 {
            tick(&mut ui, out, ptr(p, false, false, false));
        }
        tick(&mut ui, out, ptr(p, true, true, false));
        for _ in 0..4 {
            tick(&mut ui, out, ptr(p, true, false, false));
        }
        tick(&mut ui, out, ptr(p, false, false, true));
        for _ in 0..12 {
            tick(&mut ui, out, InputFrame::default());
        }
    }
    for _ in 0..8 {
        tick(&mut ui, out, key(UiKey::Tab, "Tab"));
        tick(&mut ui, out, key(UiKey::Enter, "Enter"));
        tick(&mut ui, out, InputFrame::default());
    }
}

#[test]
fn r11_dump() {
    let Ok(path) = std::env::var("R11_DUMP") else {
        return;
    };
    let case: usize = std::env::var("R11_CASE").unwrap().parse().unwrap();
    let mut out = String::new();
    match case {
        0 => run(&mut out, true, 1.0, false, false, false),
        1 => run(&mut out, true, 1.5, true, false, false),
        2 => run(&mut out, true, 1.0, false, true, true),
        3 => run(&mut out, false, 1.0, false, false, false),
        4 => run(&mut out, false, 1.5, true, true, true),
        _ => {
            for (dpr, rtl, coarse, fantasy) in [
                (1.0, false, false, false),
                (1.5, true, false, false),
                (1.0, false, true, true),
            ] {
                immediate(&mut out, dpr, rtl, coarse, fantasy);
            }
        }
    }
    std::fs::write(format!("{path}.{case}"), out).unwrap();
}
