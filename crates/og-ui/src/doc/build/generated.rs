//! Generated content in the build: `::before`/`::after` text and boxes,
//! `::marker` content, `::first-letter` runs, and the document-order
//! counter and quote run ([`crate::doc::generated::Run`]) every element
//! moves as it is built.
use super::*;
use crate::doc::generated::{self as gc, Content, PseudoBox, Quotes};

/// The computed-value entry carrying the inherited `quotes`.
pub(super) const QUOTES: &str = "@quotes";

impl Ui {
    /// Enters an element in the counter run and applies its counter
    /// properties; nothing unless some document uses generated content.
    pub(super) fn run_enter(&mut self, r: &Resolved) {
        if !self.docs.uses_run {
            return;
        }
        // Style containment scopes its counters and quotes to its subtree.
        if crate::doc::contain_css::contain(|p| r.str(p).map(str::to_owned))
            .has(crate::layout::Contain::STYLE)
        {
            self.docs.run.enter_contained();
        } else {
            self.docs.run.enter();
        }
        if let Some(g) = &r.generated {
            self.docs.run.apply(&g.ops);
        }
    }

    pub(super) fn run_exit(&mut self) {
        if self.docs.uses_run {
            self.docs.run.exit();
        }
    }

    /// A pseudo-element's generated text, evaluated at its place in the
    /// run (its own counter properties apply first); `None` for `normal`
    /// and `none`, or `display: none`.
    pub(super) fn pseudo_text(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        which: &PseudoBox,
    ) -> Option<String> {
        let Content::Pieces(pieces) = &which.content else {
            return None;
        };
        if hidden(&which.style) {
            return None;
        }
        let quotes = r
            .cascade
            .computed
            .get(QUOTES)
            .cloned()
            .flatten()
            .and_then(|q| gc::parse_quotes(&q).ok())
            .unwrap_or(Quotes::Auto);
        let facts = super::structure::selector_facts(n, |e| b.eval(e));
        let attr = |name: &str| {
            facts.get(name).map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                v => v.to_string(),
            })
        };
        let run = &mut self.docs.run;
        run.enter();
        run.apply(&which.ops);
        let text = run.evaluate(pieces, &quotes, &b.doc.sheet, &attr, r.ordinal);
        run.exit();
        Some(text)
    }

    /// A box's `::before` or `::after`: its generated text, styled by the
    /// pseudo-element's text properties, in the pseudo-element's own box
    /// when it has one ([`crate::doc::DocNode::pseudo`]).
    pub(super) fn generated_box(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        path: &str,
        before: bool,
    ) {
        let Some(g) = r.generated.clone() else {
            return;
        };
        let Some(which) = (if before { &g.before } else { &g.after }) else {
            return;
        };
        let Some(text) = self.pseudo_text(b, n, r, which) else {
            return;
        };
        let style = self.pseudo_style(&which.style, r);
        let name = if before { "::before" } else { "::after" };
        let run = move |ui: &mut Ui| {
            let span = crate::TextSpan {
                name: "generated".into(),
                position: 0,
                length: text.chars().count() as u32,
                properties: Default::default(),
            };
            ui.rich_text_spans(
                name,
                &text,
                std::slice::from_ref(&span),
                crate::RichTextDesc::default(),
                |_| style.clone(),
            );
        };
        match self.pseudo_node(b, r, path, &name[2..]) {
            Some((at, node)) => {
                self.build_resolved_node(b, at, node, |ui, _, _| run(ui));
            }
            None => run(self),
        }
    }

    /// Element `r`'s box pseudo-element `which`, resolved, as the build
    /// takes it, with its node.
    pub(super) fn pseudo_node(
        &self,
        b: &Build<'_>,
        r: &Resolved,
        path: &str,
        which: &str,
    ) -> Option<(usize, ResolvedNode)> {
        let (_, at, values) = r.pseudo.iter().find(|(w, ..)| *w == which)?;
        let path = format!("{path}/::{which}");
        Some((
            *at,
            ResolvedNode {
                id: format!("::{which}"),
                key: crate::id::key_of_path(&path),
                values: (**values).clone(),
                probe: Built {
                    path: path.clone(),
                    doc: b.doc.name.clone(),
                    node: *at,
                    line: b.doc.nodes[*at].line,
                    mount: b.mount.clone(),
                    ..Built::default()
                },
                path,
            },
        ))
    }

    /// Styles node `i`, a box a control draws itself (a modal's scrim, a
    /// select's open list), as its element's pseudo-element `node`: every
    /// box property of the pseudo-element's rules over the control's own.
    pub(super) fn style_pseudo_box(
        &mut self,
        b: &Build<'_>,
        i: usize,
        kind: &'static str,
        (at, node): &(usize, ResolvedNode),
    ) {
        let n = &b.doc.nodes[*at];
        let mut built = node.probe.clone();
        let props = self.props_of(b, n, node.key, &node.values, &mut built);
        let fields = fields::RenderFields::capture(
            self,
            self.nodes[i].key,
            i,
            kind,
            Default::default(),
            props,
            None,
        );
        self.apply_document_fields(&b.doc, n, &node.values, &fields);
    }

    /// A pseudo-element's text style from its declarations (`name:
    /// value;`); invalid ones are reported and skipped.
    pub(super) fn pseudo_style(&mut self, body: &str, _r: &Resolved) -> crate::TextSpanStyle {
        let body: String = crate::doc::css::split_decls(body)
            .unwrap_or_default()
            .into_iter()
            .filter(|(name, _)| name != "display")
            .map(|(name, value)| {
                let name = if name == "text-decoration" {
                    "text-decoration-line".to_string()
                } else {
                    name
                };
                format!("{name}: {value};")
            })
            .collect();
        match crate::doc::spans::style_of(&body, self.theme(), &|n| self.family_declared(n)) {
            Ok(style) => style,
            Err(error) => {
                self.warnings.push(format!("generated content: {error}"));
                crate::TextSpanStyle::default()
            }
        }
    }

    /// A text element's text with its `::before` and `::after` content and
    /// the runs that style them and its `::first-letter`; `None` when it
    /// generates nothing.
    pub(super) fn generated_text(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        key: u64,
        text: &str,
    ) -> Option<(String, Vec<(crate::TextSpan, crate::TextSpanStyle)>)> {
        let g = r.generated.clone()?;
        let pre = g
            .before
            .as_ref()
            .and_then(|w| Some((self.pseudo_text(b, n, r, w)?, w)));
        let post = g
            .after
            .as_ref()
            .and_then(|w| Some((self.pseudo_text(b, n, r, w)?, w)));
        if pre.is_none() && post.is_none() && g.first_letter.is_none() && g.first_line.is_none() {
            return None;
        }
        let mut spans = Vec::new();
        let mut full = String::new();
        let mut body_at = 0u32;
        let parts = [
            pre.as_ref()
                .map(|(s, w)| (s.as_str(), Some(w.style.as_str()), "::before")),
            Some((text, None, "")),
            post.as_ref()
                .map(|(s, w)| (s.as_str(), Some(w.style.as_str()), "::after")),
        ];
        let mut at = 0u32;
        for (s, style, name) in parts.into_iter().flatten() {
            let length = s.chars().count() as u32;
            match style {
                Some(style) if length > 0 => {
                    let style = self.pseudo_style(style, r);
                    spans.push((
                        crate::TextSpan {
                            name: name.into(),
                            position: at,
                            length,
                            properties: Default::default(),
                        },
                        style,
                    ));
                }
                Some(_) => {}
                None => body_at = at,
            }
            full.push_str(s);
            at += length;
        }
        // `::first-letter`: the first typographic letter unit, with the
        // punctuation before it, of the text including `::before`.
        if let Some(style) = &g.first_letter {
            let chars: Vec<char> = full.chars().collect();
            let lead = chars
                .iter()
                .take_while(|c| c.is_ascii_punctuation() || c.is_whitespace())
                .count();
            if lead < chars.len() {
                let style = self.pseudo_style(style, r);
                spans.push((
                    crate::TextSpan {
                        name: "::first-letter".into(),
                        position: 0,
                        length: lead as u32 + 1,
                        properties: Default::default(),
                    },
                    style,
                ));
            }
        }
        // `::first-line`: the characters of the first line as the last
        // layout broke it (the whole text before the first). The line moves
        // with layout, so the element builds again each frame.
        if let Some(style) = &g.first_line {
            let total = full.chars().count();
            let first = self.first_line_chars(key).unwrap_or(total).min(total);
            if let Some(owned) = self.owning.last_mut() {
                owned.moved = true;
            }
            if first > 0 {
                let style = self.pseudo_style(style, r);
                spans.insert(
                    0,
                    (
                        crate::TextSpan {
                            name: "::first-line".into(),
                            position: 0,
                            length: first as u32,
                            properties: Default::default(),
                        },
                        style,
                    ),
                );
            }
        }
        // The element's own runs move past `::before`.
        for (span, style) in &r.span_styles {
            let mut span = span.clone();
            span.position += body_at;
            spans.push((span, style.clone()));
        }
        Some((full, spans))
    }

    /// `::marker`'s content and ink for a list item: `None` content keeps
    /// the list style's marker.
    pub(super) fn marker_override(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
    ) -> (Option<Option<String>>, Option<Srgba>) {
        let Some(m) = r.generated.as_ref().and_then(|g| g.marker.clone()) else {
            return (None, None);
        };
        let style = self.pseudo_style(&m.style, r);
        let ink = style.color.and_then(|c| c.srgb(self.theme()));
        let content = match &m.content {
            Content::Normal => None,
            Content::None => Some(None),
            Content::Pieces(_) => Some(self.pseudo_text(b, n, r, &m)),
        };
        (content, ink)
    }
}

