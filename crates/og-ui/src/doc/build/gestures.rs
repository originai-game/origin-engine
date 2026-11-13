//! HTML's drag and drop and Pointer Events on document elements (docs/ui.md
//! §2.1): `draggable` makes an element's [`crate::element::DragSpec`], a
//! handler of `ondragover` or `ondrop` its [`crate::element::DropSpec`],
//! and the input pass ([`super::retained`]'s `input_events`) turns the
//! tick's gestures and pointer into the handlers' events. Only the held
//! element, the one under the pointer and a drop's target are looked at,
//! so the pass costs nothing more as the tree grows.
use super::*;
use crate::element::DragSpec;

/// A `data-detents` value's depths in CSS px (`%` of the viewport's
/// height inside its safe area, or its width for a side sheet), in the
/// order written.
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
        return Err("data-detents takes one or more heights".into());
    }
    Ok(out)
}

/// The edge a sheet docks to (`data-edge`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SheetEdge {
    #[default]
    BlockEnd,
    InlineStart,
    InlineEnd,
}

/// A `data-edge` value.
pub(crate) fn sheet_edge(text: &str) -> Result<SheetEdge, String> {
    match text.trim() {
        "block-end" => Ok(SheetEdge::BlockEnd),
        "inline-start" => Ok(SheetEdge::InlineStart),
        "inline-end" => Ok(SheetEdge::InlineEnd),
        w => Err(format!(
            "data-edge: '{w}' is block-end, inline-start or inline-end"
        )),
    }
}

impl Ui {
    /// A modal `<dialog>` with `data-detents`, just opened by
    /// `open_dialog`: its backdrop docks the panel to `edge` (the bottom at
    /// full width, or the inline start or end at full height, mirrored right
    /// to left), as deep as its deepest detent, and the panel drags across
    /// that edge onto its detents (the first when it opens) and off it to
    /// close. Answers whether a swipe dismissed it.
    pub(crate) fn sheet(&mut self, id: &str, text: &str, edge: SheetEdge, open: bool) -> bool {
        let panel = self.tree.parent();
        let scrim = self.tree.nodes()[panel].parent;
        let side = edge != SheetEdge::BlockEnd;
        let vh = self.env.viewport().height - self.env.viewport().safe_area.vertical();
        let vw = self.env.viewport().width - self.env.viewport().safe_area.horizontal();
        let room = if side { vw } else { vh };
        let Ok(hs) = self::detents(text, room) else {
            return false;
        };
        let max = hs.iter().copied().fold(0.0, f32::max).min(room);
        let min = hs.iter().copied().fold(max, f32::min);
        // The inline end is the right left to right; the layout mirrors
        // the dock, the drag's offset is the screen's.
        let rtl = self.direction() == og_i18n::Direction::Rtl;
        let sign = match edge {
            SheetEdge::BlockEnd => 1.0,
            SheetEdge::InlineEnd => {
                if rtl {
                    -1.0
                } else {
                    1.0
                }
            }
            SheetEdge::InlineStart => {
                if rtl {
                    1.0
                } else {
                    -1.0
                }
            }
        };
        {
            let s = self.tree.el_mut(scrim);
            s.padding.bottom = 0.0;
            s.padding.left = 0.0;
            s.padding.right = 0.0;
            if side {
                s.padding.top = 0.0;
                s.align = crate::layout::Align::Start;
                s.cross = if edge == SheetEdge::InlineEnd {
                    crate::layout::Cross::End
                } else {
                    crate::layout::Cross::Start
                };
            } else {
                s.align = crate::layout::Align::End;
            }
            let p = self.tree.el_mut(panel);
            if side {
                p.width = Size::Fixed(max);
                p.height = Size::grow();
            } else {
                p.width = Size::grow();
                p.height = Size::Fixed(max);
            }
        }
        let key = self.tree.nodes()[panel].key;
        let at = |h: f32| sign * (max - h.min(max));
        let travel = max - min;
        let lo = if sign < 0.0 { -travel } else { 0.0 };
        self.tree.props_mut(panel).drag = Some(DragSpec {
            axis: if side {
                crate::element::DragAxis::X
            } else {
                crate::element::DragAxis::Y
            },
            bounds: Some(if side {
                crate::layout::Rect::new(lo, 0.0, travel, 0.0)
            } else {
                crate::layout::Rect::new(0.0, 0.0, 0.0, travel)
            }),
            rubber: 0.55,
            snap: hs
                .iter()
                .map(|&h| if side { (at(h), 0.0) } else { (0.0, at(h)) })
                .collect(),
            momentum: true,
            dismiss: Some(crate::element::Dismiss::default()),
            ..DragSpec::default()
        });
        let memo = format!("{id}~sheet");
        let was = self.memory(&memo, 0.0) > 0.0;
        self.remember(&memo, f64::from(u8::from(open)));
        if open && !was {
            let first = at(hs[0]);
            let start = if side {
                og_core::glam::Vec2::new(first, 0.0)
            } else {
                og_core::glam::Vec2::new(0.0, first)
            };
            self.interaction.seed_drag(key, start);
            return false;
        }
        open && was && self.gesture_of(key).dismissed
    }
}

