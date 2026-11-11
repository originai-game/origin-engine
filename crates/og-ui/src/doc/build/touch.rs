//! On-screen controls in documents (docs/ui.md §3.3): `<joystick
//! action="move">`, `<dpad action="move">` and `<button
//! data-og-touch="jump">`. Each draws as the native control
//! ([`Ui::joystick`], [`Ui::dpad`], a document button) and is an input
//! source of an og-input action: its rect goes to the action map as a
//! [`ControlZone`] ([`Ui::take_control_zones`]), which samples the touches
//! that started on it in the tick, deterministically, as it samples
//! bindings. The document names the action; no handler runs.
use super::*;
use og_input::{ControlZone, ZoneKind};

impl Ui {
    /// A `<joystick>` or `<dpad>`: the native control, sized by its CSS
    /// width (or height: it is round) from `styled`, and its zone.
    pub(super) fn build_touch_control(
        &mut self,
        kind: &str,
        id: &str,
        r: &Resolved,
        styled: &Resolved,
    ) -> crate::ui::Response {
        let size = match styled.size("width").or(styled.size("height")) {
            Some(Size::Fixed(w)) => w,
            _ => 0.0,
        };
        let disabled = r.flag("disabled");
        let dead_zone = r.num("dataDeadZone").unwrap_or(0.0).clamp(0.0, 0.95);
        let (pressed, zone) = if kind == "joystick" {
            let out = self.joystick(
                id,
                crate::widgets::JoystickDesc {
                    size,
                    dead_zone,
                    customize: None,
                    disabled,
                },
            );
            (out.active, ZoneKind::Stick)
        } else {
            let out = self.dpad(
                id,
                crate::widgets::DpadDesc {
                    size,
                    customize: None,
                    disabled,
                },
            );
            (out.up || out.down || out.left || out.right, ZoneKind::Pad)
        };
        let key = self.tree.nodes[self.tree.last_closed].key;
        self.docs
            .note_zone(key, r.str("action"), zone, dead_zone, disabled);
        crate::ui::Response {
            pressed,
            ..Default::default()
        }
    }

    /// The on-screen controls documents laid out, for og-input's action
    /// map ([`og_input::ActionMap::set_zones`]), after [`Ui::end`]: `None`
    /// when nothing moved since the last call. Each zone is the rect its
    /// control was hit-tested at this frame (HUD edit's moves and scales
    /// included); one no longer mounted leaves. Work: a lookup per zone
    /// (HUD edit moves controls without laying anything out); none
    /// without on-screen controls.
    pub fn take_control_zones(&mut self) -> Option<Vec<ControlZone>> {
        if self.docs.host.zones.is_empty() && !self.docs.host.zones_changed {
            return None;
        }
        let live = |key: &u64| self.docs.host.retained.owns(*key);
        let gone: Vec<u64> = self
            .docs
            .host
            .zones
            .keys()
            .copied()
            .filter(|k| !live(k))
            .collect();
        let mut changed = std::mem::take(&mut self.docs.host.zones_changed) || !gone.is_empty();
        for key in gone {
            self.docs.host.zones.remove(&key);
        }
        let rects: Vec<(u64, Option<crate::layout::Rect>)> = self
            .docs
            .host
            .zones
            .keys()
            .map(|&k| (k, self.interaction.hit(k).map(|h| h.rect)))
            .collect();
        for (key, rect) in rects {
            let z = self
                .docs
                .host
                .zones
                .get_mut(&key)
                .expect("a zone just listed");
            let r = rect.unwrap_or_default();
            let at = [r.x, r.y, r.width, r.height];
            if [z.x, z.y, z.width, z.height] != at {
                [z.x, z.y, z.width, z.height] = at;
                changed = true;
            }
        }
        changed.then(|| {
            self.docs
                .host
                .zones
                .values()
                .filter(|z| z.width > 0.0 && z.height > 0.0)
                .cloned()
                .collect()
        })
    }
}

impl crate::ui::Tree {
    /// A touch `<button data-og-touch="jump">`'s zone (any element's).
    pub(super) fn note_touch_button(&mut self, r: &Resolved, docs: &mut crate::ui::Docs) {
        if self.last_closed != NONE && self.last_closed < self.nodes.len() {
            let key = self.nodes[self.last_closed].key;
            let action = r.str("dataOgTouch");
            docs.note_zone(key, action, ZoneKind::Button, 0.0, r.flag("disabled"));
        }
    }
}

