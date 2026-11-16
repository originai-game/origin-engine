//! Custom highlights (CSS Custom Highlight API 1): the game registers
//! named ranges of the laid-out text ([`Ui::set_highlight`], the
//! registry), and `::highlight(<name>)` rules style them. Each frame marks
//! every text element a range reaches with its characters and its
//! highlight's style, which each property takes from the nearest of the
//! element and its ancestors that gives it (highlight inheritance, CSS
//! Pseudo-Elements 4 §3.5). A highlight with no style paints nothing:
//! custom highlights have no default (§4.2.2).
//!
//! Paint stacks the overlays under the selection's, in ascending
//! priority, a tie by registration order, the later one above (§4.2.5). Within one highlight, overlapping ranges paint once,
//! as their union, and a collapsed range paints nothing (§4.2.4).

use super::select::SelectEnd;
use super::*;
use crate::doc::generated::Selection;
use crate::layout::NONE;
use std::sync::Arc;

/// A custom highlight: the ranges of text it covers and where its
/// overlay stacks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Highlight {
    pub ranges: Vec<HighlightRange>,
    /// Higher stacks above; a tie goes to the highlight registered later.
    pub priority: i32,
}

/// A range of text from a character of one text element to a character
/// of the same or a later one in document order: each element by its full
/// path, each offset counting its characters over its lines joined by one
/// break each (as `og.ui`'s selection counts them).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HighlightRange {
    pub start: String,
    pub start_offset: u32,
    pub end: String,
    pub end_offset: u32,
}

/// A registered highlight, its ranges' ends by render key.
#[derive(Clone, Debug)]
pub(crate) struct Registered {
    name: Arc<str>,
    priority: i32,
    ranges: Vec<(SelectEnd, SelectEnd)>,
}

impl Ui {
    /// Register `highlight` under `name` (`CSS.highlights.set`), replacing
    /// the one there in its place of registration; `None` removes it. It
    /// shows from the next frame and holds until it is set again.
    pub fn set_highlight(&mut self, name: &str, highlight: Option<Highlight>) {
        let registry = &mut self.interaction.highlights;
        let at = registry.iter().position(|h| &*h.name == name);
        let Some(h) = highlight else {
            if let Some(at) = at {
                registry.remove(at);
            }
            return;
        };
        let key = crate::id::key_of_path;
        let entry = Registered {
            name: name.into(),
            priority: h.priority,
            ranges: h
                .ranges
                .iter()
                .map(|r| ((key(&r.start), r.start_offset), (key(&r.end), r.end_offset)))
                .collect(),
        };
        match at {
            Some(at) => registry[at] = entry,
            None => registry.push(entry),
        }
    }

    /// The frame's custom-highlight marks, as the selection's are marked:
    /// each reached element's overlays on its node, bottom first, and the
    /// nodes whose marks changed.
    pub(crate) fn mark_highlights(&mut self) -> Vec<usize> {
        let mut order: Vec<&Registered> = self.interaction.highlights.iter().collect();
        // Stable: of equal priorities, the later registered stays above.
        order.sort_by_key(|h| h.priority);
        let mut now: Vec<(usize, HighlightMarks)> = Vec::new();
        for h in order {
            for (i, r) in self.tree.passage_ranges(&h.ranges) {
                let Some(style) = self.highlight_style(i, &h.name) else {
                    continue;
                };
                let at = match now.iter().position(|(j, _)| *j == i) {
                    Some(at) => at,
                    None => {
                        now.push((i, Vec::new()));
                        now.len() - 1
                    }
                };
                now[at].1.push((Arc::new(style), r));
            }
        }
        let tree = &mut self.tree;
        let mut changed = Vec::new();
        for key in std::mem::take(&mut self.interaction.highlight_marked) {
            if let Some(i) = tree.index.find(key)
                && !now.iter().any(|(j, _)| *j == i)
                && !std::mem::take(&mut tree.nodes[i].marks.highlights).is_empty()
            {
                changed.push(i);
            }
        }
        for (i, marks) in now {
            // Equal marks keep their styles, so the draw cache sees none
            // changed.
            if tree.nodes[i].marks.highlights != marks {
                tree.nodes[i].marks.highlights = marks;
                changed.push(i);
            }
            self.interaction.highlight_marked.push(tree.nodes[i].key);
        }
        changed
    }

    /// `::highlight(name)`'s style for node `i`'s text
    /// ([`Self::inherited_highlight`]).
    fn highlight_style(&self, i: usize, name: &str) -> Option<Selection> {
        let styles = &self.docs.state.highlights;
        if styles.is_empty() {
            return None;
        }
        self.inherited_highlight(i, |key| styles.get(&key).and_then(|m| m.get(name)))
    }

    /// `::selection`'s style for node `i`'s text
    /// ([`Self::inherited_highlight`]).
    pub(crate) fn selection_paint(&self, i: usize) -> Option<Selection> {
        let styles = &self.docs.state.selection;
        if styles.is_empty() {
            return None;
        }
        self.inherited_highlight(i, |key| styles.get(&key).map(|s| &**s))
    }

    /// A highlight pseudo-element's style for node `i`'s text, `own`
    /// giving each element's own by its render key: each property from the
    /// nearest of the text's element and its ancestors that declares it,
    /// inherited or not (highlight inheritance, CSS Pseudo-Elements 4
    /// §3.5); `None` when none does.
    fn inherited_highlight<'a>(
        &self,
        i: usize,
        own: impl Fn(u64) -> Option<&'a Selection>,
    ) -> Option<Selection> {
        let mut out = Selection::default();
        let mut k = self.text_owner(i);
        while k != NONE {
            let n = &self.tree.nodes[k];
            if let Some(s) = own(n.key) {
                out.background = out.background.or(s.background);
                out.color = out.color.or(s.color);
                out.decoration = out.decoration.take().or_else(|| s.decoration.clone());
                out.shadows = out.shadows.take().or_else(|| s.shadows.clone());
            }
            k = n.parent;
        }
        (out != Selection::default()).then_some(out)
    }

    /// Node `i`'s custom-highlight overlays from `under` on, bottom first
    /// ([`Self::overlay_commands`]); where the next overlay's fills go.
    pub(super) fn highlight_commands(
        &self,
        i: usize,
        mut under: usize,
        cmds: &mut Vec<DrawCmd>,
    ) -> usize {
        for (style, marks) in &self.tree.nodes[i].marks.highlights {
            under = self.overlay_commands(i, under, marks, Some(style), style.background, cmds);
        }
        under
    }
}

/// A node's custom-highlight overlays, bottom first: each one's resolved
/// style and its characters, `[start, end)`.
pub type HighlightMarks = Vec<(Arc<Selection>, Vec<(u32, u32)>)>;
