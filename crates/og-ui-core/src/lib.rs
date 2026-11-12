//! og-ui's core (docs/ui.md §7): the immediate-mode toolkit's stages and the
//! state they keep, from the frame's input to its draw list. The documents
//! (og-ui) build into it through [`doc::build::Documents`].

/// The engine's default CSS, generated from the shared design tokens.
pub const DEFAULT_CSS: &str = include_str!("../default.css");

#[doc(hidden)]
pub mod a11y;
pub mod actions;
pub use og_reactive::bind;
pub mod colr;
pub mod css_color;
pub(crate) mod decor;
pub mod doc;
mod draw;
mod draw_codec;
pub mod dress;
pub(crate) mod edit;
pub mod element;
pub mod hud_layout;
#[doc(hidden)]
pub mod input_type;
pub use input_type::{InputMode, InputType};
#[doc(hidden)]
pub mod id;
#[doc(hidden)]
pub mod input;
mod juice;
#[doc(hidden)]
pub mod layer_stack;
#[doc(hidden)]
pub mod layout;
pub mod markup;
pub mod motion;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_a11y;
mod nav;
pub mod paint;
pub mod paint_entry;
pub use og_reactive::reactive;
mod recognize;
#[doc(hidden)]
pub mod rich;
pub use og_rive as rive;
mod scroll;
pub use scroll::{Overscroll, ScrollCss, ScrollState};
pub mod slot;
pub mod sound_css;
#[doc(hidden)]
pub mod style;
pub mod style_file;
pub mod svg;
pub use og_text::{glyphs, shape};
mod text;
pub mod tokens;
pub mod vcursor;
#[rustfmt::skip]
pub mod theme;
pub mod particles;
#[doc(hidden)]
pub mod ui;
mod view_transition;
pub mod viewport;
#[doc(hidden)]
pub mod virtualize;
#[doc(hidden)]
pub mod widgets;
pub mod world;

pub use a11y::{A11yNode, A11yTree, ANNOUNCEMENTS, Announcement, ROLES, Semantics};
pub use actions::{ActionDesc, ActionHint, ActionTrigger, Device, Glyph};
pub use doc::UiDocument;
pub use doc::build::{DocEvent, DocResponse};
pub use draw::{
    DrawCmd, DrawItem, EmitDesc, LAYER_BASE, LAYER_SPAN, LayerRole, MAX_LAYERS, drawn_lines,
};
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
    Breakpoint, Contrast, Densities, Density, DensityMode, Durations, Palette, READABLE, Radius,
    Roles, Scheme, Space, Spring, Srgba, State, Theme, TypeRamp, TypeStep,
};
pub use style_file::{Art, Slice, SliceFill, StylePart, UiSound};
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
