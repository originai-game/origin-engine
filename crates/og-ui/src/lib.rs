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

/// The engine's default CSS, generated from the shared design tokens.
pub const DEFAULT_CSS: &str = include_str!("../default.css");

/// The role rules of the default CSS (`:where([part$=".frame"])` and the
/// rest, its last section): what dresses a document's own elements. A style
/// pack's sheet holds the whole default; a document without a pack takes
/// these, and reads their tokens from the root cascade.
pub static DEFAULT_ROLE_CSS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let at = DEFAULT_CSS
        .find("/* Roles:")
        .expect("the generated default CSS ends with its role rules");
    format!("@layer og-default {{\n{}", &DEFAULT_CSS[at..])
});

#[doc(hidden)]
pub mod a11y;
pub use interaction::actions;
pub use og_reactive::bind;
pub(crate) use paint::decor;
pub use style::css_color;
pub use text::colr;
pub mod css;
pub mod doc;
mod draw;
pub(crate) use draw::draw_codec;
pub use style::dress;
pub(crate) use text::edit;
pub mod element;
#[doc(hidden)]
pub use element::id;
pub use input_type::{InputMode, InputType};
pub use interaction::hud_layout;
#[doc(hidden)]
pub use text::input_type;
pub mod interaction;
#[doc(hidden)]
pub use interaction::input;
pub(crate) use motion::juice;
#[doc(hidden)]
pub use ui::layer_stack;
#[doc(hidden)]
pub mod layout;
pub use text::markup;
pub mod motion;
#[cfg(not(target_arch = "wasm32"))]
pub use a11y::native_a11y;
pub mod paint;
pub(crate) use interaction::recognize;
pub(crate) use interaction::scroll;
pub use og_reactive::reactive;
pub use og_rive as rive;
pub use paint::slot;
pub use scroll::{Overscroll, ScrollCss, ScrollState};
pub use style::paint_entry;
pub use style::sound_css;
#[doc(hidden)]
pub use text::rich;
#[doc(hidden)]
pub mod style;
pub use og_text::{glyphs, shape};
pub use paint::svg;
pub use style::style_file;
mod text;
pub use interaction::vcursor;
pub use style::tokens;
#[rustfmt::skip]
pub mod theme;
pub use paint::particles;
#[doc(hidden)]
pub mod ui;
#[doc(hidden)]
pub use layout::virtualize;
pub(crate) use motion::view_transition;
pub use world::viewport;
#[doc(hidden)]
pub mod widgets;
pub mod world;

pub mod layers;
mod probe;
pub mod style_pack;

pub use probe::{SharedUi, UiProbe};

#[cfg(test)]
mod structure;

