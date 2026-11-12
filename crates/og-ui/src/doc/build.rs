//! Mounting a document (docs/ui.md §2.1): its tree is built into the
//! immediate-mode [`Ui`] where it is mounted, every tick, so documents and
//! code mix in one tree. Props are resolved in order (the line, the
//! matching `at` blocks, a probe edit), bindings are evaluated against the
//! viewmodel ([`crate::bind`]), `for` builds a list's rows (only those in
//! view under `list` and `cells`), `slot`s are left for code to fill
//! ([`Ui::open_slot`]), triggers fire, and controls write their values back
//! into the viewmodel (two-way bindings). What the build saw is kept for
//! the probe: every element's document line, bindings and their values,
//! the slots and art slots, the probe's edits.

pub use og_ui_core::doc::build::*;

pub(crate) mod color;
pub(crate) mod datetime;
pub(crate) mod ext;
pub(crate) mod generated;
pub(crate) mod gestures;
pub(crate) mod kbd;
pub(crate) mod media;
pub(crate) mod mount;
pub(crate) mod nodes;
pub(crate) mod popover;
pub(crate) mod retained;
pub(crate) mod state;
#[doc(hidden)]
pub mod svg;
pub(crate) mod text_motion;
pub(crate) mod touch;
pub(crate) mod viewport;
pub(crate) mod world;
pub use ext::*;
pub(crate) use state::DocsState;

use super::schema::{self, PROPS, Ty};
use super::{Act, DocNode, Lit, NodeKind, Paint, PropValue, UiDocument, When};
use crate::bind::{Env, set_path, truthy};
use crate::element::Props;
use crate::layout::{Cross, Direction, Element, Insets, NONE, Size, TextAlign};
use crate::style::Srgba;
use crate::ui::{ButtonDesc, ButtonVariant, LabelDesc, Ui};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
