use super::*;
#[cfg(test)]
pub(crate) use og_ui_next::fonts::FamilyFace;
pub(crate) use og_ui_next::fonts::{FontFamily, FontQuery, Fonts, Synth};
pub(crate) use og_ui_next::icons::{IconAtlas, IconCell, InlineImage};

/// How to create a [`Ui`]. The zero value is the dark theme at 60 ticks per second.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDesc {
    pub theme: Theme,
    /// The world's tick rate, for delays given in milliseconds (0: 60).
    pub ticks_per_second: u32,
    /// The fonts; without one, text is measured at 0.5 em per character and not drawn.
    pub fonts: Fonts,
}

impl Metrics for Fonts {
    fn metrics_for(&self, role: FontRole, text: &str) -> &FontMetrics {
        &self.for_text(role, text).metrics
    }

    fn metrics_styled(&self, role: FontRole, query: FontQuery, text: &str) -> &FontMetrics {
        &self.for_styled(role, query, text).0.metrics
    }
}

/// One laid-out frame: what the probe reads and what the renderer draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFrame {
    pub viewport: Viewport,
    /// Draw commands with their painter levels.
    pub items: Vec<DrawItem>,
    /// Problems in this frame's tree (duplicate ids, unclosed elements), each naming the fix.
    pub warnings: Vec<String>,
    /// Characters this frame's texts draw that their face lacks, in its
    /// atlas and in the file it renders from at run time: per face, in the
    /// order first drawn, each (face, character) once in the UI's life. The
    /// host names the face and warns ([`Fonts::face`] has its metrics).
    pub missing_glyphs: Vec<(og_draw::FontHandle, Vec<char>)>,
    /// The path of the control each pad's focus is on (pad 0: the shared
    /// focus), or empty.
    pub focus_paths: [String; 4],
    /// The focused text control, for the host: where an input method's
    /// window goes, and the selection a copy takes (`None`: no text focus).
    pub text_input: Option<TextTarget>,
    /// What motion did this frame: timeline events (sounds, haptics,
    /// callbacks), plays that ended, swipes, snaps and drops.
    pub events: Vec<UiEvent>,
    /// Elements built, rebuilt and laid out (docs/ui.md §2).
    pub stats: LayoutStats,
    /// The og-ui unit, CSS px per unit (docs/ui.md §6.1): `viewport`,
    /// `elements`, `a11y`, `text_input` and `overlaps` are in units (the
    /// game's coordinates); `items` are already in CSS px. A host that hands
    /// a rectangle to the page (an input method's window, the accessibility
    /// mirror, the probe) multiplies by it.
    pub unit: f32,
    /// The pointer's shape over what it is on (or holds), resolved
    /// ([`crate::Cursor`]; never `Auto`): the web shell sets the canvas's
    /// CSS cursor from it.
    pub cursor: crate::Cursor,
    /// The `cursor: url()` images declared with that keyword, tried in
    /// order before it (CSS UI 4 §5.1): a host shows the first that loads.
    pub cursor_images: Vec<crate::CursorImage>,
    /// Where this tick's grounds are seen: each world panel the UI lays
    /// onto the screen itself, no render target ([`crate::DrawCmd::EndLayer`]'s
    /// `ground`). A panel behind the camera has none and is not drawn.
    pub grounds: Vec<crate::GroundQuad>,
    /// Names its `items`, `viewport` and `unit`: a frame whose items are
    /// last frame's has its generation; any other has a new one, unique in
    /// the process. A host keeps what it made from a generation (the
    /// renderer's overlay) until it changes.
    pub generation: u64,
    /// A property transaction against exactly the preceding paint frame.
    pub(crate) properties: Option<crate::draw::PropertyUpdate>,
}

