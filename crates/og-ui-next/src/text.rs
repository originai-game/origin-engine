//! Text leaves measured with og-text: its cooked font metrics, UAX #14
//! line breaks and Parley/HarfRust shaping, the engine's one text stack.
//! A leaf is measured only when its text or [`Run`] changes: the arena
//! keys rune-taffy's measure cache on both.

use crate::arena::Run;
use og_text::shape::Features;
use og_text::text::{FontMetrics, break_lines, widest_line};

/// The size of `text` run as `run`, given the width it may take.
pub fn measure(m: &FontMetrics, text: &str, run: &Run, width: Option<f32>) -> (f32, f32) {
    if run.nowrap || width.is_none() {
        let lines = text.split('\n').count().max(1);
        let w = widest_line(text, m, run.size, run.tracking, Features::NONE);
        return (w, lines as f32 * run.line);
    }
    let lines = break_lines(text, m, run.size, run.tracking, Features::NONE, width);
    let w = lines.iter().map(|l| l.width).fold(0.0, f32::max);
    (w, lines.len().max(1) as f32 * run.line)
}