pub use a11y::{A11yNode, A11yTree, ANNOUNCEMENTS, Announcement, ROLES, Semantics};
pub use actions::{ActionDesc, ActionHint, ActionTrigger, Device, Glyph};
pub use doc::UiDocument;
pub use doc::build::DocEvent;
pub use draw::{
    DrawCmd, DrawItem, EmitDesc, LAYER_BASE, LAYER_SPAN, LayerRole, MAX_LAYERS, drawn_lines,
};
pub use element::transform3d;
pub use element::{
    Affine, AutoRepeat, Breadth, Cursor, CursorImage, Customize, Dismiss, DragAxis, DragSpec,
    DropSpec, Edges, FocusGroup, GestureSpec, GridFlow, GridLine, GridPlace, GridSpec, Interact,
    LayoutMode, LayoutStats, Lines, NavLinks, Orientation, Overlap, Patch, PinchSpec,
    PointerEvents, Position, Prop, Props, Query, SvgPointer, Template, Track, Transform,
    UserSelect, Values, Variant, Visibility,
};
pub use hud_layout::{HudEditInfo, HudEditorDesc, HudEditorResponse, HudPlacement};
pub use id::{Id, ui_id, ui_idi};
pub use input::{GamepadFrame, InputFrame, InputSource, Nav, OgInput, UiKey};
pub use juice::{
    BurstDesc, CounterDesc, ScreenTransitionDesc, ScreenWipe, ShakeDesc, Side, format_count,
};
pub use layer_stack::{LayerInfo, LayerOp};
pub use layout::anchor;
pub use layout::float::ImageAlpha;
pub use layout::{
    Align, Anchor, Bidi, Calc, CalcOp, Contain, ContentVisibility, Cross, Direction, Element,
    Family, FlexItem, FloatSide, Flow, FontRole, Insets, Length, Margin, Multicol, Rect, Size,
    TablePart, TextAlign, VerticalAlign,
};
pub use motion::{
    Choreo, Ease, EventKind, GestureInfo, Key, KeyEase, MotionDump, MotionToken, MotionTokens,
    PlayDesc, PlayInfo, PresenceSpec, Reduced, Repeat, SpringSpec, StaggerFrom, StateGroup,
    StateTransition, Timeline, TimelineEvent, TimelineTrack, Transition, TransitionKind, UiEvent,
    VisualState, cubic_bezier, ease_exit, ease_out,
};
pub use particles::{EmitFrom, ParticlesDesc};
pub use rich::{
    InlineAtom, RichTextDesc, RichTextResponse, TextSpan, TextSpanStyle, TextSpanValue,
};
pub use slot::{Aspect, SlotKind, SlotSpec};
pub use style::{
    Breakpoint, Contrast, Controls, Densities, Density, DensityMode, Durations, Palette, READABLE,
    Radius, Roles, Scheme, Space, Spring, Srgba, State, Theme, TypeRamp, TypeStep,
};
pub use style_file::{Art, Slice, SliceFill, StylePart, UiSound};
pub use text::text_path;
pub use text::{
    FontMetrics, Piece, UiFont, VisualGlyph, bidi_runs, needs_visual, pieces, pieces_with,
};
pub use text_path::{PathShape, TextPath};
pub use tokens::StyleTokens;
pub use ui::{
    ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, ButtonDesc, ButtonVariant, CjkLocale,
    ControlSize, FaceStyle, FamilyFace, FeedbackPlay, FieldDesc, FontFace, FontFamily, Fonts,
    HUD_MAX_ASPECT, IconAtlas, IconCell, ImageDesc, InlineImage, LabelDesc, NumberInputOptions,
    PanelDesc, ProgressDesc, Response, ScrollAxes, ScrollDesc, Synth, TextAreaDesc, TextTarget, Ui,
    UiDebugElement, UiDebugStyle, UiDesc, UiFrame, Viewport, debug_overlay,
};
pub use vcursor::VirtualCursorDesc;
pub use viewport::{ViewportDesc, ViewportResponse};
pub use widgets::{
    AnchorDesc, ArtDesc, CheckFieldDesc, ChoiceIndexResult, DetailsDesc, DetailsResult, DialogDesc,
    DpadDesc, DpadResponse, DropdownDesc, DropdownOption, DropdownResult, GaugeDesc, GridDesc,
    IconDesc, JoystickDesc, JoystickResponse, LabelSide, ListDesc, ListRange, MeterRegion,
    ModalDesc, ModalResponse, NumberResult, NumberStepperDesc, Presence, RadioGroupDesc,
    RadioOption, RadioVariant, SegmentItem, SegmentedDesc, SliderMark, SpotlightDesc, SwitchResult,
    ToastDesc, ToggleSwitchDesc, Tone, ValueFormat, ValueSliderDesc, ValueSliderResult,
    kept_options,
};
pub use world::{
    AnchorInfo, AnchorPoint, AnchorScale, Declutter, EdgeClamp, GroundQuad, MassHandle, MetaDesc,
    MetaKind, PanelInfo, WorldAnchorDesc, WorldCmd, WorldFrame, WorldOcclusion, WorldPanelDesc,
    WorldTarget, WorldView,
};

/// The probe's element type: what `dump_ui` lists and the id and text lookups search.
pub use og_probe::UiElement;
