//! The player's HUD layout (docs/ui.md §2.1): elements whose props say
//! [`Customize`] (a HUD group, a touch stick or button) can be moved,
//! scaled and faded by the player in an edit mode, each placement kept per
//! player under the element's key. The game saves the layout in its
//! settings ([`Ui::hud_layout`]) and gives it back on the next run
//! ([`Ui::set_hud_layout`]); [`Ui::hud_layout_editor`] is the edit mode's
//! panel.

use crate::element::Customize;
use crate::input::InputFrame;
use crate::layout::{Rect, Size};
use crate::style::Srgba;
use crate::ui::{ButtonDesc, ButtonVariant, LabelDesc, PanelDesc, Ui};
use crate::{DrawCmd, Id};
use og_core::glam::Vec2;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// Where the player put an element: moved by `x`, `y` (units) from its
/// laid-out place, scaled about its centre and faded. The zero value is
/// not the default: [`HudPlacement::default`] leaves it as laid out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HudPlacement {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub opacity: f32,
}

impl Default for HudPlacement {
    fn default() -> Self {
        HudPlacement {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            opacity: 1.0,
        }
    }
}

/// The edit mode as the last frame left it ([`Ui::hud_edit`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudEditInfo {
    /// The edit mode is on.
    pub on: bool,
    /// The player whose layout this screen shows and edits.
    pub player: u32,
    /// The element being edited (its key; empty: none).
    pub selected: String,
    /// Its placement.
    pub placement: HudPlacement,
    /// The customisable elements on screen, in drawing order.
    pub keys: Vec<String>,
    /// The player changed the layout this tick (a move ended, a step, a
    /// reset): the time to save it.
    pub changed: bool,
}

/// What [`Ui::hud_layout_editor`] shows. Empty labels take English ones.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudEditorDesc {
    pub title: String,
    pub size_label: String,
    pub opacity_label: String,
    pub next_label: String,
    pub reset_label: String,
    pub done_label: String,
}

/// What the editor's buttons did this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HudEditorResponse {
    /// Done was pressed: the edit mode is off from the next tick.
    pub done: bool,
    /// The layout changed this tick.
    pub changed: bool,
}

/// A customisable element of the last frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HudTarget {
    pub key: String,
    /// Its laid-out rectangle on the screen, before its placement.
    pub base: Rect,
    /// As drawn, with its placement.
    pub shown: Rect,
    pub spec: Customize,
}

/// The layouts and the edit mode between ticks (in [`Ui::save_state`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HudState {
    /// Each player's placements by key.
    pub layouts: [BTreeMap<String, HudPlacement>; 4],
    pub player: u8,
    pub edit: bool,
    pub selected: String,
    /// A move in progress: where the press started and the placement then,
    /// and two touches' spread when a pinch began (0: none).
    pub drag: Option<(Vec2, HudPlacement, f32)>,
    /// The last frame's customisable elements, in drawing order.
    pub targets: Vec<HudTarget>,
    /// The editor panel's rectangle last frame (presses there edit nothing).
    pub editor: Rect,
    /// The tick the player last changed the layout.
    pub changed: u64,
}

impl HudPlacement {
    fn to_json(self) -> Value {
        json!({ "x": self.x, "y": self.y, "scale": self.scale, "opacity": self.opacity })
    }

    fn from_json(v: &Value) -> Option<HudPlacement> {
        let f = |k: &str, d: f32| match &v[k] {
            Value::Null => Some(d),
            n => n.as_f64().map(|n| n as f32).filter(|n| n.is_finite()),
        };
        Some(HudPlacement {
            x: f("x", 0.0)?,
            y: f("y", 0.0)?,
            scale: f("scale", 1.0)?.max(0.01),
            opacity: f("opacity", 1.0)?.clamp(0.0, 1.0),
        })
    }

    /// Within the element's limits, and its centre on the surface.
    fn limited(mut self, t: &HudTarget, w: f32, h: f32) -> HudPlacement {
        let s = &t.spec;
        let (lo, hi) = s.scale_range();
        self.scale = if s.lock_scale {
            1.0
        } else {
            self.scale.clamp(lo, hi)
        };
        self.opacity = if s.lock_opacity {
            1.0
        } else {
            self.opacity.clamp(s.opacity_floor(), 1.0)
        };
        if s.lock_move {
            self.x = 0.0;
            self.y = 0.0;
        } else {
            let (cx, cy) = t.base.center();
            let along = |v: f32, c: f32, half: f32, room: f32| {
                let (a, b) = (half - c, room - half - c);
                if a > b { (a + b) / 2.0 } else { v.clamp(a, b) }
            };
            self.x = along(self.x, cx, t.base.width * self.scale / 2.0, w);
            self.y = along(self.y, cy, t.base.height * self.scale / 2.0, h);
        }
        self
    }
}