impl Ui {
    /// How many characters the first line of text node `key` held in the
    /// last layout.
    fn first_line_chars(&self, key: u64) -> Option<usize> {
        let n = self
            .previous_nodes()
            .find(|n| n.key == key)
            .or_else(|| self.tree.find(key).map(|i| &self.nodes[i]))?;
        if let Some(line) = n.rich_lines.first() {
            return Some(line.pieces.iter().map(|p| p.text.chars().count()).sum());
        }
        n.lines.first().map(|l| l.text.chars().count())
    }

    /// `::placeholder`'s color as declared.
    pub(super) fn pseudo_ink(&self, r: &Resolved) -> Option<Srgba> {
        let body = r.generated.as_ref()?.placeholder.as_ref()?;
        let decls = crate::doc::css::split_decls(body).ok()?;
        self.declared_color(&decls, &["color"])
    }

    /// The last of `want` among `decls`, as a colour.
    fn declared_color(&self, decls: &[(String, String)], want: &[&str]) -> Option<Srgba> {
        let (_, value) = decls
            .iter()
            .rev()
            .find(|(n, _)| want.contains(&n.as_str()))?;
        let entries = crate::doc::css::declarations(&format!("color: {value};"), 0).ok()?;
        let (_, spec) = schema::prop("color")?;
        match crate::doc::parse_lit(spec.ty, &entries.first()?.values).ok()? {
            Lit::Paint(Paint::Color(c)) => Some(c),
            Lit::Paint(Paint::Token(name)) => {
                crate::paint::ColorRef::token(&name).srgb(self.theme())
            }
            _ => None,
        }
    }

