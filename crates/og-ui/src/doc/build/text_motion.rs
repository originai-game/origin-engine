//! Core controls documents write as text and selects (docs/ui.md §2): a
//! counted number (`og-counter`), a typewriter text (`og-typewriter`, `onrevealed`),
//! a cycler (`<select data-cycle>`) and a spotlight
//! (`<og-spotlight target>`).

use super::*;

pub(crate) trait TextMotionUi {
    /// A counter that interpolates to `value` over its duration in ms
    /// (900 by default).
    fn count_text(&mut self, id: &str, value: f32, desc: &LabelDesc, r: &Resolved);

    /// What a typewriter text (`rate` graphemes per second) shows
    /// this tick: it starts again when its text changes and is whole under
    /// reduced motion. It fires `onrevealed` once as the last character
    /// shows. Its progress is widget memory (kept in snapshots); it builds
    /// every tick until it is whole.
    fn reveal<'a>(&mut self, key: u64, text: &'a str, cps: Option<f32>) -> &'a str;

    /// A `<select>` drawn as a cycler: previous, its option, next; each
    /// press steps (wrapping, past disabled options) and writes `value`
    /// back. Answers whether it changed.
    fn cycler(&mut self, b: &mut Build<'_>, n: &DocNode, id: &str, r: &Resolved) -> bool;

    /// An open `<og-spotlight target="id">`: the page dimmed but
    /// for the element of that id, the dialog's content a callout against
    /// it. Answers the shade's response and whether the callout layer is
    /// open (it is when the element is on the page).
    fn open_spotlight(
        &mut self,
        id: &str,
        _: (u64, &str),
        target: &str,
        r: &Resolved,
    ) -> (crate::ui::Response, bool);
}

impl TextMotionUi for Ui {
    fn count_text(&mut self, id: &str, value: f32, desc: &LabelDesc, r: &Resolved) {
        let shown = self.counter(
            id,
            f64::from(value),
            &crate::CounterDesc {
                step: desc.step,
                font: desc.font,
                color: desc.color,
                align: desc.align,
                width: desc.width,
                duration_ms: r.num("countDuration").unwrap_or(0.0),
                decimals: r.num("countDecimals").unwrap_or(0.0) as u32,
                ..crate::CounterDesc::default()
            },
        );
        // Still counting: it builds again next tick (its memory is still).
        if shown != f64::from(value)
            && let Some(owned) = self.tree.owning.last_mut()
        {
            owned.moved = true;
        }
    }

    fn reveal<'a>(&mut self, key: u64, text: &'a str, cps: Option<f32>) -> &'a str {
        let Some(cps) = cps.filter(|c| *c > 0.0) else {
            return text;
        };
        let memory = crate::id::child_key(key, "~reveal");
        let hash = og_core::hash::hash_bytes(text.as_bytes());
        let now = self.interaction.state.tick;
        let mut a = self.anim(memory);
        let fresh = !self.has_anim(memory) || a.h != hash;
        if fresh {
            a = crate::ui::Anim {
                h: hash,
                t: now,
                a: 0.0,
                b: 0.0,
            };
        }
        let graphemes = og_i18n::graphemes(text);
        let total = graphemes.len();
        let shown = if self.still() {
            total
        } else {
            let elapsed = now.saturating_sub(a.t) as f32 / self.env.ticks_per_second.max(1) as f32;
            ((elapsed * cps) as usize).min(total)
        };
        if shown == total && a.a < 1.0 {
            self.docs.host.happened.push(When::Revealed);
        }
        a.a = if shown == total { 1.0 } else { 0.0 };
        self.set_anim(memory, a);
        if shown < total
            && let Some(owned) = self.tree.owning.last_mut()
        {
            owned.moved = true;
        }
        let end = graphemes
            .get(shown)
            .and_then(|(start, _)| text.char_indices().nth(*start))
            .map_or(text.len(), |(i, _)| i);
        &text[..end]
    }

    fn cycler(&mut self, b: &mut Build<'_>, n: &DocNode, id: &str, r: &Resolved) -> bool {
        let strs = |name: &str| match r.get(name) {
            Some(V::Strs(s)) => s.clone(),
            _ => Vec::new(),
        };
        let (items, keys, off) = (strs("items"), strs("keys"), strs("disabledItems"));
        let mut values = strs("values");
        values.resize(items.len(), String::new());
        for (v, item) in values.iter_mut().zip(&items) {
            if v.is_empty() {
                v.clone_from(item);
            }
        }
        let count = items.len();
        let now = r
            .str("current")
            .and_then(|c| values.iter().position(|v| v == c))
            .unwrap_or(0);
        let disabled = r.flag("disabled");
        let label = |k: usize| {
            keys.get(k)
                .filter(|key| !key.is_empty())
                .and_then(|key| b.env.translate(key))
                .map(|t| b.env.shown(t))
                .unwrap_or_else(|| items.get(k).cloned().unwrap_or_default())
        };
        let text = label(now);
        let t = self.env.theme;
        self.open(
            id,
            Element {
                direction: Direction::Row,
                gap: t.space.xs,
                cross: Cross::Center,
                width: r.size("width").unwrap_or_default(),
                ..Element::default()
            },
        );
        let step = |ui: &mut Ui, name: &str, glyph: &str| {
            ui.button(
                name,
                glyph,
                ButtonDesc {
                    variant: ButtonVariant::Ghost,
                    disabled: disabled || count < 2,
                    ..ButtonDesc::default()
                },
            )
            .clicked
        };
        let back = step(self, "Prev", "‹");
        self.label(
            "Value",
            &text,
            LabelDesc {
                width: Size::grow(),
                align: TextAlign::Center,
                ..LabelDesc::default()
            },
        );
        let on = step(self, "Next", "›");
        self.close();
        let by: isize = match (back, on) {
            (true, false) => -1,
            (false, true) => 1,
            _ => return false,
        };
        let mut k = now;
        for _ in 0..count {
            k = (k as isize + by).rem_euclid(count as isize) as usize;
            if !off.iter().any(|o| o == &k.to_string()) {
                break;
            }
        }
        if k == now {
            return false;
        }
        self.docs.write_back(b, n, "current", json!(values[k]));
        true
    }

    fn open_spotlight(
        &mut self,
        id: &str,
        (key, path): (u64, &str),
        target: &str,
        r: &Resolved,
    ) -> (crate::ui::Response, bool) {
        // Its place in the tree, as a modal dialog's: a retained document
        // finds the element's node by its id under its parent.
        let at = self.layer_place(&crate::Id::from(id), key);
        self.tree.props_mut(at).position = crate::element::Position::Absolute;
        let found = self
            .tree
            .nodes
            .iter()
            .position(|node| node.id == target)
            .filter(|&i| self.tree.nodes[i].key != 0);
        let hole = found.and_then(|i| self.last_bounds(self.tree.nodes[i].key));
        let shade = self.spotlight(
            format!("{id}~spotlight").as_str(),
            hole,
            crate::widgets::SpotlightDesc::default(),
        );
        let t = self.env.theme;
        let content = Self::box_of(
            r,
            Element {
                direction: Direction::Column,
                gap: t.space.sm,
                padding: Insets::all(t.space.md),
                frame: Some(crate::style_file::StylePart::Panel),
                ..Element::default()
            },
            self.style.em_of(r, &self.env),
        );
        // Keyed and named as the element itself (as a world layer is), so
        // its content's keys are the ones a retained document gives it.
        self.open_layer_keyed(
            "popover",
            key,
            path.to_string(),
            content,
            crate::layout::Float::Popover,
            found.unwrap_or(NONE),
        );
        (shade, true)
    }
}