const SCALE_STEP: f32 = 1.1;
const OPACITY_STEP: f32 = 0.1;

impl Ui {
    /// Turns the HUD edit mode on or off (kept until changed): customisable
    /// elements stop answering their own presses; a press on one selects it
    /// and a drag moves it, two touches or the wheel scale it, and with no
    /// virtual cursor pad 0's left stick moves the selected one.
    pub fn set_hud_edit(&mut self, on: bool) {
        let hud = &mut self.interaction.state.hud;
        hud.edit = on;
        hud.drag = None;
        if !on {
            hud.selected.clear();
        }
    }

    /// Whose layout this screen shows and edits (0 to 3; kept until changed).
    pub fn set_hud_player(&mut self, player: u32) {
        self.interaction.state.hud.player = player.min(3) as u8;
    }

    /// The edit mode as the last frame left it.
    pub fn hud_edit(&self) -> HudEditInfo {
        let hud = &self.interaction.state.hud;
        let player = usize::from(hud.player);
        HudEditInfo {
            on: hud.edit,
            player: u32::from(hud.player),
            selected: hud.selected.clone(),
            placement: hud.layouts[player]
                .get(&hud.selected)
                .copied()
                .unwrap_or_default(),
            keys: hud.targets.iter().map(|t| t.key.clone()).collect(),
            changed: hud.changed != 0 && hud.changed == self.interaction.state.tick,
        }
    }

    /// A player's layout as JSON, for the game's settings: each moved,
    /// scaled or faded element's key with its `x`, `y`, `scale` and
    /// `opacity`.
    pub fn hud_layout(&self, player: u32) -> String {
        let l = &self.interaction.state.hud.layouts[player.min(3) as usize];
        let m: Map<String, Value> = l.iter().map(|(k, p)| (k.clone(), p.to_json())).collect();
        Value::Object(m).to_string()
    }

    /// Gives a player's layout back ([`Ui::hud_layout`]'s JSON; `{}` resets
    /// it). Refused, leaving the layout as it was, when it does not read.
    pub fn set_hud_layout(&mut self, player: u32, layout: &str) -> Result<(), String> {
        let v: Value = serde_json::from_str(layout).map_err(|e| format!("hud layout: {e}"))?;
        let Value::Object(m) = v else {
            return Err("hud layout: expected an object of placements by key".into());
        };
        let mut out = BTreeMap::new();
        for (k, p) in &m {
            let p = HudPlacement::from_json(p)
                .ok_or_else(|| format!("hud layout: '{k}' is not a placement"))?;
            out.insert(k.clone(), p);
        }
        self.interaction.state.hud.layouts[player.min(3) as usize] = out;
        Ok(())
    }

    /// Selects an element to edit by its key (empty: none).
    pub fn hud_select(&mut self, key: &str) {
        self.interaction.state.hud.selected = key.to_owned();
    }

    /// Steps the selected element's size and opacity (each -1, 0 or +1
    /// step: a tenth larger or smaller, a tenth more or less opaque).
    pub fn hud_step(&mut self, scale: i32, opacity: i32) {
        let hud = &self.interaction.state.hud;
        let Some(t) = hud.targets.iter().find(|t| t.key == hud.selected).cloned() else {
            return;
        };
        let mut p = self.interaction.placement(&t.key);
        p.scale *= SCALE_STEP.powi(scale.signum());
        p.opacity += OPACITY_STEP * opacity.signum() as f32;
        self.interaction.place(&t, p, &self.env, &self.tree);
    }

    /// Puts the selected element (empty `key`), or every one (`"*"`), back
    /// where it was laid out.
    pub fn hud_reset(&mut self, key: &str) {
        let now = self.interaction.hud_now(&self.tree);
        let hud = &mut self.interaction.state.hud;
        let player = usize::from(hud.player);
        if key == "*" {
            hud.layouts[player].clear();
        } else {
            let k = if key.is_empty() {
                hud.selected.clone()
            } else {
                key.to_owned()
            };
            hud.layouts[player].remove(&k);
        }
        hud.changed = now;
    }

