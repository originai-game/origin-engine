//! Rolling number transitions (`transition-count`): extracts a number from
//! a text node's string and rolls the displayed digits smoothly over time.

use super::{Kind, NONE, Tree};

/// A text rolling to a new number ([`Tree::roll`]).
#[derive(Clone, Debug)]
pub(super) struct Count {
    /// The text node.
    pub(super) e: u32,
    from: f64,
    /// What it draws now.
    now: f64,
    to: Number,
    /// The time it started.
    since: f64,
    /// When it last sounded `tick` (`NEG_INFINITY`: not yet).
    ticked: f64,
}

/// The least time between a rolling number's `tick`s (s): a fast roll
/// ticks at this rate rather than once per drawn value.
const TICK_GAP: f64 = 0.05;

/// The one number in a text (`1,250`, `-3.5`, `x12`), how it is written,
/// and where: what a `transition-count` text rolls.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Number {
    value: f64,
    /// Bytes before it and after it.
    at: [usize; 2],
    decimals: usize,
    /// The thousands separator it is grouped by, if any.
    group: Option<char>,
}

impl Number {
    /// The text's number, if it holds exactly one.
    fn of(text: &str) -> Option<Number> {
        let start = text.find(|c: char| c.is_ascii_digit())?;
        let neg = text[..start].ends_with('-');
        let mut group = None;
        let mut digits = String::new();
        let mut end = start;
        let mut decimals = None::<usize>;
        let b = text.as_bytes();
        let mut i = start;
        while i < text.len() {
            let c = text[i..].chars().next()?;
            if c.is_ascii_digit() {
                digits.push(c);
                if let Some(d) = decimals.as_mut() {
                    *d += 1;
                }
                i += 1;
                end = i;
            } else if (c == ',' || c == '\u{2009}' || c == '\u{202f}')
                && decimals.is_none()
                && text[i + c.len_utf8()..].starts_with(|d: char| d.is_ascii_digit())
            {
                group = Some(c);
                i += c.len_utf8();
            } else if c == '.' && decimals.is_none() && b.get(i + 1).is_some_and(u8::is_ascii_digit)
            {
                digits.push('.');
                decimals = Some(0);
                i += 1;
            } else {
                break;
            }
        }
        if text[end..].contains(|c: char| c.is_ascii_digit()) {
            return None;
        }
        let value: f64 = digits.parse().ok()?;
        Some(Number {
            value: if neg { -value } else { value },
            at: [if neg { start - 1 } else { start }, end],
            decimals: decimals.unwrap_or(0),
            group,
        })
    }

    /// `text`, the one this number was read from, with `v` in its place,
    /// written as it is.
    fn show(&self, v: f64, text: &str) -> String {
        let n = format!("{:.*}", self.decimals, v.abs());
        let (int, frac) = n.split_once('.').unwrap_or((&n, ""));
        let mut out = String::with_capacity(text.len() + 4);
        out.push_str(&text[..self.at[0]]);
        if v < 0.0 && n.bytes().any(|d| d != b'0' && d != b'.') {
            out.push('-');
        }
        for (k, d) in int.chars().enumerate() {
            if k > 0
                && (int.len() - k).is_multiple_of(3)
                && let Some(g) = self.group
            {
                out.push(g);
            }
            out.push(d);
        }
        if !frac.is_empty() {
            out.push('.');
            out.push_str(frac);
        }
        out.push_str(&text[self.at[1]..]);
        out
    }
}

impl Tree {
    /// `e`'s text; an image's is its source. In a `transition-count` text
    /// drawn already, a number that changes rolls there on its curve: its
    /// text (what the probe and a screen reader read) is the new one at
    /// once, and what draws counts up or down to it ([`Tree::roll`]).
    pub(super) fn set_text(&mut self, e: u32, text: &str) {
        if self.els[e as usize].kind == Kind::Image {
            return self.set_src(e, text);
        }
        let el = &self.els[e as usize];
        if &*el.text == text {
            return;
        }
        let rolls = el.kind == Kind::Label
            && el.entering.is_none()
            && el.parent != NONE
            && self.els[el.parent as usize].record.count
            && !self.motion.reduced();
        // A number written short (`0`) rolls written as the long one was;
        // one whose words or unit change (`999` to `1.2K`) jumps.
        let old = Number::of(&el.text);
        let group = old.and_then(|n| n.group);
        let to = Number::of(text).map(|n| Number {
            group: n.group.or(group),
            ..n
        });
        let same = match (old, to) {
            (Some(o), Some(to)) => {
                el.text[..o.at[0]] == text[..to.at[0]] && el.text[o.at[1]..] == text[to.at[1]..]
            }
            _ => false,
        };
        let shown = self.counts.iter().position(|c| c.e == e);
        let from = match shown {
            Some(i) => Some(self.counts.swap_remove(i).now),
            None => Number::of(&el.text).map(|n| n.value),
        };
        let el = &mut self.els[e as usize];
        el.text.clear();
        el.text.push_str(text);
        match (from, to) {
            (Some(from), Some(to)) if rolls && from != to.value && same => {
                self.counts.push(Count {
                    e,
                    from,
                    now: from,
                    to,
                    since: self.time,
                    ticked: f64::NEG_INFINITY,
                })
            }
            _ => {
                if let Some(leaf) = el.leaf {
                    self.arena.set_text(leaf, text);
                }
            }
        }
        self.para_touch(e);
        self.paint = true;
    }

    /// Draws each rolling number at its place on its text's curve, and
    /// lands it on its text when the curve comes to rest. As the drawn
    /// number changes the text sounds `tick` (its parent's slot's, so
    /// `Count.roll` may restyle it), at most once per [`TICK_GAP`].
    pub(super) fn roll(&mut self) {
        let mut i = 0;
        while i < self.counts.len() {
            let c = &self.counts[i];
            let el = &self.els[c.e as usize];
            let spec = self.els[el.parent as usize].record.motion;
            let (p, rest) = crate::motion::Motion::progress(&spec, (self.time - c.since) as f32);
            let done = rest || el.dead || self.motion.reduced();
            let now = c.from + (c.to.value - c.from) * p as f64;
            let text = if done {
                el.text.clone()
            } else {
                c.to.show(now, &el.text)
            };
            let (parent, leaf) = (el.parent, el.leaf);
            let moved = leaf.is_some_and(|l| self.arena.text_of(l) != Some(&*text));
            if let Some(leaf) = leaf {
                self.arena.set_text(leaf, &text);
            }
            let e = c.e;
            if moved && !done && self.time - c.ticked >= TICK_GAP {
                self.counts[i].ticked = self.time;
                self.sound(if parent == NONE { e } else { parent }, "tick");
            }
            self.para_touch(e);
            if done {
                self.counts.swap_remove(i);
            } else {
                self.counts[i].now = now;
                i += 1;
            }
        }
    }
}
