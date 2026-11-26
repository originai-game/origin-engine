//! Portable checkpoint and restore of a [`Tree`]'s deterministic presentation
//! state: focus (with keyboard modality and focus-memory groups), open modal
//! focus stack, scroll offsets, `<Input>` text/caret/selection, UI sound RNG
//! state, virtual cursor position/state, and UI clock time.
//!
//! Node references are stored by renderer (`js`) id rather than internal arena
//! indices so checkpoints remain stable across pool reuse and survive
//! serialization alongside world snapshots (`UiCheckpoint`).

use super::input::Modality;
use super::{NONE, Tree};

/// Saved state of a single `<Input>` element, keyed by its renderer (`js`) id.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InputFieldState {
    pub js: u32,
    pub value: String,
    pub caret: usize,
    pub anchor: usize,
    #[serde(default)]
    pub scroll_x: f32,
    #[serde(default)]
    pub scroll_y: f32,
}

/// Saved state of the pad virtual cursor between ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VirtualCursorCheckpoint {
    pub pos: [f32; 2],
    pub shown: bool,
    pub placed: bool,
    pub held: u32,
    pub down: bool,
    pub last: [f32; 2],
    pub over: bool,
}

/// Portable deterministic snapshot of a [`Tree`]'s interactive presentation
/// state. Carried beside the simulation snapshot in `UiCheckpoint`.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TreeCheckpoint {
    /// Currently focused element's renderer id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<u32>,
    /// Whether keyboard/pad modality is active (`focus-visible:`).
    #[serde(default)]
    pub focus_keyboard: bool,
    /// Last-used device flags (`pad` / `touch`).
    #[serde(default)]
    pub pad: bool,
    #[serde(default)]
    pub touch: bool,
    /// Open modal stack as `(modal_js_id, return_focus_js_id)`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modals: Vec<(u32, Option<u32>)>,
    /// `focusMemory` last-focused member per group: `(group_js_id, member_js_id)`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_last: Vec<(u32, u32)>,
    /// Non-zero scroll offsets by scroller renderer id, sorted by id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scrolls: Vec<(u32, [f32; 2])>,
    /// `<Input>` field text, selection and horizontal scroll by renderer id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputFieldState>,
    /// Deterministic PRNG state for pack sound clip/pitch/volume selection.
    #[serde(default)]
    pub sound_rng: u64,
    /// Virtual cursor position and ramp state.
    #[serde(default)]
    pub vcursor: VirtualCursorCheckpoint,
    /// UI clock time when the checkpoint was captured.
    #[serde(default)]
    pub time: f64,
}

impl Tree {
    /// Resolves a live op element index to its renderer (`js`) id.
    fn js_of(&self, e: u32) -> Option<u32> {
        let el = self.els.get(e as usize)?;
        (!el.dead && el.js != NONE).then_some(el.js)
    }

    /// Resolves a renderer (`js`) id back to a live op element index.
    fn el_of_js(&self, js: u32) -> Option<u32> {
        let &e = self.by_js.get(js as usize)?;
        (e != NONE && self.els.get(e as usize).is_some_and(|el| !el.dead)).then_some(e)
    }

    /// Captures the tree's deterministic interactive state for snapshot/restore.
    pub fn checkpoint(&self) -> TreeCheckpoint {
        let focus = (self.input.focus != NONE)
            .then(|| self.js_of(self.input.focus))
            .flatten();
        let modals = self
            .input
            .modals
            .iter()
            .filter_map(|&(m, ret)| {
                let m_js = self.js_of(m)?;
                let ret_js = (ret != NONE).then(|| self.js_of(ret)).flatten();
                Some((m_js, ret_js))
            })
            .collect();
        let mut group_last = Vec::new();
        let mut scrolls = Vec::new();
        let mut inputs = Vec::new();
        for (idx, el) in self.els.iter().enumerate() {
            if el.dead || el.js == NONE {
                continue;
            }
            let Some(per) = self.input.per.get(idx) else {
                continue;
            };
            if let Some(last) = per.last
                && let Some(last_js) = self.js_of(last)
            {
                group_last.push((el.js, last_js));
            }
            if per.scroll != [0.0, 0.0] {
                scrolls.push((el.js, per.scroll));
            }
            if per.input {
                inputs.push(InputFieldState {
                    js: el.js,
                    value: per.value.clone(),
                    caret: per.caret,
                    anchor: per.anchor,
                    scroll_x: per.scroll_x,
                    scroll_y: per.scroll_y,
                });
            }
        }
        group_last.sort_unstable_by_key(|&(js, _)| js);
        scrolls.sort_unstable_by_key(|&(js, _)| js);
        inputs.sort_unstable_by_key(|f| f.js);
        TreeCheckpoint {
            focus,
            focus_keyboard: self.input.modality == Modality::Keyboard,
            pad: self.input.pad,
            touch: self.input.touch,
            modals,
            group_last,
            scrolls,
            inputs,
            sound_rng: self.input.sound_rng,
            vcursor: self.vcursor.checkpoint(),
            time: self.time,
        }
    }

