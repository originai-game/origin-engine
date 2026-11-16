//! A text's inline elements lowered onto its runs: each run takes the CSS
//! of the innermost element it sits in where that differs from the text's,
//! the `::part(name)` rules of the elements naming a part over it, and
//! its elements' `display` gives the text its ruby structure (CSS Ruby 1
//! §2.2).

use super::{Build, Resolved, V, schema};
use crate::doc::DocNode;
use crate::rich::{InlineAtom, RubyLeaf, RubyOut, RubyRole, ruby_structure};
use crate::ui::Ui;
use crate::{TextSpan, TextSpanStyle};

/// `top`'s declared fields over `base`'s.
fn overlay(base: &mut TextSpanStyle, top: &TextSpanStyle) {
    macro_rules! over {
        ($($field:ident),*) => { $(
            if top.$field.is_some() { base.$field = top.$field.clone(); }
        )* };
    }
    over!(
        color,
        step,
        font,
        weight,
        bold,
        italic,
        underline,
        strike,
        outline,
        motion,
        link,
        size,
        line_height,
        tracking_em,
        atom,
        decoration,
        vertical_align,
        size_scale,
        word_spacing,
        synthesis,
        oblique,
        stretch,
        combine,
        variations,
        optical_sizing,
        ruby_role,
        ruby_position,
        palette
    );
}

/// The ruby role a reflected `display` gives.
fn role_of(display: &str) -> Option<RubyRole> {
    Some(match display {
        "ruby" => RubyRole::Container,
        "rubyBase" => RubyRole::Base,
        "rubyText" => RubyRole::Annotation,
        "rubyBaseContainer" => RubyRole::BaseContainer,
        "rubyTextContainer" => RubyRole::AnnotationContainer,
        "none" => RubyRole::None,
        "inline" => RubyRole::Inline,
        _ => return None,
    })
}

