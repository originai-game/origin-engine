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
    /// What the last `#fragment` link indicated: CSS `:target`.
    pub target: Option<FragmentTarget>,
    /// The text directives of a link followed this tick, for the layout
    /// to find ([`crate::ui::target_text`]).
    pub(crate) text_directives: Option<Vec<crate::ui::target_text::TextDirective>>,
    /// Selector states set from outside the documents this frame
    /// ([`crate::Ui::set_selector_state`]): `(key, state, on)`, applied
    /// when the documents next refresh.
    pub selector_states: Vec<(u64, crate::bind::InteractionState, bool)>,
    /// The scroll controls built for each scroll container and each
    /// `scroll-target-group` element, by its render key, as last built
    /// (CSS Overflow 5 §3).
    pub scroll_groups: BTreeMap<u64, super::scroll_markers::Group>,
    /// Each element's `::scroll-marker` as it was last built, by the
    /// element's render key.
    pub scroll_markers: BTreeMap<u64, Arc<super::scroll_markers::Marker>>,
    /// Each multi-column element's `::column::scroll-marker` as it was last
    /// built, by the element's render key: a marker for each of its columns.
    pub column_markers: BTreeMap<u64, Arc<super::scroll_markers::Marker>>,
    /// The column each `::column::scroll-marker` targets, by the target key
    /// its group names it by ([`super::scroll_markers::column_target`]):
    /// its multi-column element's render key and the column's index.
    pub column_targets: BTreeMap<u64, (u64, usize)>,
    /// The `#id` each element with an `href` naming one names, by its
    /// render key.
    pub scroll_anchors: BTreeMap<u64, String>,
    /// The elements with `scroll-target-group: auto`, by render key.
    pub target_groups: BTreeSet<u64>,
    /// The scroll controls pressed this frame, by their container's key:
    /// they scroll once the frame is laid out.
    pub scroll_presses: Vec<(u64, super::scroll_markers::Press)>,
    /// The `:target-*` state each scroll marker was last given, by its key.
    pub marker_states: BTreeMap<u64, crate::bind::InteractionState>,
    /// The scroll buttons last set `:disabled`, by key.
    pub button_states: BTreeSet<u64>,
    /// Each scroll container's current scroll target, by key: the target
    /// a marker scrolled it to and the offset it aimed at, kept while the
    /// container stays there.
    pub scroll_pins: BTreeMap<u64, (u64, og_core::glam::Vec2)>,
    /// The boxes styled as a pseudo-element since the last refresh:
    /// `(the box's render key, the pseudo-element's key)`.
    pub pseudo_boxes: Vec<(u64, u64)>,
    /// The fields and disclosures whose own value or open state the player
    /// changed this frame ([`DocMemory::values`], [`DocMemory::open`]):
    /// their selector facts are read again.
    pub edited: BTreeSet<u64>,
    /// Events queued for code (screens built at the end of a tick reach
    /// code the next).
    pub queued: Vec<DocEvent>,
    /// What remote frames' documents sent, queued as `queued` is.
    pub frame_messages: Vec<crate::doc::frames::FrameMessage>,
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
    /// View-transition covers still opening after their layer left: the
    /// covers, their colour and image and the tick it left.
    pub revealing: Vec<Revealing>,
    /// The on-screen controls documents lay out (`<joystick action>`,
    /// `<dpad action>`, `<button data-og-touch>`), by render key, their
    /// rects filled from the frame's hits; `zones_changed`: og-input's
    /// copy is stale ([`crate::Ui::take_control_zones`]).
    pub zones: BTreeMap<u64, og_input::ControlZone>,
    pub zones_changed: bool,
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
    /// `::target-text` styles by render key.
    pub target_text: BTreeMap<u64, Arc<crate::doc::generated::Selection>>,
    /// Each `::highlight(<name>)` style by render key, then name.
    pub highlights: BTreeMap<u64, BTreeMap<String, crate::doc::generated::Selection>>,
}

/// The element a `#fragment` link indicates (CSS `:target`).
#[derive(Clone, Debug, PartialEq)]
pub enum FragmentTarget {
    /// The element with this id.
    Id(String),
    /// The element (render key) holding the passage a text fragment
    /// matched.
    Element(u64),
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
        self.restore_scroll_controls(v.get("scrollControls"))?;
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
        self.scroll_groups.retain(|k, _| built(*k));
        self.scroll_markers.retain(|k, _| built(*k));
        self.column_markers.retain(|k, _| built(*k));
        self.column_targets.retain(|_, (owner, _)| built(*owner));
        self.scroll_anchors.retain(|k, _| built(*k));
        self.target_groups.retain(|k| built(*k));
        self.scroll_pins.retain(|k, _| built(*k));
        let retained = &self.memory.retained;
        self.validity
            .retain(|k, _| built(*k) || retained.keeps_memory(*k));
        self.media_sent
            .retain(|k, _| built(*k) || retained.keeps_memory(*k));
        let retained = &self.memory.retained;
        let kept: Vec<u64> = self
            .memory
            .values
            .keys()
            .copied()
            .filter(|k| built(*k) || retained.keeps_memory(*k))
            .collect();
        self.memory.values.retain(|k, _| kept.contains(k));
        let retained = &self.memory.retained;
        let kept: Vec<u64> = self
            .memory
            .open
            .keys()
            .copied()
            .filter(|k| built(*k) || retained.keeps_memory(*k))
            .collect();
        self.memory.open.retain(|k, _| kept.contains(k));
    }
}