    /// Records a built element's `::selection` style for the draw.
    pub(super) fn note_selection(&mut self, key: u64, r: &Resolved) {
        let style = r
            .generated
            .as_ref()
            .and_then(|g| g.selection.as_ref())
            .and_then(|body| self.selection_style(body));
        match style {
            Some(s) => {
                if self.docs.selection.get(&key).is_none_or(|old| **old != s) {
                    self.docs.selection.insert(key, Arc::new(s));
                }
            }
            None => {
                if !self.docs.selection.is_empty() {
                    self.docs.selection.remove(&key);
                }
            }
        }
    }

    /// `::selection`'s background, color, decoration and shadows (CSS
    /// Pseudo-Elements 4 §3.2); `None` when it declares none of them.
    fn selection_style(&self, body: &str) -> Option<crate::doc::generated::Selection> {
        let decls = crate::doc::css::split_decls(body).ok()?;
        let color = self.declared_color(&decls, &["color"]);
        let mut s = crate::doc::generated::Selection {
            background: self.declared_color(&decls, &["background-color", "background"]),
            color,
            ..Default::default()
        };
        if let Some((_, v)) = decls.iter().rev().find(|(n, _)| n == "text-decoration")
            && let Ok((mut d, c)) = crate::doc::generated::selection_decoration(v)
        {
            d.color = c.and_then(|c| self.declared_color(&[("color".into(), c)], &["color"]));
            s.decoration = Some(d);
        }
        if let Some((_, v)) = decls.iter().rev().find(|(n, _)| n == "text-shadow") {
            let current =
                crate::paint::ColorRef::Literal(color.unwrap_or(self.theme().palette.ink));
            let shadows = crate::doc::paint_css::text_shadows(v, current).unwrap_or_default();
            s.shadows = Some(
                shadows
                    .iter()
                    .filter_map(|t| Some((t.offset, t.blur, t.color.srgb(self.theme())?)))
                    .collect(),
            );
        }
        (s != Default::default()).then_some(s)
    }
}

