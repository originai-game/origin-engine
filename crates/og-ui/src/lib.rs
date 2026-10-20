//! Origin Engine UI (`docs/design.md` §2.7): an immediate-mode toolkit whose
//! tree is declared every tick, laid out Clay-style in one pass, themed by the
//! platform's design tokens, drawn through og-render's screen-space overlay
//! (sprites and MSDF text over the finished frame), and addressable by the probe through stable string ids.
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
//! frame.emit(&EmitDesc { atlas, layer: 1000 }, overlay);
//! ```
//!
//! Everything is in CSS pixels of the play surface (origin top left, y down),
//! the unit of `og.Input` and the probe. README.md has the widget set, the
//! layout rules, the id grammar, the theme generation and the probe contract.

mod a11y;
mod draw;
pub(crate) mod edit;
pub mod glyphs;
mod id;
mod input;
mod juice;
mod layout;
pub mod markup;
mod motion;
mod nav;
mod probe;
mod rich;
mod scroll;
pub mod shape;
pub mod skin;
mod style;
mod text;
#[rustfmt::skip]
pub mod theme;
mod ui;
mod widgets;

pub use a11y::{A11yNode, A11yTree};
pub use draw::{DrawCmd, DrawItem, EmitDesc, atlas_texture};
pub use id::{Id, ui_id, ui_idi};
pub use input::{GamepadFrame, InputFrame, InputSource, Nav, OgInput, UiKey};
pub use juice::{
    BurstDesc, CounterDesc, ScreenTransition, ScreenTransitionDesc, ScreenWipe, ShakeDesc, Side,
    format_count,
};
pub use layout::{
    Align, Anchor, Cross, Direction, Element, FontRole, Insets, Rect, Size, TextAlign,
};
pub use motion::{TransitionKind, cubic_bezier, ease_exit, ease_out};
pub use probe::{SharedUi, UiProbe};
pub use rich::{RichTextDesc, RichTextResponse};
pub use skin::{Skin, SkinFile, SkinPart, Slice, SliceFill, UiSound, parse_skin};
pub use style::{
    Breakpoint, Contrast, Density, DensityMode, Durations, HueName, Hues, Palette, Radius, Scheme,
    Space, Spring, Srgba, Theme, TypeRamp, TypeStep,
};
pub use text::{FontMetrics, UiFont, VisualGlyph, bidi_runs, needs_visual};
pub use ui::{
    ButtonDesc, ButtonVariant, ControlSize, FieldDesc, Fonts, IconAtlas, ImageDesc, LabelDesc,
    PanelDesc, ProgressDesc, Response, ScrollAxes, ScrollDesc, SliderDesc, TextAreaDesc,
    TextTarget, Ui, UiDesc, UiFrame, Viewport,
};
pub use widgets::{
    ActionButtonDesc, AnchorDesc, ChipDesc, CooldownDesc, DialogueDesc, DialogueResponse, DpadDesc,
    DpadResponse, GridDesc, IconDesc, JoystickDesc, JoystickResponse, ListDesc, ListRange,
    MapMarker, MeterDesc, MeterKind, MinimapDesc, ModalDesc, ModalResponse, Presence, RadioDesc,
    SelectDesc, SkeletonDesc, SpotlightDesc, StepperDesc, TabsDesc, ToastDesc, Tone,
    TransitionDesc, ZoomResponse, ZoomViewDesc,
};

/// The probe's element type: what `dump_ui` lists and the id and text lookups search.
pub use og_probe::UiElement;
