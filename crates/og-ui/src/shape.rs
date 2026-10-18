//! Shaping and visual order for labels (E6 localization). A font cooked
//! with a shaped script (Arabic and og-i18n's other `needs_shaping`
//! scripts) carries its font file and its contextual forms as glyphs under
//! U+F0000 + glyph id; a word in such a script is shaped with rustybuzz and
//! drawn as those private-use characters, in the visual order rustybuzz
//! answers. Lines are broken in logical order (words measured shaped) and
//! turned visual for drawing: words of a right-to-left paragraph run right
//! to left, a run of right-to-left words inside a left-to-right paragraph is
//! reversed, and digits and Latin words keep their own order.

use og_i18n::{Direction, is_rtl_char, needs_shaping, paragraph_direction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Where shaped glyphs are drawn from: U+F0000 + glyph id (og-cook's font cook).
pub const GLYPH_PUA: u32 = 0xF_0000;

/// A font file for run-time shaping, with a cache of shaped words.
pub struct Shaper {
    ttf: Arc<Vec<u8>>,
    cache: Mutex<HashMap<String, Option<String>>>,
}

impl std::fmt::Debug for Shaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shaper({} bytes)", self.ttf.len())
    }
}

/// The most shaped words a font remembers before it starts over.
const CACHE_WORDS: usize = 4096;

impl Shaper {
    pub fn new(ttf: Arc<Vec<u8>>) -> Option<Arc<Shaper>> {
        rustybuzz::Face::from_slice(&ttf, 0)?;
        Some(Arc::new(Shaper {
            ttf,
            cache: Mutex::new(HashMap::new()),
        }))
    }

    /// One word shaped: private-use glyph characters in visual order, or
    /// `None` when the font cannot (the word is drawn as written).
    pub fn word(&self, word: &str, has: &dyn Fn(char) -> bool) -> Option<String> {
        if let Some(hit) = self.cache.lock().ok()?.get(word) {
            return hit.clone();
        }
        let shaped = (|| {
            let face = rustybuzz::Face::from_slice(&self.ttf, 0)?;
            let mut buf = rustybuzz::UnicodeBuffer::new();
            buf.push_str(word);
            buf.guess_segment_properties();
            let out = rustybuzz::shape(&face, &[], buf);
            let mut s = String::with_capacity(out.len() * 4);
            for g in out.glyph_infos() {
                let c = char::from_u32(GLYPH_PUA + g.glyph_id)?;
                if !has(c) {
                    return None;
                }
                s.push(c);
            }
            Some(s)
        })();
        if let Ok(mut c) = self.cache.lock() {
            if c.len() >= CACHE_WORDS {
                c.clear();
            }
            c.insert(word.to_string(), shaped.clone());
        }
        shaped
    }
}

impl PartialEq for Shaper {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ttf, &other.ttf)
    }
}

fn is_rtl_word(w: &str) -> bool {
    w.chars().any(|c| is_rtl_char(c) && c.is_alphabetic())
}

/// A word as drawn: shaped when it needs it and the font can, reversed
/// when it is right to left and unshaped (Hebrew), else as written.
fn draw_word(word: &str, shaper: Option<&Shaper>, has: &dyn Fn(char) -> bool) -> String {
    if word.chars().any(needs_shaping)
        && let Some(s) = shaper.and_then(|s| s.word(word, has))
    {
        return s;
    }
    if is_rtl_word(word) {
        return word.chars().rev().collect();
    }
    word.to_string()
}

/// Whether a line needs any work to be drawn (shaping or reordering).
pub fn needs_visual(line: &str) -> bool {
    line.chars().any(|c| needs_shaping(c) || is_rtl_char(c))
}

/// One line in drawing order (see the module docs). `default` is the UI's
/// direction, for a line with no strong character.
pub fn visual(
    line: &str,
    default: Direction,
    shaper: Option<&Shaper>,
    has: &dyn Fn(char) -> bool,
) -> String {
    if !needs_visual(line) {
        return line.to_string();
    }
    // Words and the spaces between them, in logical order.
    let mut tokens: Vec<(String, bool)> = Vec::new();
    for c in line.chars() {
        let space = c == ' ';
        match tokens.last_mut() {
            Some((t, s)) if *s == space => t.push(c),
            _ => tokens.push((c.to_string(), space)),
        }
    }
    let drawn: Vec<(String, bool, bool)> = tokens
        .iter()
        .map(|(t, space)| {
            (
                if *space {
                    t.clone()
                } else {
                    draw_word(t, shaper, has)
                },
                *space,
                !*space && is_rtl_word(t),
            )
        })
        .collect();
    let mut out = String::with_capacity(line.len() * 2);
    match paragraph_direction(line, default) {
        Direction::Rtl => {
            for (t, _, _) in drawn.iter().rev() {
                out.push_str(t);
            }
        }
        Direction::Ltr => {
            // Runs of right-to-left words (with the spaces between them) reverse.
            let mut i = 0;
            while i < drawn.len() {
                if drawn[i].2 {
                    let mut j = i;
                    while j + 1 < drawn.len()
                        && (drawn[j + 1].2
                            || (drawn[j + 1].1 && drawn.get(j + 2).is_some_and(|d| d.2)))
                    {
                        j += 1;
                    }
                    for (t, _, _) in drawn[i..=j].iter().rev() {
                        out.push_str(t);
                    }
                    i = j + 1;
                } else {
                    out.push_str(&drawn[i].0);
                    i += 1;
                }
            }
        }
    }
    out
}
