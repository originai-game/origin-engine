//! Origin Engine UI (`docs/design.md` §2.7): an immediate-mode toolkit whose
//! tree is declared every tick, laid out Clay-style in one pass, themed by the
//! platform's design tokens, drawn through og-render's sprite and MSDF text
//! batches, and addressable by the probe through stable string ids.
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
//! frame.emit(&EmitDesc { atlas, layer: 1000, z: 0.0 }, &mut scene.sprites, &mut scene.texts);
//! ```
//!
//! Everything is in CSS pixels of the play surface (origin top left, y down),
//! the unit of `og.Input` and the probe. README.md has the widget set, the
//! layout rules, the id grammar, the theme generation and the probe contract.

mod draw;
mod id;
mod input;
mod layout;
mod probe;
mod style;
mod text;
#[rustfmt::skip]
pub mod theme;
mod ui;

pub use draw::{atlas_texture, camera, DrawCmd, DrawItem, EmitDesc};
pub use id::{ui_id, ui_idi, Id};
pub use input::{InputFrame, InputSource, OgInput, UiKey};
pub use layout::{Align, Cross, Direction, Element, FontRole, Insets, Rect, Size, TextAlign};
pub use probe::{SharedUi, UiProbe};
pub use style::{
    Density, DensityMode, Durations, Hues, Palette, Radius, Scheme, Space, Spring, Srgba, Theme,
    TypeRamp, TypeStep,
};
pub use text::{FontMetrics, UiFont};
pub use ui::{
    ButtonDesc, ButtonVariant, ControlSize, FieldDesc, Fonts, ImageDesc, LabelDesc, PanelDesc,
    ProgressDesc, Response, ScrollDesc, SliderDesc, Ui, UiDesc, UiFrame, Viewport,
};

/// The probe's element type: what `dump_ui` lists and the id and text lookups search.
pub use og_probe::UiElement;
