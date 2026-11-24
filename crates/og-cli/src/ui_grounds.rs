//! The grounds and inks a game's TSX writes by hand (crates/og-ui-next/
//! CONVENTIONS.md §Theme and packs, Grounds and inks): text sits on a pack
//! slot (`packClass`) or on an opaque theme ground role, and its ink is
//! that ground's pair. `og check` reports each break of that rule as
//! warning OG9108 at the class's line and column, naming what fixes it:
//!
//! - an ink with an opacity modifier (`text-fg/66`): the muted roles;
//! - a literal `white` or `black` ground or ink (`bg-white/8`): a role;
//! - an arbitrary colour (`bg-[#1b1d24]`): a role;
//! - a ground under text less than nearly opaque (`bg-well/82`,
//!   `from-entry-lo`, `bg-card`): a slot or an opaque role. An element's
//!   ground is under text when the element is a `<Text>` or `<Input>`, or
//!   holds one (or literal text) that no nearer element with a ground of
//!   its own covers.
//!
//! Parsed (oxc), not scanned: a class reaches an element through its
//! `class` attribute's literals, templates, conditionals and the
//! module-scope constants they name, so a constant shared by several
//! elements is reported once, where it is written.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild,
    JSXElement, JSXElementName, StringLiteral, TemplateLiteral, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use crate::ui_values::Scale;

/// The check's code in `og check`'s warnings.
pub const CODE: u32 = 9108;

/// The code of a child's ink on an art slot (OG9107, the pack ground-ink
/// rule carried to the text inside a slot).
pub const ART_INK: u32 = 9107;

/// A theme's art slots as grounds: what OG9107 checks a child's ink on.
#[derive(Clone, Debug, Default)]
pub struct ArtGrounds {
    /// Each scheme's colours by name (`Scheme::ALL` order).
    pub colors: [BTreeMap<String, String>; 2],
    /// Each art slot (`Button.plate.primary`): its scheme and the faces of
    /// its art.
    pub slots: BTreeMap<String, (usize, Vec<String>)>,
}

impl ArtGrounds {
    /// The art slots of a compiled theme, over the default pack.
    pub fn of(theme: &og_ui_next::theme::Theme) -> Result<ArtGrounds, String> {
        use og_ui_next::pack::{Layer, Pack, Scheme};
        let pack = Pack::new(&[Layer::default_pack(), theme.layer(Default::default())?]);
        let colors = Scheme::ALL.map(|s| {
            serde_json::from_str::<serde_json::Value>(&pack.theme_json(s))
                .ok()
                .and_then(|v| {
                    Some(
                        v.get("colors")?
                            .as_object()?
                            .iter()
                            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
                            .collect(),
                    )
                })
                .unwrap_or_default()
        });
        let mut slots = BTreeMap::new();
        for (key, class) in pack.variants() {
            let (images, rest) = og_ui_next::pack::split_nine(class);
            let Some(image) = images.first() else {
                continue;
            };
            let role = match *image {
                "button-primary" => "primary",
                "button-danger" => "danger",
                i => i,
            };
            let Some(faces) = theme.art["palette"][role].as_array() else {
                continue;
            };
            let scheme = usize::from(!rest.split_whitespace().any(|c| c == "scheme-light"));
            let faces = faces
                .iter()
                .filter_map(|f| f.as_str().map(str::to_owned))
                .collect();
            slots.insert(key.clone(), (scheme, faces));
        }
        Ok(ArtGrounds { colors, slots })
    }
}