/// What [`Ui::edit_marks`] draws a text control's marks from.
pub(crate) struct EditMarks<'a> {
    pub m: &'a crate::text::FontMetrics,
    /// The control's text as shown.
    pub text: &'a str,
    /// Where its lines start across (a field's line its pen's start, an
    /// area's the left of its box), its first line box's top, and each
    /// line box's height.
    pub x: f32,
    pub top: f32,
    pub line_h: f32,
    pub size: f32,
    pub tracking: f32,
    /// The width an area's lines break at (none: a field's one line).
    pub width: Option<f32>,
    pub edit: &'a crate::layout::EditView,
    pub clip: Rect,
    pub alpha: f32,
    /// The selection (under the text), or the composition and caret (over it).
    pub under: bool,
}

/// The focused text control as a host sees it ([`UiFrame::text_input`]):
/// the browser shell puts its hidden text target over it (so an input
/// method's candidate window opens there) and copies `selection` on the
/// page's copy and cut.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextTarget {
    /// The element's path.
    pub path: String,
    /// Its rectangle, CSS px.
    pub rect: Rect,
    /// A multi-line text area.
    pub multiline: bool,
    /// The selected text (empty: none).
    pub selection: String,
    /// An input method is composing in it.
    pub composing: bool,
    /// Its HTML `type` (`FieldDesc::input_type`; an area's is text).
    pub input_type: crate::InputType,
    /// The virtual keyboard it asks for (its `inputmode`, else its type's).
    pub inputmode: crate::InputMode,
    /// Its autofill hint (`autocomplete`); empty: none.
    pub autocomplete: String,
    /// A password: `selection` is always empty, and the host neither
    /// copies nor cuts from it nor offers its text to anything else.
    pub secret: bool,
    /// Read-only (HTML `readonly`): the host's input is read-only too, so a
    /// phone shows no keyboard for it.
    pub readonly: bool,
}

impl UiFrame {
    /// The element at an id or `Parent/Child` path (topmost first), as `touch_move_to` resolves it.
    /// Painter levels used: the most sprite and text runs the frame can take.
    pub fn levels(&self) -> i32 {
        self.items.iter().map(|i| i.level + 1).max().unwrap_or(0)
    }

    /// Appends the frame's sprites, text and clip rectangles to an overlay
    /// (`RenderScene::overlay`), in CSS pixels: make it with the viewport's
    /// size and `dpr` (`og_draw::Overlay::new(vp.width, vp.height, vp.dpr)`).
    pub fn emit(&self, desc: &EmitDesc, overlay: &mut Overlay) {
        emit(&self.items, desc, overlay, &|_| None, &self.grounds);
    }

    /// Emits through a caller-owned retained display list. Flat shape
    /// paints share their emitted geometry and patch property-only ranges;
    /// unsupported paints and mixed writers use [`Self::emit`]. A missed
    /// generation rebuilds the list rather than applying an incomplete delta.
    pub fn emit_retained(
        &self,
        desc: &EmitDesc,
        overlay: &mut Overlay,
        cache: &mut crate::RetainedEmission,
    ) {
        cache.emit(self, desc, overlay);
    }

    /// [`UiFrame::emit`] with each anchored element ([`crate::WorldCmd::Anchor`])
    /// moved by `shift` of its key, CSS px: a host drawing between ticks
    /// moves a nameplate by how far its point's projection moved since the
    /// tick placed it, so it stays on its entity. `None` leaves it.
    pub fn emit_shifted(
        &self,
        desc: &EmitDesc,
        overlay: &mut Overlay,
        shift: &dyn Fn(u64) -> Option<og_core::glam::Vec2>,
    ) {
        // The shift is in units (the frame's viewport); the items in CSS px.
        let unit = self.unit.max(1.0);
        emit(
            &self.items,
            desc,
            overlay,
            &|k| shift(k).map(|d| d * unit),
            &self.grounds,
        );
    }
}

