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

mod a11y;
pub mod actions;
pub mod bind;
pub mod colr;
pub mod css_color;
pub(crate) mod decor;
pub mod doc;
mod draw;
mod draw_codec;
pub mod dress;
pub(crate) mod edit;
pub mod element;
pub mod glyphs;
pub mod hud_layout;
mod input_type;
pub use input_type::{InputMode, InputType};
mod id;
mod input;
mod juice;
pub mod layers;
mod layout;
pub mod markup;
pub mod motion;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_a11y;
mod nav;
pub mod paint;
pub mod paint_entry;
mod probe;
pub mod reactive;
mod recognize;
mod rich;
pub mod rive;
mod scroll;
pub use scroll::{Overscroll, ScrollCss, ScrollState};
pub mod shape;
pub mod slot;
mod style;
pub mod style_file;
pub mod style_gen;
pub mod svg;
mod text;
pub mod tokens;
pub mod vcursor;
#[rustfmt::skip]
pub mod theme;
pub mod particles;
mod ui;
mod view_transition;
pub mod viewport;
mod virtualize;
mod widgets;
pub mod world;

pub use a11y::{A11yNode, A11yTree, ANNOUNCEMENTS, Announcement, ROLES, Semantics};
pub use actions::{ActionDesc, ActionHint, ActionTrigger, Device, Glyph};
pub use doc::UiDocument;
pub use doc::build::{DocEvent, DocResponse};
pub use draw::{DrawCmd, DrawItem, EmitDesc, LAYER_BASE, LAYER_SPAN, MAX_LAYERS, drawn_lines};
pub use dress::{DressFrame, Dressing, Selector};
pub mod text_path;
pub mod transform3d;
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
    BurstDesc, CounterDesc, ScreenTransition, ScreenTransitionDesc, ScreenWipe, ShakeDesc, Side,
    format_count,
};
pub use layers::{LayerInfo, LayerOp};
pub use layout::anchor;
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
pub use probe::{SharedUi, UiProbe};
pub use rich::{
    InlineAtom, RichTextDesc, RichTextResponse, TextSpan, TextSpanStyle, TextSpanValue,
};
pub use slot::{Aspect, SlotKind, SlotSpec};
pub use style::{
    Breakpoint, Contrast, Density, DensityMode, Durations, Palette, READABLE, Radius, Roles,
    Scheme, Space, Spring, Srgba, Theme, TypeRamp, TypeStep,
};
pub use style_file::{
    Art, Slice, SliceFill, StyleFile, StylePart, UiSound, default_dressings, migrate_style,
    parse_style, read_style_doc, upgrade_style, write_style_doc,
};
pub use text::{
    FontMetrics, Piece, UiFont, VisualGlyph, bidi_runs, needs_visual, pieces, pieces_with,
};
pub use text_path::{PathShape, TextPath};
pub use tokens::StyleTokens;
pub use ui::{
    ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, ButtonDesc, ButtonVariant, CjkLocale,
    ControlSize, FaceStyle, FamilyFace, FeedbackPlay, FieldDesc, FontFace, Fonts, HUD_MAX_ASPECT,
    IconAtlas, IconCell, ImageDesc, InlineImage, LabelDesc, PanelDesc, ProgressDesc, Response,
    ScrollAxes, ScrollDesc, Synth, TextAreaDesc, TextTarget, Ui, UiDebugElement, UiDebugStyle,
    UiDesc, UiFrame, Viewport, debug_overlay,
};
pub use vcursor::VirtualCursorDesc;
pub use viewport::{ViewportDesc, ViewportResponse};
pub use widgets::{
    ActionButtonDesc, AnchorDesc, ArtDesc, CheckFieldDesc, ChipDesc, ChoiceIndexResult,
    CooldownDesc, DetailsDesc, DetailsResult, DialogDesc, DialogueDesc, DialogueResponse, DpadDesc,
    DpadResponse, DropdownDesc, DropdownOption, DropdownResult, GaugeDesc, GridDesc, IconDesc,
    JoystickDesc, JoystickResponse, LabelSide, ListDesc, ListRange, MapMarker, MeterDesc,
    MeterKind, MeterRegion, MinimapDesc, ModalDesc, ModalResponse, NumberResult, NumberStepperDesc,
    Presence, RadioGroupDesc, RadioOption, RadioVariant, SegmentItem, SegmentedDesc, SkeletonDesc,
    SliderMark, SpotlightDesc, SwitchResult, ToastDesc, ToggleSwitchDesc, Tone, TransitionDesc,
    ValueFormat, ValueSliderDesc, ValueSliderResult, ZoomResponse, ZoomViewDesc, kept_options,
};
pub use world::{
    AnchorInfo, AnchorPoint, AnchorScale, Declutter, EdgeClamp, GroundQuad, MassHandle, MetaDesc,
    MetaKind, PanelInfo, WorldAnchorDesc, WorldCmd, WorldFrame, WorldOcclusion, WorldPanelDesc,
    WorldTarget, WorldView,
};

/// The probe's element type: what `dump_ui` lists and the id and text lookups search.
pub use og_probe::UiElement;
