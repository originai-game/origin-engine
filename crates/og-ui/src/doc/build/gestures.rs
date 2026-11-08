//! HTML's drag and drop and Pointer Events on document elements (docs/ui.md
//! §2.1): `draggable` makes an element's [`crate::element::DragSpec`], a
//! handler of `ondragover` or `ondrop` its [`crate::element::DropSpec`],
//! and the input pass ([`super::retained`]'s `input_events`) turns the
//! tick's gestures and pointer into the handlers' events. Only the held
//! element, the one under the pointer and a drop's target are looked at,
//! so the pass costs nothing more as the tree grows.
use super::*;
use crate::element::{DragSpec, DropSpec};

/// A `-og-sheet-detents` value's heights in CSS px (`%` of the viewport's
/// height inside its safe area), in the order written.
pub(crate) fn detents(text: &str, vh: f32) -> Result<Vec<f32>, String> {
    let out: Vec<f32> = text
        .split_whitespace()
        .map(|w| {
            let (n, k) = if let Some(n) = w.strip_suffix('%') {
                (n, vh / 100.0)
            } else {
                (w.strip_suffix("px").unwrap_or(w), 1.0)
            };
            n.parse::<f32>()
                .ok()
                .filter(|v| *v > 0.0)
                .map(|v| v * k)
                .ok_or_else(|| format!("'{w}' is not a detent: <n>% or <n>px"))
        })
        .collect::<Result<_, _>>()?;
    if out.is_empty() {
        return Err("-og-sheet-detents takes one or more heights".into());
    }
    Ok(out)
}

impl Ui {
    /// A modal `<dialog>` with `-og-sheet-detents`, just opened by
    /// `open_dialog`: its backdrop docks the panel to the bottom at full
    /// width, as tall as its tallest detent, and the panel drags along y
    /// onto its detents (the first when it opens) and off the bottom to
    /// close. Answers whether a swipe dismissed it.
    pub(super) fn sheet(&mut self, id: &str, text: &str, open: bool) -> bool {
        let panel = self.parent();
        let scrim = self.nodes[panel].parent;
        let vh = self.viewport.height - self.viewport.safe_area.vertical();
        let Ok(hs) = self::detents(text, vh) else {
            return false;
        };
        let max = hs.iter().copied().fold(0.0, f32::max).min(vh);
        let min = hs.iter().copied().fold(max, f32::min);
        {
            let s = &mut self.nodes[scrim].el;
            s.align = crate::layout::Align::End;
            s.padding.bottom = 0.0;
            s.padding.left = 0.0;
            s.padding.right = 0.0;
            let p = &mut self.nodes[panel].el;
            p.width = Size::grow();
            p.height = Size::Fixed(max);
        }
        let key = self.nodes[panel].key;
        self.nodes[panel].props.drag = Some(DragSpec {
            axis: crate::element::DragAxis::Y,
            bounds: Some(crate::layout::Rect::new(0.0, 0.0, 0.0, max - min)),
            rubber: 0.55,
            snap: hs.iter().map(|h| (0.0, max - h.min(max))).collect(),
            momentum: true,
            dismiss: Some(crate::element::Dismiss::default()),
            ..DragSpec::default()
        });
        let memo = format!("{id}~sheet");
        let was = self.memory(&memo, 0.0) > 0.0;
        self.remember(&memo, f64::from(u8::from(open)));
        if open && !was {
            self.seed_drag(key, og_core::glam::Vec2::new(0.0, max - hs[0].min(max)));
            return false;
        }
        open && was && self.gesture_of(key).dismissed
    }
}