/// The slot keys a `packClass(component, slot, variant?)` call can give:
/// a literal variant, each literal branch of a conditional one (a branch
/// that is no string gives the bare slot), or none.
fn slot_keys(c: &oxc_ast::ast::CallExpression) -> Vec<String> {
    let lit = |a: Option<&oxc_ast::ast::Argument>| match a.and_then(|a| a.as_expression()) {
        Some(Expression::StringLiteral(s)) => Some(s.value.display().to_string()),
        _ => None,
    };
    let (Some(component), Some(slot)) = (lit(c.arguments.first()), lit(c.arguments.get(1))) else {
        return Vec::new();
    };
    let base = format!("{component}.{slot}");
    let variants: Vec<Option<String>> = match c.arguments.get(2).and_then(|a| a.as_expression()) {
        None => vec![None],
        Some(Expression::StringLiteral(s)) => vec![Some(s.value.display().to_string())],
        Some(Expression::ConditionalExpression(x)) => [&x.consequent, &x.alternate]
            .into_iter()
            .map(|e| match e {
                Expression::StringLiteral(s) => Some(s.value.display().to_string()),
                _ => None,
            })
            .collect(),
        Some(_) => vec![None],
    };
    variants
        .into_iter()
        .map(|v| v.map_or(base.clone(), |v| format!("{base}.{v}")))
        .collect()
}

/// The least opacity a ground under text has, in either scheme: below it,
/// the scene behind shows through enough to move the label's contrast.
const OPAQUE: f32 = 0.85;

/// One break of the rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// [`CODE`], or [`crate::ui_values::CODE`] for a hand-written value.
    pub code: u32,
    pub line: u32,
    pub col: u32,
    pub class: String,
    pub message: String,
    /// The class or classes a hand-written value's finding names, and
    /// whether they draw the same value.
    pub fix: Option<(String, bool)>,
}

/// The theme's colours by name: the least alpha the colour has over both
/// schemes (1 for an opaque one).
pub type Colors = BTreeMap<String, f32>;

/// The colours of `theme_json`s (one per scheme, as `Pack::theme_json`
/// gives them), each with its least alpha.
pub fn colors(themes: &[String]) -> Colors {
    let mut out = Colors::new();
    for t in themes {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(t) else {
            continue;
        };
        let Some(map) = v.get("colors").and_then(|c| c.as_object()) else {
            continue;
        };
        for (name, value) in map {
            let Some(hex) = value.as_str() else { continue };
            let a = alpha(hex);
            let e = out.entry(name.clone()).or_insert(1.0);
            *e = e.min(a);
        }
    }
    out
}

/// A `#rrggbb[aa]` colour's alpha (1 when it has none).
fn alpha(hex: &str) -> f32 {
    let h = hex.trim_start_matches('#');
    if h.len() == 8 {
        u8::from_str_radix(&h[6..], 16).map_or(1.0, |a| a as f32 / 255.0)
    } else {
        1.0
    }
}