    /// Restores deterministic interactive state from `cp` without emitting
    /// spurious focus, blur or change events to JS.
    pub fn restore(&mut self, cp: &TreeCheckpoint) {
        self.time = cp.time;
        self.input.sound_rng = cp.sound_rng;
        self.input.pad = cp.pad;
        self.input.touch = cp.touch;
        self.input.modality = if cp.focus_keyboard {
            Modality::Keyboard
        } else {
            Modality::Pointer
        };
        // Clear transient pointer/gesture, motion, rolling-count and
        // text-selection interactions so restored state starts clean from
        // the checkpointed tick.
        self.input.clear_transient();
        self.motion.clear();
        // New nodes may enter before present samples again. Start their
        // transitions at the restored time, never at the abandoned future.
        self.motion.tick_at(cp.time);
        self.counts.clear();
        self.select = None;
        self.select_press = None;

        // Restore virtual cursor state.
        self.vcursor.restore(&cp.vcursor);

        // Reset scroll offsets and focus-memory markers before applying saved values.
        for per in &mut self.input.per {
            per.scroll = [0.0, 0.0];
            per.last = None;
        }
        for &(js, offset) in &cp.scrolls {
            if let Some(e) = self.el_of_js(js)
                && let Some(per) = self.input.per.get_mut(e as usize)
            {
                per.scroll = offset;
            }
        }
        for &(group_js, last_js) in &cp.group_last {
            if let (Some(g), Some(last)) = (self.el_of_js(group_js), self.el_of_js(last_js))
                && let Some(per) = self.input.per.get_mut(g as usize)
            {
                per.last = Some(last);
            }
        }
        // Restore modal return-focus stack.
        self.input.modals = cp
            .modals
            .iter()
            .filter_map(|&(m_js, ret_js)| {
                let m = self.el_of_js(m_js)?;
                let ret = ret_js.and_then(|id| self.el_of_js(id)).unwrap_or(NONE);
                Some((m, ret))
            })
            .collect();

        // Restore `<Input>` field text, caret, selection and scroll offsets.
        for field in &cp.inputs {
            if let Some(e) = self.el_of_js(field.js)
                && self.input.per.get(e as usize).is_some_and(|p| p.input)
            {
                let per = &mut self.input.per[e as usize];
                let len = field.value.len();
                let clamp_boundary = |s: &str, mut idx: usize| {
                    idx = idx.min(len);
                    while idx > 0 && !s.is_char_boundary(idx) {
                        idx -= 1;
                    }
                    idx
                };
                let caret = clamp_boundary(&field.value, field.caret);
                let anchor = clamp_boundary(&field.value, field.anchor);
                per.value = field.value.clone();
                per.caret = caret;
                per.anchor = anchor;
                per.scroll_x = field.scroll_x;
                per.scroll_y = field.scroll_y;
                per.composing.clear();
                per.undo.clear();
                per.redo.clear();
                per.last_edit = (super::input::Edit::None, 0.0);
                per.goal_x = None;
                per.caret_at = cp.time;
                self.show_value(e);
            }
        }

        // Restore focus states silently (no Focus/Blur events emitted).
        let target = cp.focus.and_then(|js| self.el_of_js(js)).unwrap_or(NONE);
        self.set_focus_silent(target, cp.focus_keyboard);
        self.paint = true;
    }
}
