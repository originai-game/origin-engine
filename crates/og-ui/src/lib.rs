//! Origin Engine UI (`docs/design.md` §2.7; the contract is `docs/ui.md`):
//! the tree that screen and component documents build and the engine's own
//! widgets add to, laid out once a frame, styled by one CSS cascade (the
//! default style, the design language the engine shares with origin-agent;
//! a style pack; the game's own CSS), moved by the tick, drawn through
//! og-render's UI pass (analytic shapes and MSDF text over the finished
//! frame, `paint`), and addressable by the probe through stable string ids.
//!
//! A game builds no tree in code: it mounts documents. The immediate calls
//! below are the engine's own front end, which the documents' controls, the
//! inspector and the tests use:
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

#[doc(hidden)]
pub mod a11y;
pub use interaction::actions;
pub use og_reactive::bind;
pub(crate) use paint::decor;
pub use style::css_color;
pub use text::colr;
pub use ui::scene_text::halo_contrast;
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
pub use og_text::text::add_segmentation;
pub use og_text::{glyphs, ink, shape};
pub use paint::svg;
pub use style::style_file;
mod text;
pub use interaction::vcursor;
pub use style::tokens;
#[rustfmt::skip]
pub mod theme;
pub use paint::particles;
mod ui;
#[doc(hidden)]
pub use layout::virtualize;
pub(crate) use motion::view_transition;
pub use world::viewport;
mod widgets;
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
pub use hud_layout::{HudEditInfo, HudPlacement};
pub use id::{Id, ui_id, ui_idi};
pub use input::{GamepadFrame, InputFrame, InputSource, Nav, OgInput, UiKey};
pub(crate) use juice::{BurstDesc, CounterDesc};
pub use juice::{ScreenTransitionDesc, ScreenWipe, Side, format_count};
pub use layer_stack::{LayerInfo, LayerOp};
pub use layout::anchor;
pub use layout::float::ImageAlpha;
pub use layout::{
    Align, Anchor, Bidi, Calc, CalcOp, Contain, ContentVisibility, Cross, Direction, Element,
    Family, FlexItem, FloatSide, Flow, FontRole, Insets, Length, Margin, Multicol, Rect, Size,
    SizeBasis, TablePart, TextAlign, VerticalAlign,
};
pub use motion::{
    Choreo, Ease, EventKind, GestureInfo, Key, KeyEase, MotionDump, MotionToken, MotionTokens,
    PlayDesc, PlayInfo, PresenceSpec, Reduced, Repeat, SpringSpec, StaggerFrom, StateGroup,
    StateTransition, Timeline, TimelineEvent, TimelineTrack, Transition, TransitionKind, UiEvent,
    VisualState, cubic_bezier, ease_exit, ease_out,
};
pub use particles::{EmitFrom, ParticlesDesc};
pub(crate) use rich::RichTextDesc;
pub use rich::{InlineAtom, TextSpan, TextSpanStyle, TextSpanValue};
pub use slot::{Aspect, SlotKind, SlotSpec};
pub use style::{
    Breakpoint, Contrast, Controls, Densities, Density, DensityMode, Durations, Palette, READABLE,
    Radius, Roles, Scheme, Seeds, Space, Spring, Srgba, State, Theme, TypeRamp, TypeStep,
};
pub use style_file::{Art, Slice, SliceFill, StylePart, UiSound};
pub use text::text_path;
pub use text::{
    FontMetrics, Piece, UiFont, VisualGlyph, bidi_runs, needs_visual, pieces, pieces_with,
};
pub use text_path::{PathShape, TextPath};
pub use tokens::StyleTokens;
pub use ui::highlight::{Highlight, HighlightRange};
pub use ui::{
    ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, CjkLocale, FaceStyle, FamilyFace, FeedbackPlay,
    FontFace, FontFamily, FontQuery, Fonts, HUD_MAX_ASPECT, IconAtlas, IconCell, InlineImage,
    Synth, Ui, UiDebugElement, UiDebugStyle, UiDesc, UiFrame, Viewport, debug_overlay,
};
pub(crate) use ui::{ControlSize, ImageDesc, LabelDesc, NumberInputOptions, Response};
pub use vcursor::VirtualCursorDesc;
pub(crate) use widgets::ListRange;
pub use widgets::{ToastDesc, Tone};
pub use world::{
    AnchorPoint, AnchorScale, Declutter, EdgeClamp, GroundQuad, MassHandle, MetaDesc, MetaKind,
    WorldCmd, WorldFrame, WorldOcclusion, WorldTarget, WorldView,
};

/// The probe's element type: what `dump_ui` lists and the id and text lookups search.
pub use og_probe::UiElement;