impl crate::ui::Docs {
    fn note_zone(
        &mut self,
        key: u64,
        action: Option<&str>,
        kind: ZoneKind,
        dead_zone: f32,
        disabled: bool,
    ) {
        let zones = &mut self.host.zones;
        match action.filter(|a| !a.is_empty() && !disabled) {
            Some(action) => {
                let next = ControlZone {
                    action: action.to_string(),
                    kind,
                    dead_zone,
                    ..zones.get(&key).cloned().unwrap_or_default()
                };
                if zones.get(&key) != Some(&next) {
                    zones.insert(key, next);
                    self.host.zones_changed = true;
                }
            }
            None => {
                if zones.remove(&key).is_some() {
                    self.host.zones_changed = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::Vec2;
    use og_input::{ActionDesc, ActionKind, ActionMap, Pointers};
    use std::collections::BTreeMap;

    const HUD: &str = r#"
<meta name="component" content="hud">
<div id="Root">
  <joystick id="Stick" action="move"/>
  <dpad id="Pad" action="look"/>
  <button id="Jump" data-og-touch="jump">Jump</button>
  <joystick id="Off" action="move" disabled/>
</div>
<style>
  #Root { padding: 0; width: 100%; height: 100%; }
  #Stick { position: absolute; inset-inline-start: 20px; top: 300px; width: 160px; }
  #Pad { position: absolute; inset-inline-start: 220px; top: 300px; }
  #Jump { position: absolute; inset-inline-end: 20px; top: 330px; width: 96px; height: 96px; }
  #Off { position: absolute; inset-inline-start: 20px; top: 20px; }
  @media (pointer: fine) { #Stick, #Pad { display: none; } }
</style>
"#;

    fn pointers(touches: &[(u32, Vec2, Vec2)], pressed: u32) -> Pointers {
        let mut p = Pointers::default();
        for &(id, start, at) in touches {
            p.held |= 1 << id;
            let (pos, st) = match id {
                0 => (&mut p.position0, &mut p.start0),
                1 => (&mut p.position1, &mut p.start1),
                2 => (&mut p.position2, &mut p.start2),
                3 => (&mut p.position3, &mut p.start3),
                _ => (&mut p.position4, &mut p.start4),
            };
            (*pos, *st) = (at, start);
        }
        p.pressed = pressed;
        p
    }

    fn frame(ui: &mut Ui, p: Pointers) -> Option<Vec<og_input::ControlZone>> {
        frame_on(ui, p, true)
    }

    fn frame_on(ui: &mut Ui, p: Pointers, coarse: bool) -> Option<Vec<og_input::ControlZone>> {
        let mut vp = Viewport::new(800.0, 480.0);
        vp.coarse_pointer = coarse;
        ui.begin(
            InputFrame {
                pointers: p,
                ..Default::default()
            },
            vp,
        );
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
        ui.take_control_zones()
    }

    /// The controls' rects reach og-input as zones and drive its actions
    /// from further touches; the knob is drawn again only while a touch
    /// holds it, and an idle frame does no work, zones included.
    ///
    /// Ways it could fail: zones in another space than the pointers' (the
    /// stick reads off-centre); a disabled control still feeding; the
    /// zones handed over every frame; the joystick, built once, never
    /// following a second finger (its knob frozen) or rebuilt every frame.
    #[test]
    fn on_screen_controls_are_input_sources_and_cost_only_while_held() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("hud.svelte", HUD).unwrap());
        ui.mount_document(
            SourceId::new("test", "hud"),
            "hud",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let mut zones = None;
        for _ in 0..4 {
            zones = frame(&mut ui, Pointers::default()).or(zones);
        }
        let zones = zones.expect("zones handed over");
        assert_eq!(zones.len(), 3, "the disabled stick is no source: {zones:?}");
        let stick = zones.iter().find(|z| z.action == "move").unwrap();
        assert_eq!(
            (stick.x, stick.y, stick.width, stick.height),
            (20.0, 300.0, 160.0, 160.0)
        );
        let jump = zones.iter().find(|z| z.action == "jump").unwrap();
        assert_eq!((jump.x + jump.width, jump.y), (780.0, 330.0));
        // Idle: nothing built, nothing handed over.
        assert!(frame(&mut ui, Pointers::default()).is_none());
        let idle = ui.frame().stats;
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");

        // A second finger lands on the stick, right of its centre, while
        // a first holds the jump button.
        let centre = Vec2::new(100.0, 380.0);
        let on_jump = Vec2::new(730.0, 370.0);
        let pushed = centre + Vec2::new(40.0, 0.0);
        let held = [(1, on_jump, on_jump), (2, pushed, pushed)];
        assert!(frame(&mut ui, pointers(&held, 0b110)).is_none());
        let busy = ui.frame().stats;
        assert!(busy.reconciled >= 1, "the stick follows: {busy:?}");
        let mut map = ActionMap::new();
        for (name, kind) in [
            ("move", ActionKind::Axis2),
            ("look", ActionKind::Axis2),
            ("jump", ActionKind::Button),
        ] {
            map.define(ActionDesc {
                name: name.into(),
                kind,
                ..ActionDesc::default()
            })
            .unwrap();
        }
        map.set_surface(og_input::Surface {
            width: 800.0,
            height: 480.0,
            coarse: true,
        });
        map.set_zones(zones);
        map.sample(
            &Default::default(),
            &pointers(&held, 0b110),
            &Default::default(),
            1,
            1.0 / 60.0,
        );
        let m = map.state("move").unwrap();
        // 40 px of an 80 px radius, past a 0.15 dead zone.
        let want = (0.5 - 0.15) / 0.85;
        assert!(
            (m.vector.x - want).abs() < 1e-4 && m.vector.y.abs() < 1e-4,
            "{m:?}"
        );
        assert!(map.state("jump").unwrap().down);
        assert!(!map.state("look").unwrap().down);
        // Still held and not moving: the stick keeps following it.
        frame(&mut ui, pointers(&held, 0));
        assert!(ui.frame().stats.reconciled >= 1);
        // Released: the knob settles (the release frame, and the one after
        // it as for any input key), then nothing.
        for _ in 0..3 {
            frame(&mut ui, Pointers::default());
        }
        let after = ui.frame().stats;
        assert_eq!((after.bindings, after.reconciled), (0, 0), "{after:?}");
        // A mouse (`pointer: fine`) hides the stick and the pad: their
        // zones leave with them.
        let fine = frame_on(&mut ui, Pointers::default(), false).expect("zones changed");
        let left: Vec<&str> = fine.iter().map(|z| z.action.as_str()).collect();
        assert_eq!(left, ["jump"]);
    }
}