/// The findings in one TSX source.
pub fn scan(
    path: &Path,
    text: &str,
    colors: &Colors,
    scale: &Scale,
    art: Option<&ArtGrounds>,
) -> Vec<Finding> {
    let Ok(source_type) = SourceType::from_path(path) else {
        return Vec::new();
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
    let mut names = Names {
        consts: HashMap::new(),
    };
    names.visit_program(&parsed.program);
    let mut v = Scan {
        text,
        colors,
        scale,
        consts: names.consts,
        art,
        frames: Vec::new(),
        found: BTreeMap::new(),
    };
    v.visit_program(&parsed.program);
    v.found.into_values().collect()
}

/// Every variable's initial value by name (a component's local `wash` as
/// well as a module's `CARD`); a shadowing name keeps the first.
struct Names<'p, 'a> {
    consts: HashMap<&'p str, &'p Expression<'a>>,
}

impl<'p, 'a> Visit<'a> for Names<'p, 'a> {
    fn visit_variable_declarator(&mut self, d: &VariableDeclarator<'a>) {
        // SAFETY: the visitor walks a program borrowed for 'p, so every
        // node it is handed lives as long; `Visit` only does not say so.
        let d: &'p VariableDeclarator<'a> = unsafe { &*(d as *const VariableDeclarator<'a>) };
        if let (BindingPattern::BindingIdentifier(b), Some(init)) = (&d.id, &d.init) {
            self.consts.entry(b.name.as_str()).or_insert(init);
        }
        walk::walk_variable_declarator(self, d);
    }
}

/// An element being walked: its own class pieces, and whether text is
/// drawn on its ground.
struct Frame {
    tokens: Vec<(String, u32)>,
    ground: bool,
    text: bool,
}

struct Scan<'s, 'p, 'a> {
    text: &'s str,
    colors: &'s Colors,
    scale: &'s Scale,
    consts: HashMap<&'p str, &'p Expression<'a>>,
    art: Option<&'s ArtGrounds>,
    frames: Vec<Frame>,
    /// By byte offset, so a constant shared by several elements is one finding.
    found: BTreeMap<u32, Finding>,
}

/// Host elements that draw text.
const TEXT_TAGS: [&str; 2] = ["Text", "Input"];
/// Control flow that renders its children in its own place.
const WRAPPERS: [&str; 5] = ["Show", "For", "Match", "Switch", "Index"];

impl<'p, 'a> Scan<'_, 'p, 'a> {
    fn position(&self, offset: u32) -> (u32, u32) {
        let before = &self.text[..(offset as usize).min(self.text.len())];
        let line = before.matches('\n').count() as u32 + 1;
        let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) as u32 + 1;
        (line, col)
    }

    fn report(&mut self, class: &str, offset: u32, message: String) {
        let (line, col) = self.position(offset);
        // A ground or an ink's finding stands over a hand-written
        // value's at the same class.
        self.found.insert(
            offset,
            Finding {
                code: CODE,
                line,
                col,
                class: class.to_owned(),
                message,
                fix: None,
            },
        );
    }

    fn report_value(&mut self, class: &str, offset: u32, fix: crate::ui_values::Fix) {
        let (line, col) = self.position(offset);
        self.found.entry(offset).or_insert(Finding {
            code: crate::ui_values::CODE,
            line,
            col,
            class: class.to_owned(),
            message: fix.message,
            fix: (!fix.classes.is_empty()).then_some((fix.classes, fix.exact)),
        });
    }

    /// The `(class, offset)` tokens an element's class expression reaches.
    fn tokens(&self, e: &Expression<'a>, depth: u32, out: &mut Vec<(String, u32)>) {
        if depth > 8 {
            return;
        }
        match e {
            Expression::StringLiteral(s) => {
                split(&s.value.display().to_string(), s.span.start + 1, out)
            }
            Expression::TemplateLiteral(t) => {
                for q in &t.quasis {
                    split(&q.value.raw, q.span.start, out);
                }
                for e in &t.expressions {
                    self.tokens(e, depth + 1, out);
                }
            }
            Expression::ConditionalExpression(c) => {
                self.tokens(&c.consequent, depth + 1, out);
                self.tokens(&c.alternate, depth + 1, out);
            }
            Expression::LogicalExpression(l) => self.tokens(&l.right, depth + 1, out),
            Expression::ParenthesizedExpression(p) => self.tokens(&p.expression, depth + 1, out),
            Expression::Identifier(i) => {
                if let Some(init) = self.consts.get(i.name.as_str()) {
                    self.tokens(init, depth + 1, out);
                }
            }
            Expression::CallExpression(c) => match &c.callee {
                Expression::Identifier(i) if i.name == "packClass" => {
                    out.push(("@slot".to_owned(), c.span.start));
                    for key in slot_keys(c) {
                        out.push((format!("@slot:{key}"), c.span.start));
                    }
                }
                // A local class function (`row()`): its body.
                callee => self.tokens(callee, depth + 1, out),
            },
            Expression::ArrowFunctionExpression(f) => {
                if let Some(body) = f.get_expression() {
                    self.tokens(body, depth + 1, out);
                }
            }
            _ => {}
        }
    }

    /// OG9107 for a child: the text just pushed, when the nearest ground
    /// under it is an art slot (a slot whose art is its ground and whose
    /// ink the theme solved on that art). An ink a child sets, or the slot
    /// element's own state ink (which lies over the slot's), replaces the
    /// slot's ink, so each is held to 4.5:1 on the art's faces.
    fn check_art_ink(&mut self) {
        let Some(art) = self.art else { return };
        let n = self.frames.len();
        for i in (0..n).rev() {
            let f = &self.frames[i];
            let slots: Vec<(&str, &(usize, Vec<String>))> = f
                .tokens
                .iter()
                .filter_map(|(c, _)| c.strip_prefix("@slot:"))
                .filter_map(|k| art.slots.get_key_value(k).map(|(k, v)| (k.as_str(), v)))
                .collect();
            if slots.is_empty() {
                if f.ground {
                    return;
                }
                continue;
            }
            let own = f.tokens.iter().filter(|(c, _)| c.contains(':'));
            let inner = self.frames[i + 1..].iter().flat_map(|g| g.tokens.iter());
            let near: Vec<&(String, u32)> = own.chain(inner).collect();
            // A state that sets its own art or shape (`data-on:nine-none`,
            // `data-on:nine-chip-cut`) replaces the slot's, so the state's
            // own fill is the ground its `-fg` ink pairs with.
            let dropped = |prefix: &str| {
                let own = format!("{prefix}nine-");
                self.frames[i..]
                    .iter()
                    .any(|g| g.tokens.iter().any(|(c, _)| c.starts_with(&own)))
            };
            let mut low = Vec::new();
            for (class, at) in near.iter().copied() {
                let Some(("text", colour, _)) = colour_class(class) else {
                    continue;
                };
                let prefix = class.rsplit_once(':').map_or("", |(p, _)| p);
                if !prefix.is_empty() && dropped(&format!("{prefix}:")) {
                    continue;
                }
                let ratio = og_ui_next::theme::INKS
                    .iter()
                    .find(|(r, _)| *r == colour)
                    .map_or(4.5, |(_, r)| *r);
                for (key, (scheme, faces)) in &slots {
                    let Some(ink) = art.colors[*scheme].get(colour) else {
                        continue;
                    };
                    let r = faces
                        .iter()
                        .map(|f| og_ui_next::theme::contrast(ink, f))
                        .fold(f32::MAX, f32::min);
                    if r < ratio {
                        low.push((class.clone(), *at, key.to_string(), r, ratio));
                        break;
                    }
                }
            }
            for (class, at, key, r, ratio) in low {
                let (line, col) = self.position(at);
                self.found.insert(
                    at,
                    Finding {
                        code: ART_INK,
                        line,
                        col,
                        class: class.clone(),
                        message: format!(
                            "`{class}` sets the ink on {key}'s art at {r:.2}:1 (under {ratio}:1): the theme solved the slot's own ink on that art, so leave the ink to the slot; a state that paints its own fill sets its own shape with it (`data-on:nine-chip-cut`, or `data-on:nine-none`)"
                        ),
                        fix: None,
                    },
                );
            }
            return;
        }
    }

    /// The rules any class string keeps, wherever it is written.
    fn check_string(&mut self, text: &str, offset: u32) {
        let mut tokens = Vec::new();
        split(text, offset, &mut tokens);
        for (class, at) in tokens {
            if let Some(message) = self.anywhere(&class) {
                self.report(&class, at, message);
            } else if let Some(fix) = crate::ui_values::hand_written(&class, self.scale) {
                self.report_value(&class, at, fix);
            }
        }
    }

    fn anywhere(&self, class: &str) -> Option<String> {
        let (prop, colour, modifier) = colour_class(class)?;
        if colour.starts_with('[') {
            return (colour.contains('#') || colour.contains("rgb") || colour.contains("hsl"))
                .then(|| format!("`{class}` writes a colour by hand: use a theme role (`og api show screens` §5), so a pack restyles it"));
        }
        if colour == "white" || colour == "black" {
            let role = literal_role(prop, colour, modifier);
            return Some(format!(
                "`{class}` is a literal {colour}, the same in every pack and scheme: use {role}"
            ));
        }
        if prop == "text" && modifier.is_some() && self.colors.contains_key(colour) {
            let fix = match (colour, modifier) {
                ("fg", _) => "`text-fg-muted` (`text-fg-subtle` only for a disabled line: it holds 3:1, not 4.5:1)".to_owned(),
                _ => format!("`text-{colour}` (or `text-fg-muted` for a quieter line)"),
            };
            return Some(format!(
                "`{class}` thins the ink, so it no longer holds its ground's 4.5:1: use {fix}"
            ));
        }
        None
    }

    /// A ground under text: the element's colour classes that leave it
    /// less than nearly opaque.
    fn check_ground(&mut self, tokens: &[(String, u32)]) {
        for (class, at) in tokens {
            let Some((prop, colour, modifier)) = colour_class(class) else {
                continue;
            };
            if prop == "text" {
                continue;
            }
            let Some(a) = self.colors.get(colour) else {
                continue;
            };
            let a = a * modifier.map_or(1.0, |m| m as f32 / 100.0);
            if a >= OPAQUE {
                continue;
            }
            let fix = match modifier {
                Some(_) if self.colors[colour] >= OPAQUE => format!(
                    "the slot's state variant for a tinted state (`packClass(\"Row\", \"frame\", \"selected\")`), or the whole `{prop}-{colour}` with its `-fg` ink"
                ),
                _ => "a slot (`packClass(\"Row\", \"frame\")`, `packClass(\"Plate\", \"frame\")`) or an opaque role (`bg-surface`, `bg-raised-lo`, `bg-glass`)".to_owned(),
            };
            self.report(
                class,
                *at,
                format!(
                    "`{class}` is a ground under text at {}% opacity: the scene shows through and moves the label's contrast; use {fix}",
                    (a * 100.0).round() as u32
                ),
            );
        }
    }
}

