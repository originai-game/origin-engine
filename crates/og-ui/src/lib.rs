//! Origin Engine UI (`docs/design.md` §2.7): an immediate-mode toolkit whose
//! tree is declared every tick, laid out Clay-style in one pass, styled by
//! the game's own UI style (the default style, the design language the
//! engine shares with origin-agent, until it picks one), drawn through og-render's UI pass
//! (analytic shapes and MSDF text over the finished frame, `paint`), and addressable by the probe through stable string ids.
//!
//! ```ignore
//! ui.begin(InputFrame::from_world(&world).unwrap_or_default(), viewport);
//! ui.panel("Menu", PanelDesc::default(), |ui| {
//!     ui.label("Title", "Origin", LabelDesc { step: Some(ui.theme().type_ramp.display_sm), ..Default::default() });
//!     if ui.button("Start", "Start", ButtonDesc { variant: ButtonVariant::Primary, ..Default::default() }).clicked {
//!         // start the game
//!     }
//! });
//! let frame = ui.end();
//! let overlay = scene.overlay.get_or_insert_with(|| Overlay::new(vp.width, vp.height, vp.dpr));
//! frame.emit(&EmitDesc { layer: 1000, time: ui.seconds() as f32 }, overlay);
//! ```
//!
//! Everything is in CSS pixels of the play surface (origin top left, y down),
//! the unit of `og.Input` and the probe. README.md has the widget set, the
//! layout rules, the id grammar, the theme generation and the probe contract.

pub use og_ui_core::*;

pub mod doc;
pub mod layers;
mod probe;
pub mod style_pack;

pub use probe::{SharedUi, UiProbe};

/// The document build's extension traits: `use og_ui::prelude::*` brings
/// its methods on [`Ui`] and [`UiDocument`] into scope.
pub mod prelude {
    pub use crate::doc::build::{
        LayersUi, MountUi, MountUi3, ProbeUi, RetainedMountingUi, StylePackUi, SvelteUiDocument,
        SvelteUiDocument2, TouchUi,
    };
}