/// The element's drag and drop props from its attributes and handlers.
pub(super) fn props(n: &DocNode, r: &Resolved, p: &mut Props) {
    p.pointer = n.triggers.iter().any(|t| {
        matches!(
            t.when,
            When::PointerDown | When::PointerMove | When::PointerUp | When::Wheel
        )
    });
    if r.flag("draggable") {
        p.drag = Some(DragSpec {
            return_home: true,
            tag: r.str("dataDragType").unwrap_or_default().to_string(),
            ..DragSpec::default()
        });
    }
    if n.triggers
        .iter()
        .any(|t| matches!(t.when, When::DragOver | When::Drop))
    {
        p.drop = Some(DropSpec {
            accept: r
                .str("dataAccept")
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_string)
                .collect(),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::Vec2;
    use serde_json::json;
    use std::collections::BTreeMap;

    const DOC: &str = r#"<meta name="component" content="snap">
<div id="Root">
  <div id="List"><div id="A" class="Row"></div><div id="B" class="Row"></div><div id="C" class="Row"></div><div id="D" class="Row"></div><div id="E" class="Row"></div></div>
  <dialog id="Sheet" modal bind:open={$open}><p>Sheet</p></dialog>
</div>
<style>
#List { height: 150px; width: 200px; overflow-y: auto; scroll-snap-type: y mandatory; }
.Row { height: 100px; width: 200px; scroll-snap-align: start; }
#Sheet { -og-sheet-detents: 40% 90%; }
</style>"#;

    fn tick(ui: &mut Ui, at: Vec2, down: bool, was: bool, wheel: f32) {
        ui.begin(
            InputFrame {
                pointer: at,
                pointer_inside: true,
                down,
                pressed: down && !was,
                released: !down && was,
                wheel: Vec2::new(0.0, wheel),
                ..Default::default()
            },
            Viewport::new(400.0, 600.0),
        );
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
    }

    /// A wheel stopping between rows settles on the nearest row's start,
    /// and a sheet opens at its first detent and closes when swiped down,
    /// writing `open` back. Ways it could fail: a snapping list left where
    /// the wheel stopped, snapping to the far row, a sheet that opens at
    /// its tallest detent, or a swipe that throws the panel off without
    /// closing the dialog.
    #[test]
    fn scroll_snaps_and_sheets_settle_on_their_points() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("snap.svelte", DOC).unwrap());
        let open = SourceId::new("test", "open");
        ui.reactive
            .execute(Command::Signal {
                source: open.clone(),
                initial: json!(false),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "snap"),
            "snap",
            BTreeMap::from([("open".to_string(), open.clone())]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let list = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/List"))
            .map(|e| Vec2::new(e.x + 20.0, e.y + 20.0));
        for _ in 0..3 {
            tick(
                &mut ui,
                list.unwrap_or(Vec2::new(20.0, 20.0)),
                false,
                false,
                0.0,
            );
        }
        let at = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/List"))
            .map(|e| Vec2::new(e.x + 20.0, e.y + 20.0))
            .unwrap();
        tick(&mut ui, at, false, false, 130.0);
        for _ in 0..120 {
            tick(&mut ui, at, false, false, 0.0);
        }
        let s = ui.scroll_state("test:snap/List").unwrap();
        assert!((s.offset_y - 100.0).abs() < 0.5, "{s:?}");

        ui.reactive
            .execute(Command::Set {
                source: open.clone(),
                value: json!(true),
                force: false,
            })
            .unwrap();
        for _ in 0..30 {
            tick(&mut ui, Vec2::new(390.0, 10.0), false, false, 0.0);
        }
        let panel = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("Sheet/Panel"))
            .unwrap_or_else(|| {
                panic!(
                    "{:?}",
                    ui.probe_elements()
                        .iter()
                        .map(|e| e.path.clone())
                        .collect::<Vec<_>>()
                )
            });
        // 90% of 600 tall, opened at 40%: its top at 600 − 240.
        assert!((panel.y - 360.0).abs() < 2.0, "{panel:?}");
        let from = Vec2::new(200.0, panel.y + 20.0);
        tick(&mut ui, from, true, false, 0.0);
        for k in 1..=6 {
            tick(
                &mut ui,
                from + Vec2::new(0.0, 40.0 * k as f32),
                true,
                true,
                0.0,
            );
        }
        tick(&mut ui, from + Vec2::new(0.0, 240.0), false, true, 0.0);
        for _ in 0..30 {
            tick(&mut ui, Vec2::new(390.0, 10.0), false, false, 0.0);
        }
        let v = ui.reactive.execute(Command::Read { source: open }).unwrap();
        assert_eq!(v["value"], json!(false), "a swipe down closes the sheet");
    }
}
