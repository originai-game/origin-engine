//! NSH's shared pointer-input decisions (longbridge/nsh 6fa6368f,
//! `crates/nsh-core/src/input.rs`, verbatim) over the nsh-core types they
//! name: what a press, a pan, a pinch, hover and a wheel do over plain
//! data, and the tab order. Hit testing is the host's (og-ui-next's arena).
mod shim;
pub mod input;
pub use nsh_tailwind::{Bounds, Edges, Point, Points, Size, geometry, geometry::px};
pub use shim::{EdgeInsets, EventKind, EventMask, GestureValues, NodeId, PanActivation, ROOT_ID, ViewKind, animation, keymap};
pub mod tailwind {
    pub use nsh_tailwind::tailwind::state;
}