    /// The edit mode's input: a press on a customisable element selects it
    /// and takes the press (the element answers nothing), a drag moves it,
    /// a second touch pinches it, the wheel over it scales it, and pad 0's
    /// stick moves the selected one. Run on the input in units before the
    /// hit test; presses on the editor panel pass.
    pub(crate) fn hud_edit_input(&mut self, input: &mut InputFrame) {
        if !self.interaction.state.hud.edit {
            return;
        }
        let p = input.pointer;
        let over_editor = self.interaction.state.hud.editor.contains(p.x, p.y);
        let mut took = false;
        if input.pressed && input.pointer_inside && !over_editor {
            if let Some(t) = self.interaction.hud_target_at(p) {
                self.interaction.state.hud.selected = t.key.clone();
                self.interaction.state.hud.drag =
                    Some((p, self.interaction.placement(&t.key), 0.0));
                self.sound(crate::style_file::UiSound::Press);
                took = true;
            }
        } else if let Some((start, from, spread0)) = self.interaction.state.hud.drag {
            let sel = self.interaction.state.hud.selected.clone();
            let t = self
                .interaction
                .state
                .hud
                .targets
                .iter()
                .find(|t| t.key == sel)
                .cloned();
            if input.down
                && let Some(t) = t
            {
                // Two touches: the second one's spread scales from where it began.
                let ps = &input.pointers;
                let spread = if ps.held & 0b11 == 0b11 {
                    (ps.position1 - ps.position0).length()
                } else {
                    0.0
                };
                let spread0 = if spread > 0.0 && spread0 <= 0.0 {
                    self.interaction.state.hud.drag = Some((start, from, spread));
                    spread
                } else {
                    spread0
                };
                let mut to = from;
                to.x += p.x - start.x;
                to.y += p.y - start.y;
                if spread > 0.0 && spread0 > 0.0 {
                    to.scale *= spread / spread0;
                }
                let (w, h) = (self.env.viewport.width, self.env.viewport.height);
                let hud = &mut self.interaction.state.hud;
                let to = to.limited(&t, w, h);
                let l = &mut hud.layouts[usize::from(hud.player)];
                l.insert(t.key.clone(), to);
            } else {
                self.interaction.state.hud.drag = None;
                self.interaction.state.hud.changed = self.interaction.hud_now(&self.tree);
                let hud = &mut self.interaction.state.hud;
                hud.layouts[usize::from(hud.player)].retain(|_, p| *p != HudPlacement::default());
            }
            took = true;
        }
        if input.wheel.y != 0.0
            && input.pointer_inside
            && !over_editor
            && let Some(t) = self.interaction.hud_target_at(p)
        {
            self.interaction.state.hud.selected = t.key.clone();
            let mut to = self.interaction.placement(&t.key);
            to.scale *= 0.998f32.powf(input.wheel.y);
            self.interaction.place(&t, to, &self.env, &self.tree);
            input.wheel = Vec2::ZERO;
        }
        // With no virtual cursor, pad 0's stick moves the selected element.
        let stick = input.gamepad.stick;
        if self.interaction.state.vcursor.desc.is_none() && stick.length() > 0.2 {
            let sel = self.interaction.state.hud.selected.clone();
            if let Some(t) = self
                .interaction
                .state
                .hud
                .targets
                .iter()
                .find(|t| t.key == sel)
                .cloned()
            {
                let mut to = self.interaction.placement(&t.key);
                let step = 400.0 / self.env.ticks_per_second.max(1) as f32;
                to.x += stick.x * step;
                to.y += stick.y * step;
                self.interaction.place(&t, to, &self.env, &self.tree);
                input.gamepad.stick = Vec2::ZERO;
            }
        }
        if took {
            input.pressed = false;
            input.down = false;
            input.released = false;
            input.pointers = og_input::Pointers::default();
        }
    }

