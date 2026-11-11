//! Control elements: what a control is built from, how it answers input
//! and how it looks, kept apart (#41 step 2, docs/ui.md §2).
//!
//! A control is an element kind in three parts:
//!
//! - its **model** ([`Control`], one struct of plain values per kind):
//!   what it is built from. A document fills it from its resolved values,
//!   the immediate front end from its `*Desc`. [`Ui::set_control`] keeps
//!   it with the control's host node, by the node's key, for as long as
//!   the node lives.
//! - its **behaviour** ([`Ui::dispatch_control`]): one tick's input to the
//!   host node. It reads the interaction state and the model, moves the
//!   control's own motion and memory, asks for focus and feedback, and
//!   answers a [`ControlOutput`]: the response and the state the
//!   appearance is drawn from. It writes nothing of the node's box, parts
//!   or content, so it runs on a node where it stands.
//! - its **appearance** ([`Ui::present_control`]): the model and an output
//!   written onto the host node and its part boxes, in place and
//!   idempotently. It processes no input and samples no motion.
//!
//! Who runs them:
//!
//! - The immediate front end (`Ui::button(…)`) runs all three each tick
//!   ([`Ui::control_at`]) and answers the output's response.
//! - A document building an element runs all three once, records the
//!   output in the element's render fields and layers its CSS over the
//!   appearance (`apply_document_fields`, which presents again from the
//!   recorded output whenever the element is restyled).
//! - A mounted document whose control is hovered, pressed, focused or
//!   resized does not emit the element again: its host runs the behaviour
//!   on the node in place (`RetainedHost::behave`), records the new
//!   output, presents it through the same `apply_document_fields` and
//!   fires the element's events from the output's response, in the place
//!   in the frame the element's emission had. The element is emitted only
//!   when its resolved inputs or its structure change.
//!
//! The output is returned to whoever dispatched, not queued: events and
//! feedback keep the order of the elements in the frame.

use crate::ui::{Response, Ui};

mod button;
pub(crate) use button::ButtonModel;

/// A control's model, by kind.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Control {
    Button(ButtonModel),
}

/// What a control's behaviour made of one tick's input: its response, and
/// the state its appearance is drawn from.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ControlOutput {
    pub(crate) response: Response,
    /// How far into hover and press it is (its part's state mix).
    pub(crate) amounts: (f32, f32),
    /// How far in its press motion is.
    pub(crate) press: f32,
}

/// The controls of the nodes that live, by host node key.
#[derive(Clone, Debug, Default)]
pub(crate) struct Controls {
    live: std::collections::HashMap<u64, Control>,
    /// How many lived after the last sweep.
    swept: usize,
}

impl Controls {
    /// Forgets the controls of nodes that left the trees, once enough
    /// have gathered to be worth the walk (a frame that builds nothing
    /// pays nothing).
    pub(crate) fn sweep(&mut self, tree: &crate::ui::Tree) {
        if self.live.len() <= self.swept * 2 + 64 {
            return;
        }
        self.live
            .retain(|key, _| tree.index.by_key.contains_key(key));
        self.swept = self.live.len();
    }
}

impl Ui {
    /// Keeps `control` as node `i`'s model (build and update: the same call).
    pub(crate) fn set_control(&mut self, i: usize, control: Control) {
        let key = self.tree.nodes[i].key;
        self.controls.live.insert(key, control);
    }

    /// Whether the node of key `key` carries a control.
    pub(crate) fn has_control(&self, key: u64) -> bool {
        self.controls.live.contains_key(&key)
    }

    /// The behaviour of node `i`'s control for this tick's input.
    pub(crate) fn dispatch_control(&mut self, i: usize) -> ControlOutput {
        let key = self.tree.nodes[i].key;
        match self.controls.live.get(&key) {
            Some(Control::Button(model)) => {
                let behaviour = model.behaviour();
                self.dispatch_button(i, behaviour)
            }
            None => ControlOutput::default(),
        }
    }

    /// Writes node `i`'s control as its model and `output` show it.
    pub(crate) fn present_control(&mut self, i: usize, output: &ControlOutput) {
        let key = self.tree.nodes[i].key;
        let Some(control) = self.controls.live.remove(&key) else {
            return;
        };
        match &control {
            Control::Button(model) => self.present_button(i, model, output),
        }
        self.controls.live.insert(key, control);
    }

    /// The three parts on node `i` in one tick: what the immediate front
    /// end and a document's first build of an element do.
    pub(crate) fn control_at(&mut self, i: usize, control: Control) -> ControlOutput {
        self.set_control(i, control);
        let output = self.dispatch_control(i);
        self.present_control(i, &output);
        output
    }
}

impl crate::ui::Tree {
    /// Opens the retained node `i` and its content box for an update where
    /// they stand: the frame compares them with what they were, as it does
    /// a node written again, and they set their draws again.
    pub(crate) fn update_in_place(&mut self, i: usize, layout: &mut crate::ui::Layout) {
        let content = self.nodes[i].content_slot;
        for at in [i, content] {
            if at >= self.nodes.len() {
                continue;
            }
            if !self.emitted.contains(&at) {
                layout
                    .cache
                    .prev
                    .entry(at)
                    .or_insert_with(|| self.nodes[at].clone());
                self.emitted.push(at);
            }
            self.draws.remove(&at);
            self.dressable.remove(&at);
        }
    }
}