/// Splits a class string at byte `offset` into its classes with theirs.
fn split(text: &str, offset: u32, out: &mut Vec<(String, u32)>) {
    let mut at = 0;
    for part in text.split(char::is_whitespace) {
        if !part.is_empty() {
            out.push((part.to_owned(), offset + at as u32));
        }
        at += part.len() + 1;
    }
}

/// A colour class's property (`bg`, `from`, `via`, `to`, `text`), colour
/// name and opacity modifier, under any variants.
fn colour_class(class: &str) -> Option<(&str, &str, Option<u32>)> {
    // The utility after the variants, outside brackets.
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in class.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            ':' if depth == 0 => start = i + 1,
            _ => {}
        }
    }
    let utility = class[start..].trim_start_matches('!');
    let (prop, rest) = ["bg-", "from-", "via-", "to-", "text-"]
        .iter()
        .find_map(|p| utility.strip_prefix(p).map(|r| (&p[..p.len() - 1], r)))?;
    if rest.starts_with('[') {
        return Some((prop, rest, None));
    }
    let (colour, modifier) = match rest.rsplit_once('/') {
        Some((c, m)) => (c, m.parse().ok()),
        None => (rest, None),
    };
    Some((prop, colour, modifier))
}

/// The role a literal white or black stands for.
fn literal_role(prop: &str, colour: &str, modifier: Option<u32>) -> String {
    match (prop, colour, modifier.unwrap_or(100)) {
        ("text", "white", _) => "`text-fg` (or the ground's own `-fg` pair)".to_owned(),
        ("text", _, _) => {
            "the ground's own `-fg` pair (`text-accent-fg`, `text-gold-fg`)".to_owned()
        }
        (_, "white", n) if n <= 6 => {
            format!("`{prop}-lit-soft` for a highlight away from text, or a slot under text")
        }
        (_, "white", n) if n <= 14 => {
            format!("`{prop}-lit` for a highlight away from text, or a slot under text")
        }
        (_, "white", _) => {
            format!("`{prop}-sheen` for a highlight away from text, or a slot under text")
        }
        (_, _, n) if n <= 30 => {
            format!("`{prop}-shade` for a shadow away from text, or a slot under text")
        }
        (_, _, n) if n <= 60 => format!(
            "`{prop}-scrim` behind a sheet, `{prop}-shade-deep` for a shadow, or a slot under text"
        ),
        _ => format!("`{prop}-shade-deep` for a shadow away from text, or a slot under text"),
    }
}

