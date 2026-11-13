//! Interaction: what the UI reads from the player and what it does with it
//! before any element answers. Input sources, named actions and their glyphs,
//! focus navigation, gesture recognisers, scrolling, the pad's virtual cursor
//! and the player's own HUD layout. Each exists once; the immediate context
//! (`crate::ui`), the widgets and the documents call them.

pub mod actions;
pub mod hud_layout;
#[doc(hidden)]
pub mod input;
pub(crate) mod nav;
pub(crate) mod recognize;
pub(crate) mod scroll;
pub mod vcursor;
