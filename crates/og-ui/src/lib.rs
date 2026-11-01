//! Origin Engine UI (`docs/design.md` §2.7): an immediate-mode toolkit whose
//! tree is declared every tick, laid out Clay-style in one pass, styled by
//! the game's own UI style (the placeholder `prototype` until it picks
//! one; never the platform's design language), drawn through og-render's UI pass
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

mod a11y;
mod draw;
mod draw_codec;
pub mod dress;
pub(crate) mod edit;
pub mod element;
pub mod glyphs;
mod id;
mod input;
mod juice;
mod layout;
pub mod markup;
pub mod motion;
mod nav;
pub mod paint;
mod probe;
mod rich;
mod scroll;
pub mod shape;
pub mod slot;
mod style;
pub mod style_file;
mod text;
pub mod tokens;
#[rustfmt::skip]
pub mod theme;
mod ui;
mod widgets;

pub use a11y::{A11yNode, A11yTree};
pub use draw::{DrawCmd, DrawItem, EmitDesc, LAYER_BASE, LAYER_SPAN, MAX_LAYERS};
pub use dress::{DressFrame, Dressing, Selector};
pub use element::{
    Affine, Dismiss, DragAxis, DragSpec, DropSpec, Edges, GridFlow, GridPlace, GridSpec,
    LayoutMode, LayoutStats, Orientation, Overlap, Patch, PinchSpec, Position, Prop, Props, Query,
    Track, Transform, Values, Variant,
};
pub use id::{Id, ui_id, ui_idi};
pub use input::{GamepadFrame, InputFrame, InputSource, Nav, OgInput, UiKey};
pub use juice::{
    BurstDesc, CounterDesc, ScreenTransition, ScreenTransitionDesc, ScreenWipe, ShakeDesc, Side,
    format_count,
};
pub use layout::{
    Align, Anchor, Cross, Direction, Element, FontRole, Insets, Rect, Size, TextAlign,
};
pub use motion::{
    Choreo, Ease, EventKind, GestureInfo, Key, KeyEase, MotionDump, MotionToken, MotionTokens,
    PlayDesc, PlayInfo, PresenceSpec, Reduced, Repeat, SpringSpec, StaggerFrom, StateGroup,
    StateTransition, Timeline, TimelineEvent, TimelineTrack, Transition, TransitionKind, UiEvent,
    VisualState, cubic_bezier, ease_exit, ease_out,
};
pub use probe::{SharedUi, UiProbe};
pub use rich::{RichTextDesc, RichTextResponse};
pub use slot::{Aspect, SlotKind, SlotSpec};
pub use style::{
    Breakpoint, Contrast, Density, DensityMode, Durations, Palette, READABLE, Radius, Scheme,
    Space, Spring, Srgba, Theme, TypeRamp, TypeStep,
};
pub use style_file::{
    Art, Slice, SliceFill, StyleFile, StylePart, UiSound, parse_style, upgrade_style,
};
pub use text::{FontMetrics, Piece, UiFont, VisualGlyph, bidi_runs, needs_visual, pieces};
pub use tokens::StyleTokens;
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
