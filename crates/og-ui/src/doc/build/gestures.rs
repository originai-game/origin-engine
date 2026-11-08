//! HTML's drag and drop and Pointer Events on document elements (docs/ui.md
//! §2.1): `draggable` makes an element's [`crate::element::DragSpec`], a
//! handler of `ondragover` or `ondrop` its [`crate::element::DropSpec`],
//! and the input pass ([`super::retained`]'s `input_events`) turns the
//! tick's gestures and pointer into the handlers' events. Only the held
//! element, the one under the pointer and a drop's target are looked at,
//! so the pass costs nothing more as the tree grows.
use super::*;
use crate::element::{DragSpec, DropSpec};

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