fn tag<'n>(name: &'n JSXElementName) -> &'n str {
    match name {
        JSXElementName::Identifier(i) => i.name.as_str(),
        JSXElementName::IdentifierReference(i) => i.name.as_str(),
        _ => "",
    }
}

impl<'p, 'a> Visit<'a> for Scan<'_, 'p, 'a> {
    fn visit_string_literal(&mut self, s: &StringLiteral<'a>) {
        let text = s.value.display().to_string();
        self.check_string(&text, s.span.start + 1);
    }

    fn visit_template_literal(&mut self, t: &TemplateLiteral<'a>) {
        for q in &t.quasis {
            let raw: &str = &q.value.raw;
            self.check_string(raw, q.span.start);
        }
        walk::walk_template_literal(self, t);
    }

    fn visit_jsx_element(&mut self, e: &JSXElement<'a>) {
        let name = tag(&e.opening_element.name);
        let mut tokens = Vec::new();
        for a in &e.opening_element.attributes {
            let JSXAttributeItem::Attribute(a) = a else {
                continue;
            };
            let JSXAttributeName::Identifier(n) = &a.name else {
                continue;
            };
            if n.name != "class" {
                continue;
            }
            match &a.value {
                Some(JSXAttributeValue::StringLiteral(s)) => split(
                    &s.value.display().to_string(),
                    s.span.start + 1,
                    &mut tokens,
                ),
                Some(JSXAttributeValue::ExpressionContainer(c)) => {
                    if let Some(x) = c.expression.as_expression() {
                        self.tokens(x, 0, &mut tokens);
                    }
                }
                _ => {}
            }
        }
        let ground = tokens.iter().any(|(c, _)| {
            c.starts_with("@slot")
                || colour_class(c).is_some_and(|(p, colour, _)| {
                    p != "text"
                        && (self.colors.contains_key(colour)
                            || colour == "white"
                            || colour == "black")
                })
        });
        let text = TEXT_TAGS.contains(&name)
            || e.children
                .iter()
                .any(|c| matches!(c, JSXChild::Text(t) if !t.value.trim().is_empty()));
        self.frames.push(Frame {
            tokens,
            ground,
            text,
        });
        if TEXT_TAGS.contains(&name) {
            self.check_art_ink();
        }
        walk::walk_jsx_element(self, e);
        let f = self.frames.pop().unwrap();
        if f.text {
            let tokens = f.tokens;
            self.check_ground(&tokens);
        }
        // Text reaches the parent's ground unless this element covers it
        // with its own (a wrapper never does).
        if f.text
            && (!f.ground || WRAPPERS.contains(&name))
            && let Some(parent) = self.frames.last_mut()
        {
            parent.text = true;
        }
    }
}

/// The sources a check covers: `.tsx` and `.jsx` under `dirs`.
pub fn files(dirs: &[&Path]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for d in dirs {
        crate::lockstep_lint::sources(d, &mut files);
    }
    files.retain(|f| f.extension().is_some_and(|e| e == "tsx" || e == "jsx"));
    files.sort();
    files.dedup();
    files
}

/// Every finding under `dirs`, paths relative to `root`.
pub fn lint(
    root: &Path,
    dirs: &[&Path],
    colors: &Colors,
    scale: &Scale,
    art: Option<&ArtGrounds>,
) -> Vec<(String, Finding)> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for path in files(dirs) {
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for f in scan(&path, &text, colors, scale, art) {
            out.push((rel.clone(), f));
        }
    }
    out
}