/// Button roles: how a style draws each.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// A clear lens: neutral fill (the default).
    #[default]
    Secondary,
    /// The primary action: one per screen.
    Primary,
    /// Text until hovered.
    Quiet,
    /// A destructive action: danger ink on the danger tint.
    Danger,
    /// An outlined button: clear inside a strong line until hovered, for
    /// a second action beside a primary that must still read as a button.
    Ghost,
}

/// Control heights from the density (`controlSm/md/lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonDesc {
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub disabled: bool,
    /// `None`: fit the label.
    pub width: Option<Size>,
    /// Take the focus when it appears (it was not on screen last frame)
    /// and no dialog is open: what opens takes the focus, so a title
    /// menu's first item answers Enter and the pad's A at once.
    pub autofocus: bool,
    /// A square of the control height with no side padding: an icon
    /// button, a one-glyph label (`+`). Overrides `width`.
    pub square: bool,
    /// Where its label sits across it (CSS `text-align`); `None`: centred,
    /// as HTML's user-agent sheet has a button.
    pub align: Option<TextAlign>,
    /// The part its style dresses it as (a document's `part`, such as
    /// `Loadout.item` with its qualifiers); `None`: `Button.base`. Its
    /// variant and states are qualifiers of that part, and its label's ink
    /// reads against that part's dressing.
    pub part: Option<crate::dress::PartRef>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabelDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// `None`: `ink`.
    pub color: Option<Srgba>,
    /// Scene text: `sceneInk` with the scheme's `sceneShadow` separation.
    pub on_scene: bool,
    /// Uppercase in the UI's locale, as a step with `uppercase` is.
    pub uppercase: bool,
    pub align: TextAlign,
    /// Keep on one line (break only at `\n`).
    pub nowrap: bool,
    pub width: Size,
    pub font: FontRole,
    /// A heading: a style with a flourish frames it.
    pub heading: bool,
    /// Cut with "…" where the text does not fit its width: on its one line
    /// (with `nowrap`; the label can then shrink in a row) or on its last
    /// line (`max_lines`).
    pub ellipsis: bool,
    /// The most lines a wrapping label shows (0: no limit); with `ellipsis`
    /// the last one ends in "…" when text is left out.
    pub max_lines: u32,
    /// Auto-fit: where the text does not fit its width at `step`, it takes
    /// the ramp's smaller steps down to this one (then its ellipsis, with
    /// `ellipsis`); `None` keeps its step.
    pub fit_min: Option<TypeStep>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelDesc {
    /// Lay children left to right (default: top to bottom).
    pub row: bool,
    pub width: Size,
    pub height: Size,
    /// `None`: `space.lg`.
    pub padding: Option<Insets>,
    /// `None`: `space.md`.
    pub gap: Option<f32>,
    pub align: Align,
    pub cross: Cross,
    /// `None`: `panel`.
    pub background: Option<Srgba>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollDesc {
    pub width: Size,
    pub height: Size,
    pub padding: Insets,
    pub gap: f32,
    /// Which ways it scrolls (vertical by default).
    pub axes: ScrollAxes,
    /// Draws no scrollbars.
    pub hide_bars: bool,
    /// How far (CSS px) the content fades into `fade_into` at an edge
    /// where more of it lies beyond; 0 draws no fade.
    pub fade: f32,
    /// The surface the content fades into; `None`: `panel`.
    pub fade_into: Option<Srgba>,
}

/// Which ways a scroll container scrolls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxes {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageDesc {
    /// `None` draws a solid rectangle of `tint`.
    pub texture: Option<TextureHandle>,
    /// `None`: the whole texture (`[u0, v0, u1, v1]`, v down).
    pub uv: Option<[f32; 4]>,
    /// `None`: white.
    pub tint: Option<Srgba>,
    pub width: Size,
    pub height: Size,
    /// Width over height (0: none): with the height left to fit, the image
    /// is as tall as its width over this (`Element::aspect`).
    pub aspect: f32,
    /// What a screen reader calls it (and the probe's text); empty is
    /// decoration.
    pub label: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressDesc {
    /// `None`: grow along its axis, its thickness across it.
    pub width: Option<Size>,
    /// `None`: as `width`, the other way.
    pub height: Option<Size>,
    pub axis: crate::widgets::ValueAxis,
}

/// Spinner behavior for an editable number field. Its raw string remains
/// the editing value; formatting affects only its unfocused presentation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NumberInputOptions {
    /// Wrap between min and the last valid step at or below max.
    pub wrap: bool,
    /// Shift-click step; zero uses ten ordinary steps.
    pub large_step: f64,
    pub format: crate::widgets::ValueFormat,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldDesc {
    /// Shown, dimmed, while the field is empty.
    pub placeholder: String,
    /// `None`: grow.
    pub width: Option<Size>,
    /// The most characters it takes (0: no limit).
    pub max_chars: u32,
    /// Its text is wrong (a form's error): `Field.frame[invalid]`, the
    /// danger outline an undressed frame takes.
    pub invalid: bool,
    /// No frame and no focus ring of its own, a small inset: a component
    /// frames it (an input with an icon and a clear button).
    pub bare: bool,
    /// No inset and the height of its text's line: the text sits exactly
    /// where the caller lays the field (in a box it draws, a line of a
    /// page). With or without `bare`.
    pub tight: bool,
    /// The component its style dresses it as (`Poem`, `Search[large]`):
    /// its frame `<component>.frame`, its text `<component>.text` and its
    /// placeholder `<component>.placeholder`, with its states (focused,
    /// invalid, disabled). `None`: `Field`.
    pub component: Option<String>,
    /// What it takes (HTML `type`, [`crate::InputType`]): a password is
    /// drawn masked with a reveal toggle (`<component>.reveal[revealed]`)
    /// and never copied or cut; a number takes only a number's
    /// characters; a number, email or url that is not one is invalid.
    pub input_type: crate::InputType,
    /// The virtual keyboard (HTML `inputmode`); `None`: the type's.
    pub inputmode: Option<crate::InputMode>,
    /// The page's autofill hint (HTML `autocomplete`: `username`,
    /// `current-password`, `one-time-code`, `off`…); empty: none.
    pub autocomplete: String,
    /// Its text is selected, copied and moved through but never edited
    /// (HTML `readonly`); it keeps its focus stop.
    pub readonly: bool,
    /// HTML `disabled`: no editing or focus stop; its parts match `:disabled`.
    pub disabled: bool,
    /// What else its text must be (`required`, `pattern`, `min`…): one
    /// failing makes it invalid as `invalid` does.
    pub constraints: crate::input_type::Constraints,
    /// Number spinner options; `None` uses nonwrapping unit-step behavior
    /// (or the step in `constraints`) without formatting the raw value.
    pub number: Option<NumberInputOptions>,
}

/// How a multi-line text area looks ([`Ui::text_area`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextAreaDesc {
    /// Shown, dimmed, while the area is empty.
    pub placeholder: String,
    /// `None`: grow.
    pub width: Option<Size>,
    /// Lines shown at once (0: 4); more scroll.
    pub rows: u32,
    /// The most characters it takes (0: no limit).
    pub max_chars: u32,
    /// Its text is wrong: `Field.frame[invalid]`.
    pub invalid: bool,
    /// No frame and no focus ring of its own: a component frames it.
    pub bare: bool,
    /// No inset and `rows` lines of its text's height ([`FieldDesc::tight`]).
    pub tight: bool,
    /// The component its style dresses it as ([`FieldDesc::component`]).
    pub component: Option<String>,
    /// Selected and copied, never edited ([`FieldDesc::readonly`]).
    pub readonly: bool,
    /// No editing or focus stop ([`FieldDesc::disabled`]).
    pub disabled: bool,
    /// What else its text must be ([`FieldDesc::constraints`]).
    pub constraints: crate::input_type::Constraints,
}
