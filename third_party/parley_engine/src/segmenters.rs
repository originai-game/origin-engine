// Copyright 2026 Origin Engine contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Segmenters the host supplies (an Origin Engine patch to parley_engine
//! 0.12.0; third_party/parley_engine/PATCHED.md).
//!
//! Without the `complex-scripts` feature parley_engine breaks Chinese,
//! Japanese, Thai, Lao, Khmer and Burmese with the rules alone. A host that
//! loads ICU's dictionaries at run time (rather than linking the compiled
//! ones, about 3 MB) installs [`Segmenters`] once, and every analysis then
//! asks it for its word and line segmenters and which scripts a word
//! dictionary covers.

use icu_properties::props::Script;
use icu_segmenter::options::LineBreakOptions;
use icu_segmenter::{LineSegmenterBorrowed, WordSegmenterBorrowed};

/// The segmenters an analysis uses.
#[derive(Clone, Copy)]
#[expect(missing_debug_implementations, reason = "function pointers")]
pub struct Segmenters {
    /// The word segmenter, with whatever dictionaries are loaded.
    pub word: fn() -> WordSegmenterBorrowed<'static>,
    /// The line segmenter for these options, with whatever dictionaries
    /// are loaded.
    pub line: fn(LineBreakOptions<'_>) -> LineSegmenterBorrowed<'static>,
    /// Whether the word segmenter has a dictionary for a script (else
    /// UAX #29's WB999 is applied to its runs, as without dictionaries).
    pub has_word_dictionary: fn(Script) -> bool,
}

static SEGMENTERS: once_cell::race::OnceRef<'static, Segmenters> = once_cell::race::OnceRef::new();

/// Installs `segmenters` for every later analysis. The first call wins:
/// a host installs one set, whose functions see the dictionaries it loads.
pub fn set(segmenters: &'static Segmenters) {
    let _ = SEGMENTERS.set(segmenters);
}

pub(crate) fn get() -> Option<&'static Segmenters> {
    SEGMENTERS.get()
}
