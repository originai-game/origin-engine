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

/// The check's code in `og check`'s warnings.
pub const CODE: u32 = 9108;

/// The least opacity a ground under text has, in either scheme: below it,
/// the scene behind shows through enough to move the label's contrast.
const OPAQUE: f32 = 0.85;

/// One break of the rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub line: u32,
    pub col: u32,
    pub class: String,
    pub message: String,
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
pub fn scan(path: &Path, text: &str, colors: &Colors) -> Vec<Finding> {
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
        consts: names.consts,
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
    consts: HashMap<&'p str, &'p Expression<'a>>,
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
        self.found.entry(offset).or_insert(Finding {
            line,
            col,
            class: class.to_owned(),
            message,
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
                    out.push(("@slot".to_owned(), c.span.start))
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

    /// The rules any class string keeps, wherever it is written.
    fn check_string(&mut self, text: &str, offset: u32) {
        let mut tokens = Vec::new();
        split(text, offset, &mut tokens);
        for (class, at) in tokens {
            if let Some(message) = self.anywhere(&class) {
                self.report(&class, at, message);
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
            c == "@slot"
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
pub fn lint(root: &Path, dirs: &[&Path], colors: &Colors) -> Vec<(String, Finding)> {
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
        for f in scan(&path, &text, colors) {
            out.push((rel.clone(), f));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(src: &str) -> Vec<String> {
        let colors: Colors = [
            ("fg", 1.0),
            ("surface", 1.0),
            ("well", 0.88),
            ("entry-lo", 0.74),
            ("card", 0.42),
            ("hud-accent", 1.0),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
        scan(Path::new("a.tsx"), src, &colors)
            .into_iter()
            .map(|f| format!("{}:{} {}", f.line, f.col, f.class))
            .collect()
    }

    #[test]
    fn a_translucent_ground_counts_only_under_text() {
        // The bar's fill holds no text; the row's ground holds the name
        // through a wrapper; the chip's own ground covers its label, so the
        // row under it is not the label's ground.
        let src = r#"const ROW = "flex-row bg-entry-lo";
export function A(p: { on: boolean }) {
  const wash = () => p.on ? "from-card" : "";
  return (
    <View class="bg-card">
      <View class="h-1 bg-hud-accent/40" />
      <View class={ROW}><Show when={p.on}><Text>{p.name}</Text></Show></View>
      <View class="bg-card"><View class="bg-surface"><Text class="text-fg">Chip</Text></View></View>
      <View class={`p-1 ${p.on ? "bg-well/50" : "bg-surface"}`}><Text>x</Text></View>
      <View class={wash()}><Text>y</Text></View>
    </View>
  );
}"#;
        assert_eq!(
            found(src),
            ["1:23 bg-entry-lo", "3:30 from-card", "9:35 bg-well/50"]
        );
    }

    #[test]
    fn inks_literals_and_hand_colours_count_anywhere() {
        let src = r#"const MUTED = "text-fg/78 disabled:text-fg/40";
const X = <View class="hover:bg-white/8 from-black/50 bg-[#1b1d24] shadow-[0_0_0_1px_#fff] text-fg-muted" />;"#;
        assert_eq!(
            found(src),
            [
                "1:16 text-fg/78",
                "1:27 disabled:text-fg/40",
                "2:24 hover:bg-white/8",
                "2:41 from-black/50",
                "2:55 bg-[#1b1d24]"
            ]
        );
    }
}