impl Ui {
    /// Text `n`'s values `r` with its inline elements lowered onto its
    /// runs: its `text` the words it shows and its `spans` the runs, each
    /// styled by the elements it sits in ([`Build::inline`]).
    pub(crate) fn lower_inline(&self, b: &Build<'_>, n: &DocNode, r: &Resolved) -> Resolved {
        let theme = self.theme();
        let fonts = self.fonts();
        let sheet = &b.doc.sheet;
        let flow = r.own_flow();
        let base = r.label_desc(theme, fonts, flow.rtl);
        let base_step = base.step.unwrap_or(theme.type_ramp.body);
        let base_style = r
            .text
            .as_deref()
            .map(|t| r.text_style(t, sheet))
            .unwrap_or_default();
        let element = |i: usize| {
            b.inline
                .iter()
                .find(|(e, _)| *e == i)
                .map(|(_, r)| &r.values)
        };
        // What `::part(name)` rules give an element naming its part
        // (resolved with the text, an empty span of that name).
        let part = |i: usize| {
            let name = b.doc.nodes[i].part_name()?;
            r.span_styles
                .iter()
                .find(|(s, _)| s.length == 0 && s.name == name)
                .map(|(_, style)| style)
        };
        // An element's CSS where it differs from the text's.
        let own = |e: &Resolved| {
            let Some(t) = e.text.as_deref() else {
                return TextSpanStyle::default();
            };
            let mut s = e.text_style(t, sheet);
            macro_rules! same {
                ($($field:ident),*) => { $(
                    if s.$field == base_style.$field { s.$field = None; }
                )* };
            }
            same!(
                italic,
                weight,
                oblique,
                stretch,
                variations,
                optical_sizing,
                palette,
                synthesis,
                color,
                outline,
                decoration,
                word_spacing
            );
            let d = e.label_desc(theme, fonts, flow.rtl);
            let step = d.step.unwrap_or(theme.type_ramp.body);
            s.size = (step.size != base_step.size).then_some(step.size);
            s.line_height = (step.line_height != base_step.line_height).then_some(step.line_height);
            s.tracking_em = (step.tracking_em != base_step.tracking_em).then_some(step.tracking_em);
            s.font = (d.font != base.font).then_some(d.font);
            if s.color.is_none() && d.color != base.color {
                s.color = d.color.map(crate::paint::ColorRef::Literal);
            }
            let combine = e.own_flow().combine;
            s.combine = (combine != flow.combine).then_some(combine);
            s
        };
        let role = |i: usize| {
            part(i)
                .and_then(|s| s.ruby_role)
                .or_else(|| element(i).and_then(|e| e.str("display")).and_then(role_of))
                .unwrap_or_else(|| {
                    RubyRole::of_tag(b.doc.nodes[i].selector_tag.as_deref().unwrap_or(""))
                })
        };
        // A translated `<a i18n>` is one run: the string's text.
        let translated = r
            .str("key")
            .and_then(|k| b.env.translate(k))
            .map(|t| b.env.shown(t))
            .filter(|_| n.inline.len() == 1);
        let paths: Vec<Vec<(u32, RubyRole)>> = n
            .inline
            .iter()
            .map(|run| run.path.iter().map(|&e| (e as u32, role(e))).collect())
            .collect();
        let leaves: Vec<RubyLeaf> = n
            .inline
            .iter()
            .zip(&paths)
            .map(|(run, path)| RubyLeaf {
                path,
                text: match &run.text {
                    _ if translated.is_some() => translated.clone().unwrap_or_default(),
                    crate::bind::Expr::Lit(v) => crate::bind::text(v),
                    e => crate::bind::text(&b.eval(e)),
                },
            })
            .collect();
        // An annotation container's `ruby-position`: its part's, else its
        // computed one (which its annotations inherit).
        let boxes = ruby_structure(&leaves, &|ids| {
            let &id = ids.last()?;
            let id = id as usize;
            part(id).and_then(|s| s.ruby_position).or_else(|| {
                element(id)
                    .and_then(|e| e.text.as_deref())
                    .map(|t| t.ruby.position)
            })
        });
        let mut out = r.clone();
        out.span_styles.clear();
        // The text shown: its bases and words, not its annotations.
        let shown: String = boxes
            .iter()
            .map(|o| match o {
                RubyOut::Text { text, .. } => text.clone(),
                RubyOut::Ruby { ruby, .. } => ruby.base_text(),
            })
            .collect();
        if r.str("text") != Some(shown.as_str())
            && let Some((spec, _)) = schema::prop("text")
        {
            out.vals.push((spec, V::Str(shown)));
        }
        let mut spans = Vec::new();
        let mut position = 0u32;
        for o in boxes {
            let (k, text, ruby) = match o {
                RubyOut::Text { run, text } => (run, text, None),
                RubyOut::Ruby { run, ruby } => (run, ruby.base_text(), Some(ruby)),
            };
            let run = &n.inline[k];
            let length = text.chars().count() as u32;
            // Named by the part it is in, which a style's part rules read.
            let name = run
                .path
                .iter()
                .rev()
                .find_map(|&e| b.doc.nodes[e].part_name())
                .unwrap_or("inline");
            let span = TextSpan {
                name: name.into(),
                position,
                length,
                properties: Default::default(),
            };
            position += length;
            let mut style = run
                .path
                .last()
                .and_then(|&e| element(e))
                .map(own)
                .unwrap_or_default();
            // `vertical-align` is not inherited: the innermost element
            // that declares it shifts the words in it.
            style.vertical_align = run.path.iter().rev().find_map(|&e| {
                element(e)
                    .and_then(|e| e.text.as_deref())
                    .and_then(|t| t.vertical_align.as_deref())
                    .and_then(crate::VerticalAlign::parse)
            });
            for &e in &run.path {
                if let Some(p) = part(e) {
                    overlay(&mut style, p);
                }
            }
            style.ruby_role = None;
            style.ruby_position = None;
            style.link = run.link.map(super::inline_link);
            style.atom = match ruby {
                Some(ruby) => Some(InlineAtom::Ruby(ruby)),
                None => run.atom.clone(),
            };
            out.span_styles.push((span.clone(), style));
            spans.push(span);
        }
        // Elements that style nothing and name no part leave it a plain
        // text.
        if out
            .span_styles
            .iter()
            .all(|(span, s)| span.name == "inline" && *s == TextSpanStyle::default())
        {
            out.span_styles.clear();
        } else if let Some((spec, _)) = schema::prop("spans") {
            out.vals.push((spec, V::Spans(Ok(spans))));
        }
        out
    }
}
