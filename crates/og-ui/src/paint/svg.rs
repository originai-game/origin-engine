//! SVG filters and SMIL animation of documents' inline `<svg>`, over the
//! SVG model og-ui-next owns ([`og_ui_next::svg`]).

use og_core::glam::Vec2;
pub(crate) use og_ui_next::svg::*;

pub mod filter;
pub mod smil;
