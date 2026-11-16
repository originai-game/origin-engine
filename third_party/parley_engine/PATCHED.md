# parley_engine 0.12.0, patched

Upstream: <https://github.com/linebender/parley> (`parley_engine` 0.12.0,
Apache-2.0 OR MIT; LICENSE-APACHE and LICENSE-MIT here), used through the
workspace's `[patch.crates-io]`.

One change: `src/segmenters.rs`. Without the `complex-scripts` feature an
analysis asks the segmenters a host installed (`segmenters::set`) for its
word and line segmenters and which scripts have a word dictionary, instead
of the rule-only ones. og-text installs og-i18n's, which load ICU's
dictionaries from the game's font packs (og_i18n::segmenter), so the
compiled dictionaries (about 3 MB) stay out of the engine. The rest is
upstream's code, unchanged, but for a `once_cell` (race) dependency that
holds the hook (`src/analysis.rs`: `word_segmenter`,
`line_segmenter_impl` and `has_word_dictionary` consult the hook).