/// Whether a pseudo-element's declarations hide it (`display: none`).
fn hidden(style: &str) -> bool {
    crate::doc::css::split_decls(style)
        .unwrap_or_default()
        .iter()
        .any(|(name, value)| name == "display" && value.trim() == "none")
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use std::collections::BTreeMap;

    /// Counters number headings and list items in document order through
    /// `::before`, `::marker` takes content, `::after` boxes generate text,
    /// and an idle frame builds no element again.
    #[test]
    fn generated_content_counts_in_document_order() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "doc.svelte",
                r#"<meta name="component" content="doc">
<column id="Doc">
  <h2 id="A">Intro</h2>
  <h2 id="B" class="skip">Aside</h2>
  <h2 id="C">Rules</h2>
  <ol id="List"><li id="One">a</li><li id="Two">b</li></ol>
  <column id="Box"/>
  <div id="Chip"><text id="Name">Ann</text></div>
  <text id="Quote">hi</text>
</column>
<style>
#Doc { counter-reset: sec; }
h2:not(.skip)::before { counter-increment: sec; content: counter(sec, upper-roman) ". "; color: #ff0000; }
li::marker { content: "(" counter(list-item) ") "; }
#Box::after { content: "end"; }
#Chip { display: flex; }
#Chip::before { content: counter(sec); display: block; width: 30px; padding: 4px; background: #00ff00; }
#Quote::before { content: open-quote; }
#Quote::after { content: close-quote attr(id); }
</style>"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            SourceId::new("test", "doc"),
            "doc",
            BTreeMap::new(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(600.0, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let texts: Vec<String> = ui
                .probe_elements()
                .iter()
                .filter(|e| !e.text.is_empty())
                .map(|e| e.text.clone())
                .collect();
            (texts, ui.docs.retained.reconciled)
        };
        let (texts, _) = frame(&mut ui);
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        for expected in [
            "I. Intro",
            "Aside",
            "II. Rules",
            "(1) ",
            "(2) ",
            "end",
            "\u{201C}hi\u{201D}Quote",
        ] {
            assert!(
                texts.iter().any(|t| t == expected),
                "{expected:?} in {texts:?}"
            );
        }
        // A box pseudo-element is a box: its size, padding and fill, its
        // text inside, and no sibling of the element's children.
        let before = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("Chip/::before"))
            .expect("the ::before box");
        let elements = ui.probe_elements();
        let name = elements.iter().find(|e| e.id == "Name").unwrap();
        let text = elements
            .iter()
            .find(|e| e.path.ends_with("Chip/::before/::before"))
            .expect("its text");
        assert_eq!(
            (before.width, name.x - before.x, text.x - before.x),
            (30.0, 30.0, 4.0)
        );
        assert!(texts.iter().any(|t| t == "2"), "its text counts: {texts:?}");
        // The first idle frame rebuilds as any document's does; then the
        // held subtrees replay their counter events and nothing builds.
        frame(&mut ui);
        let (again, reconciled) = frame(&mut ui);
        assert_eq!((again, reconciled), (texts, 0));
    }
}