    /// The edit mode's marks over the frame: an outline round each
    /// customisable element, the selected one's in the focus colour with
    /// its corners marked.
    pub(crate) fn hud_edit_commands(&self, cmds: &mut Vec<DrawCmd>) {
        let hud = &self.interaction.state.hud;
        if !hud.edit {
            return;
        }
        let p = &self.env.theme.palette;
        let screen = Rect::new(0.0, 0.0, self.env.viewport.width, self.env.viewport.height);
        let mut quad = |part: crate::dress::PartRef, r: Rect, owner: Rect, color: Srgba| {
            let values = self.part_values(&part, None);
            self.part_commands_at(
                None,
                &part,
                values.part_rect(r, owner),
                Some(values.color("color").unwrap_or(color)),
                0.0,
                1.0,
                screen,
                cmds,
            );
        };
        for t in &hud.targets {
            let selected = t.key == hud.selected;
            let (w, color) = if selected {
                (2.0, p.focus)
            } else {
                (1.0, p.ink.fade(0.6))
            };
            let r = t.shown.outset(3.0);
            for (side, edge) in [
                ("top", Rect::new(r.x, r.y, r.width, w)),
                ("bottom", Rect::new(r.x, r.bottom() - w, r.width, w)),
                ("left", Rect::new(r.x, r.y, w, r.height)),
                ("right", Rect::new(r.right() - w, r.y, w, r.height)),
            ] {
                quad(
                    crate::dress::PartRef::of("HudEditor", "outline", None)
                        .flag_if(selected, "selected")
                        .attr("side", side),
                    edge,
                    t.shown,
                    color,
                );
            }
            if selected {
                let k = 8.0;
                for (x, y) in [
                    (r.x, r.y),
                    (r.right(), r.y),
                    (r.x, r.bottom()),
                    (r.right(), r.bottom()),
                ] {
                    quad(
                        crate::dress::PartRef::of("HudEditor", "handle", None).flag("selected"),
                        Rect::new(x - k / 2.0, y - k / 2.0, k, k),
                        t.shown,
                        p.focus,
                    );
                }
            }
        }
    }

    /// The HUD edit mode's panel (docs/ui.md §2.1): the selected element's
    /// name, size and opacity steps, Next (the next element, for keys and
    /// pads), Reset and Done. Build it while [`Ui::hud_edit`] is on; Done
    /// turns the edit mode off. Presses on it edit nothing behind it.
    pub fn hud_layout_editor(
        &mut self,
        id: impl Into<Id>,
        desc: &HudEditorDesc,
    ) -> HudEditorResponse {
        let or = |s: &str, d: &'static str| {
            if s.is_empty() {
                d.to_owned()
            } else {
                s.to_owned()
            }
        };
        let info = self.hud_edit();
        let mut out = HudEditorResponse::default();
        let t = self.env.theme;
        self.open_panel(
            id,
            PanelDesc {
                gap: Some(t.space.sm),
                ..PanelDesc::default()
            },
        );
        self.front.hud_editor_key = self.tree.nodes[self.tree.parent()].key;
        self.label(
            "Title",
            &or(&desc.title, "Edit HUD"),
            LabelDesc {
                step: Some(t.type_ramp.lg),
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        self.label(
            "Selected",
            if info.selected.is_empty() {
                "—"
            } else {
                &info.selected
            },
            LabelDesc {
                color: Some(t.palette.ink_muted),
                nowrap: true,
                ..LabelDesc::default()
            },
        );
        let square = ButtonDesc {
            square: true,
            disabled: info.selected.is_empty(),
            ..ButtonDesc::default()
        };
        let mut step = |ui: &mut Ui, name: &str, label: String, s: i32, o: i32| {
            ui.open(
                Id::from(name),
                crate::Element {
                    direction: crate::Direction::Row,
                    gap: t.space.sm,
                    cross: crate::Cross::Center,
                    ..crate::Element::default()
                },
            );
            ui.label(
                "Name",
                &label,
                LabelDesc {
                    width: Size::grow(),
                    nowrap: true,
                    ..LabelDesc::default()
                },
            );
            if ui.button("Less", "−", square).clicked {
                ui.hud_step(-s, -o);
                out.changed = true;
            }
            if ui.button("More", "+", square).clicked {
                ui.hud_step(s, o);
                out.changed = true;
            }
            ui.close();
        };
        step(self, "Size", or(&desc.size_label, "Size"), 1, 0);
        step(self, "Opacity", or(&desc.opacity_label, "Opacity"), 0, 1);
        self.open(
            "Actions",
            crate::Element {
                direction: crate::Direction::Row,
                gap: t.space.sm,
                ..crate::Element::default()
            },
        );
        if self
            .button("Next", &or(&desc.next_label, "Next"), ButtonDesc::default())
            .clicked
            && !info.keys.is_empty()
        {
            let at = info.keys.iter().position(|k| *k == info.selected);
            let next = at.map_or(0, |i| (i + 1) % info.keys.len());
            self.hud_select(&info.keys[next]);
        }
        if self
            .button(
                "Reset",
                &or(&desc.reset_label, "Reset"),
                ButtonDesc {
                    variant: ButtonVariant::Ghost,
                    ..ButtonDesc::default()
                },
            )
            .clicked
        {
            self.hud_reset(if info.selected.is_empty() { "*" } else { "" });
            out.changed = true;
        }
        if self
            .button(
                "Done",
                &or(&desc.done_label, "Done"),
                ButtonDesc {
                    variant: ButtonVariant::Primary,
                    ..ButtonDesc::default()
                },
            )
            .clicked
        {
            self.set_hud_edit(false);
            out.done = true;
        }
        self.close();
        self.close();
        out
    }
}

impl crate::ui::Interaction {
    /// The tick of the frame being built, or of the one about to begin
    /// (the edit mode's input runs before `begin` counts it).
    fn hud_now(&self, tree: &crate::ui::Tree) -> u64 {
        self.state.tick + u64::from(!tree.building)
    }

