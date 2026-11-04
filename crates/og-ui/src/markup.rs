//! Rich text markup: inline spans in one paragraph, written in the string
//! table so a translation carries its own emphasis.
//!
//! ```text
//! Press [b]Jump[/b] to clear the [color=danger]gap[/color], then
//! [size=lg]run[/size]! Costs [icon=coin] [font=mono]250[/font].
//! [link=shop]Open the shop[/link] · [wave]wavy[/wave] [shake]angry[/shake] [pulse]alive[/pulse]
//! [bounce]hop[/bounce] [reveal]Level up![/reveal]
//! [i]whispered[/i], [u]marked[/u], [s]struck[/s], [outline=2,shadow]loud[/outline],
//! [img=portrait] and Press [action=jump] to jump.
//! ```
//!
//! | Tag | Does |
//! |---|---|
//! | `[b]…[/b]` | bold: the glyphs thickened in the MSDF shader (weight) |
//! | `[i]…[/i]` | italic: the glyphs slanted (an oblique of the face) |
//! | `[u]…[/u]`, `[s]…[/s]` | underlined, struck through: a hairline in the span's ink |
//! | `[outline=<w>[,<colour>]]` | an outline `w` CSS px wide from the font's distance field, in a colour role or `#rrggbb[aa]` (`shadow` by default) |
//! | `[img=<name>]` | a picture the game registered by name (`UiDesc.images`), one em tall at its aspect, inline (self-closing) |
//! | `[action=<name>]` | the action's glyph on the device used last (its icon, else its key's name on a cap), resolved every frame, inline (self-closing) |
//! | `[color=<ThemeColor>]` / `[color=#rrggbb[aa]]` | the ink: a token (follows the style) or a colour |
//! | `[size=<TypeStepName>]` | the type step (`lg`, `xl`, `meta`, `displaySm`…) |
//! | `[font=body\|display\|mono]` | the font role |
//! | `[icon=<name>]` | an icon of the icon atlas, one em, inline (self-closing) |
//! | `[link=<id>]…[/link]` | a link: accent ink, underlined, focusable; its id is a child element and the answer of a click |
//! | `[wave]`, `[shake]`, `[pulse]`, `[bounce]` | per-glyph motion by the tick (still under reduced motion) |
//! | `[reveal]` | its glyphs rise and fade in one after another from when the paragraph first showed (all at once under reduced motion) |
//!
//! A closing tag closes the innermost open tag of its name; `[[` is a
//! literal `[`. An unknown tag, a bad value or an unclosed tag is kept as
//! text and named in a frame warning, so a typo in a translation shows.

/// A span's per-glyph motion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpanFx {
    #[default]
    None,
    Wave,
    Shake,
    Pulse,
    /// Each glyph hops off the baseline in turn, a wave of hops.
    Bounce,
    /// Each glyph rises and fades in, one after another, once.
    Reveal,
}

/// One run of text (or an icon) in one style.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkupSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    /// `[outline=…]`'s value: a width, then optionally `,` and a colour.
    pub outline: Option<String>,
    /// A ThemeColor name or `#rrggbb[aa]`; `None`: the paragraph's ink.
    pub color: Option<String>,
    /// A TypeStepName; `None`: the paragraph's step.
    pub size: Option<String>,
    /// `body`, `display` or `mono`; `None`: the paragraph's.
    pub font: Option<String>,
    /// An inline icon (the span's text is empty).
    pub icon: Option<String>,
    /// An inline picture registered by name (the span's text is empty).
    pub image: Option<String>,
    /// An action's glyph (the span's text is empty).
    pub action: Option<String>,
    pub link: Option<String>,
    pub fx: SpanFx,
}

#[derive(Clone, Debug, PartialEq)]
enum Tag {
    B,
    I,
    U,
    S,
    Outline(String),
    Color(String),
    Size(String),
    Font(String),
    Link(String),
    Fx(SpanFx),
}

impl Tag {
    fn name(&self) -> &'static str {
        match self {
            Tag::B => "b",
            Tag::I => "i",
            Tag::U => "u",
            Tag::S => "s",
            Tag::Outline(_) => "outline",
            Tag::Color(_) => "color",
            Tag::Size(_) => "size",
            Tag::Font(_) => "font",
            Tag::Link(_) => "link",
            Tag::Fx(SpanFx::Wave) => "wave",
            Tag::Fx(SpanFx::Shake) => "shake",
            Tag::Fx(SpanFx::Bounce) => "bounce",
            Tag::Fx(SpanFx::Reveal) => "reveal",
            Tag::Fx(_) => "pulse",
        }
    }
}

