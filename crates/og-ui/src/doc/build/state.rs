//! The documents' own state (docs/ui.md §7): what a build keeps as it goes,
//! what it tells code and the probe this tick, and what its mounts keep
//! between ticks. The stages hold it in [`crate::ui::Docs`].

use super::host::{DocRecord, FieldValidity, PopoverLink, Revealing};
use super::{DocEvent, DocMemory, Edit};
use crate::style::Srgba;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct DocState {
    /// What a snapshot carries: the mounts, the reactive session, the
    /// elements seen and the triggers' last values.
    pub memory: DocMemory,
    /// The records of the retained elements being emitted, innermost last.
    pub capture: Vec<DocRecord>,
    pub edits: Vec<Edit>,
    /// Text fields' constraint validity as last built, by key: what a
    /// form's submit reads. Kept while their element is.
    pub validity: BTreeMap<u64, FieldValidity>,
    /// What media elements asked of their players this frame, taken by
    /// the host ([`crate::Ui::take_media_ops`]).
    pub media_ops: Vec<crate::bind::MediaOp>,
    /// What each media element last sent, the time it last read and the
    /// frames a seek it sent is still on its way, by key.
    pub media_sent: BTreeMap<u64, (f32, bool, f64, u8)>,
    /// What happened to the element being built beyond its response
    /// (`ontoggle`, `onclose`), taken by its triggers.
    pub happened: Vec<crate::doc::When>,
    /// The popovers documents name, by their key in their mount.
    pub popovers: BTreeMap<u64, PopoverLink>,
    /// The id the last `#fragment` link named: CSS `:target`.
    pub target: Option<String>,
    /// Events queued for code (screens built at the end of a tick reach
    /// code the next).
    pub queued: Vec<DocEvent>,
    /// Layer operations triggers asked for this tick, done after the build.
    pub ops: Vec<crate::layer_stack::LayerOp>,
    /// Plays a document or the probe started: `(path, timeline)` → play.
    pub plays: BTreeMap<(String, String), u32>,
    /// The tick the layers were last built.
    pub built_tick: u64,
    /// The mount being built is the top layer that takes input (its `on action` triggers fire).
    pub accepts: bool,
    /// The serial of the layer being built (0 outside one): what its `pop` pops.
    pub building: u32,
    /// The retained element being built holds its node: hidden
    /// (`display: none`, `hidden`), it keeps the node and its subtree, as
    /// the web keeps a hidden element's DOM, and showing it again is a field
    /// change. One never built is not built while hidden.
    pub keeps_hidden: bool,
    /// How deep the element images being built are nested (an `feImage`
    /// of an element whose filters draw element images): past 4, no more.
    pub element_images: u32,
    /// View-transition covers still opening after their layer left: the
    /// covers, their colour and image and the tick it left.
    pub revealing: Vec<Revealing>,
    /// The on-screen controls documents lay out (`<joystick action>`,
    /// `<dpad action>`, `<button data-og-touch>`), by render key, their
    /// rects filled from the frame's hits; `zones_changed`: og-input's
    /// copy is stale ([`crate::Ui::take_control_zones`]).
    pub zones: BTreeMap<u64, og_input::ControlZone>,
    pub zones_changed: bool,
    /// The instantiated sibling levels of the screen being built.
    pub levels: Vec<super::structure::Level>,
    /// `data-og-instant` elements whose first entrance has landed: a
    /// later one plays (presentation, outside `world_hash`).
    pub(crate) entered: BTreeSet<u64>,
    /// The `$ui` store's value as last published.
    pub(crate) ui_store: Option<Value>,
    /// The document-order counter and quote run of the tick being built,
    /// kept only while some defined document uses generated content.
    pub run: crate::doc::generated::Run,
    /// The `::placeholder` color of the text control being built.
    pub placeholder_ink: Option<Srgba>,
    /// `::selection` styles by render key.
    pub selection: BTreeMap<u64, Arc<crate::doc::generated::Selection>>,
}

impl DocState {
    /// The tick's document bookkeeping starts.
    pub(crate) fn begin_tick(&mut self) {
        self.run.clear();
        self.memory.begin();
    }
    /// Restores what a snapshot carried.
    pub(crate) fn restore(&mut self, v: &Value) -> Result<(), String> {
        self.memory.restore(v)?;
        self.built_tick = 0;
        Ok(())
    }
    /// The art slot registered last by the element being emitted.
    pub(crate) fn last_art_entry(&self) -> Option<&(String, crate::slot::SlotSpec)> {
        self.capture.last()?.arts.last()
    }
    /// The art slot `path`'s spec, where it was registered.
    pub(crate) fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        if let Some((_, spec)) = self
            .capture
            .iter_mut()
            .rev()
            .flat_map(|r| r.arts.iter_mut())
            .find(|(p, _)| p == path)
        {
            return Some(spec);
        }
        self.memory.record_art_slot_mut(path)
    }
    /// Keeps the widget memory of elements `built` this frame or kept by
    /// their mounts, and forgets the rest.
    pub(crate) fn retain_element_memory(&mut self, built: impl Fn(u64) -> bool) {
        let retained = &self.memory.retained;
        self.validity
            .retain(|k, _| built(*k) || retained.keeps_memory(*k));
        self.media_sent
            .retain(|k, _| built(*k) || retained.keeps_memory(*k));
    }
}
