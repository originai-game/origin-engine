//! og-ui's new core (the og-ui rewrite, branch ui-rewrite). A node's
//! classes resolve once per distinct string into a flat typed record
//! (classes.rs, NSH's Tailwind); elements live in a flat arena indexed by
//! u32 and lay out on rune-taffy's dense store; a frame with nothing dirty does no style, layout or paint
//! work. There is no CSS cascade per element.

/// NSH's cursor, which a record's `cursor-*` names.
pub use nsh_tailwind::effects::Cursor;
pub mod a11y;
pub mod arena;
pub mod classes;
pub mod cover;
pub mod default_icons;
pub mod icons;
pub mod motion;
pub mod nav;
pub mod pack;
pub mod queue;
pub mod record;
pub mod sheet;
pub mod svg;
pub mod text;
pub mod theme;
pub mod tokens;
pub mod tree;
pub mod view;

pub use tokens::Tokens;