/// Parses markup into spans and the warnings of what it kept as text.
pub fn parse_markup(markup: &str) -> (Vec<MarkupSpan>, Vec<String>) {
    let mut spans: Vec<MarkupSpan> = Vec::new();
    let mut warnings = Vec::new();
    let mut stack: Vec<Tag> = Vec::new();
    let mut text = String::new();
    let style = |stack: &[Tag]| {
        let mut s = MarkupSpan::default();
        for t in stack {
            match t {
                Tag::B => s.bold = true,
                Tag::I => s.italic = true,
                Tag::U => s.underline = true,
                Tag::S => s.strike = true,
                Tag::Outline(o) => s.outline = Some(o.clone()),
                Tag::Color(c) => s.color = Some(c.clone()),
                Tag::Size(z) => s.size = Some(z.clone()),
                Tag::Font(f) => s.font = Some(f.clone()),
                Tag::Link(l) => s.link = Some(l.clone()),
                Tag::Fx(f) => s.fx = *f,
            }
        }
        s
    };
    let flush = |spans: &mut Vec<MarkupSpan>, text: &mut String, stack: &[Tag]| {
        if !text.is_empty() {
            let mut s = style(stack);
            s.text = std::mem::take(text);
            spans.push(s);
        }
    };
    let mut rest = markup;
    while let Some(at) = rest.find('[') {
        text.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        if let Some(stripped) = after.strip_prefix('[') {
            text.push('[');
            rest = stripped;
            continue;
        }
        let Some(end) = after.find(']') else {
            warnings.push(format!(
                "rich text: '[' without ']' in \"{markup}\" (write [[ for a bracket)"
            ));
            text.push('[');
            rest = after;
            continue;
        };
        let raw = &after[..end];
        let (name, value) = match raw.split_once('=') {
            Some((n, v)) => (n.trim(), Some(v.trim())),
            None => (raw.trim(), None),
        };
        rest = &after[end + 1..];
        if let Some(closing) = name.strip_prefix('/') {
            match stack.iter().rposition(|t| t.name() == closing) {
                Some(k) => {
                    flush(&mut spans, &mut text, &stack);
                    stack.remove(k);
                }
                None => {
                    warnings.push(format!(
                        "rich text: [/{closing}] closes nothing in \"{markup}\""
                    ));
                    text.push_str(&format!("[{raw}]"));
                }
            }
            continue;
        }
        let tag = match (name, value) {
            ("b", None) => Some(Tag::B),
            ("i", None) => Some(Tag::I),
            ("u", None) => Some(Tag::U),
            ("s", None) => Some(Tag::S),
            ("outline", Some(v)) if !v.is_empty() => Some(Tag::Outline(v.to_owned())),
            ("color", Some(v)) if !v.is_empty() => Some(Tag::Color(v.to_owned())),
            ("size", Some(v)) if !v.is_empty() => Some(Tag::Size(v.to_owned())),
            ("font", Some(v @ ("body" | "display" | "mono"))) => Some(Tag::Font(v.to_owned())),
            ("link", Some(v)) if !v.is_empty() && !v.contains('/') => Some(Tag::Link(v.to_owned())),
            ("wave", None) => Some(Tag::Fx(SpanFx::Wave)),
            ("shake", None) => Some(Tag::Fx(SpanFx::Shake)),
            ("pulse", None) => Some(Tag::Fx(SpanFx::Pulse)),
            ("bounce", None) => Some(Tag::Fx(SpanFx::Bounce)),
            ("reveal", None) => Some(Tag::Fx(SpanFx::Reveal)),
            (kind @ ("icon" | "img" | "action"), Some(v)) if !v.is_empty() => {
                flush(&mut spans, &mut text, &stack);
                let mut s = style(&stack);
                let v = Some(v.to_owned());
                match kind {
                    "icon" => s.icon = v,
                    "img" => s.image = v,
                    _ => s.action = v,
                }
                spans.push(s);
                continue;
            }
            _ => None,
        };
        match tag {
            Some(t) => {
                flush(&mut spans, &mut text, &stack);
                stack.push(t);
            }
            None => {
                warnings.push(format!(
                    "rich text: [{raw}] is not a tag (b, i, u, s, outline=, color=, size=, font=, icon=, img=, action=, link=, wave, shake, pulse, bounce, reveal) in \"{markup}\""
                ));
                text.push_str(&format!("[{raw}]"));
            }
        }
    }
    text.push_str(rest);
    flush(&mut spans, &mut text, &stack);
    if let Some(open) = stack.last() {
        warnings.push(format!(
            "rich text: [{}] is never closed in \"{markup}\"",
            open.name()
        ));
    }
    (spans, warnings)
}

/// The text a markup shows (what the probe and a screen reader read).
pub fn plain_text(spans: &[MarkupSpan]) -> String {
    spans.iter().map(|s| s.text.as_str()).collect()
}