/// The first two touches held (og.Pointers) that went down where `inside`
/// holds (screen CSS px): their centre and spread, the way a pinch reads
/// them.
pub(crate) fn two_touches(
    input: &crate::InputFrame,
    inside: impl Fn(f32, f32) -> bool,
) -> Option<(og_core::glam::Vec2, f32)> {
    let pts = &input.pointers;
    let mut held = (0..og_input::MAX_POINTERS as u32)
        .filter(|&id| pts.down(id))
        .filter(|&id| {
            let s = pts.start(id);
            inside(s.x, s.y)
        });
    let (a, b) = (pts.position(held.next()?), pts.position(held.next()?));
    Some(((a + b) / 2.0, a.distance(b).max(1.0)))
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
  <dialog id="Sheet" modal data-detents="40% 90%" bind:open={$open}><p>Sheet</p></dialog>
</div>
<style>
#List { height: 150px; width: 200px; overflow-y: auto; scroll-snap-type: y mandatory; }
.Row { height: 100px; width: 200px; scroll-snap-align: start; }
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
        ui.docs
            .state_mut()
            .memory
            .reactive
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

        ui.docs
            .state_mut()
            .memory
            .reactive
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
        let v = ui
            .docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Read { source: open })
            .unwrap();
        assert_eq!(v["value"], json!(false), "a swipe down closes the sheet");
    }

    /// `data-edge="inline-end"` docks the sheet to the right left to
    /// right and to the left right to left, full height, opened at its
    /// first detent (a width), and a swipe off that edge closes it.
    ///
    /// Ways it could fail: an end sheet on the right in Arabic too, a drag
    /// that moves it the wrong way, or one that throws it off without
    /// closing.
    #[test]
    fn side_sheets_dock_to_their_inline_edge_and_mirror() {
        let doc = r#"<meta name="component" content="side">
<div id="Root"><dialog id="Sheet" modal data-detents="50% 80%" data-edge="inline-end" bind:open={$open}><p>Side</p></dialog></div>"#;
        for rtl in [false, true] {
            let mut ui = Ui::new(UiDesc::default());
            if rtl {
                ui.set_direction(og_i18n::Direction::Rtl);
            }
            ui.define_document(UiDocument::parse("side.svelte", doc).unwrap());
            let open = SourceId::new("test", "open");
            ui.docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Signal {
                    source: open.clone(),
                    initial: json!(true),
                    readable: false,
                })
                .unwrap();
            ui.mount_document(
                SourceId::new("test", "side"),
                "side",
                BTreeMap::from([("open".to_string(), open.clone())]),
                BTreeMap::new(),
                &crate::bind::NoEnv,
            )
            .unwrap();
            let idle = Vec2::new(200.0, 590.0);
            for _ in 0..30 {
                tick(&mut ui, idle, false, false, 0.0);
            }
            let panel = ui
                .probe_elements()
                .into_iter()
                .find(|e| e.path.ends_with("Sheet/Panel"))
                .unwrap();
            // 80% of 400 wide, opened at 50%: 200 px of it on screen.
            let shown = if rtl {
                panel.x + panel.width
            } else {
                400.0 - panel.x
            };
            assert!((shown - 200.0).abs() < 2.0, "rtl {rtl}: {panel:?}");
            assert!(panel.height > 590.0, "full height: {panel:?}");
            let out = if rtl { -40.0 } else { 40.0 };
            let from = Vec2::new(if rtl { 100.0 } else { 300.0 }, 300.0);
            tick(&mut ui, from, true, false, 0.0);
            for k in 1..=6 {
                tick(
                    &mut ui,
                    from + Vec2::new(out * k as f32, 0.0),
                    true,
                    true,
                    0.0,
                );
            }
            tick(&mut ui, from + Vec2::new(out * 6.0, 0.0), false, true, 0.0);
            for _ in 0..30 {
                tick(&mut ui, idle, false, false, 0.0);
            }
            let v = ui
                .docs
                .state_mut()
                .memory
                .reactive
                .execute(Command::Read { source: open })
                .unwrap();
            assert_eq!(
                v["value"],
                json!(false),
                "rtl {rtl}: a swipe off its edge closes it"
            );
        }
    }
}