    /// The topmost customisable element of last frame under `p`.
    fn hud_target_at(&self, p: Vec2) -> Option<HudTarget> {
        self.state
            .hud
            .targets
            .iter()
            .rev()
            .find(|t| t.shown.contains(p.x, p.y))
            .cloned()
    }

    fn placement(&self, key: &str) -> HudPlacement {
        let hud = &self.state.hud;
        hud.layouts[usize::from(hud.player)]
            .get(key)
            .copied()
            .unwrap_or_default()
    }

    fn place(
        &mut self,
        t: &HudTarget,
        p: HudPlacement,
        env: &crate::ui::Env,
        tree: &crate::ui::Tree,
    ) {
        let (w, h) = (env.viewport.width, env.viewport.height);
        let p = p.limited(t, w, h);
        let now = self.hud_now(tree);
        let hud = &mut self.state.hud;
        let l = &mut hud.layouts[usize::from(hud.player)];
        if p == HudPlacement::default() {
            l.remove(&t.key);
        } else {
            l.insert(t.key.clone(), p);
        }
        hud.changed = now;
    }

    /// The frame's customisable elements, after its transforms, for the
    /// next tick's edit mode.
    pub(crate) fn hud_collect(
        &mut self,
        order: &[usize],
        tree: &crate::ui::Tree,
        front: &crate::ui::Front,
    ) {
        let mut targets = Vec::new();
        let editor = match front.hud_editor_key {
            0 => Rect::default(),
            k => tree
                .index
                .find(k)
                .map_or(Rect::default(), |i| tree.nodes[i].placed.screen),
        };
        for &i in order {
            let n = &tree.nodes[i];
            let Some(spec) = &n.props.customize else {
                continue;
            };
            let pw = if n.parent == crate::layout::NONE {
                crate::Affine::IDENTITY
            } else {
                tree.nodes[n.parent].placed.world
            };
            targets.push(HudTarget {
                key: if spec.key.is_empty() {
                    tree.path_of(i)
                } else {
                    spec.key.clone()
                },
                base: pw.map_rect(&n.placed.rect),
                shown: n.placed.screen,
                spec: spec.clone(),
            });
        }
        let hud = &mut self.state.hud;
        hud.targets = targets;
        hud.editor = editor;
        if !hud.selected.is_empty() && !hud.targets.iter().any(|t| t.key == hud.selected) {
            hud.selected.clear();
        }
    }
}

impl crate::ui::Tree {
    /// Each customisable element's placement into its node before the
    /// frame's transforms (`apply_world`).
    pub(crate) fn hud_apply(&mut self, order: &[usize], interaction: &crate::ui::Interaction) {
        for &i in order {
            let Some(spec) = &self.nodes[i].props.customize else {
                continue;
            };
            let key = if spec.key.is_empty() {
                self.path_of(i)
            } else {
                spec.key.clone()
            };
            let p = interaction.placement(&key);
            self.nodes[i].style.custom = [p.x, p.y, p.scale, p.opacity];
        }
    }
}
