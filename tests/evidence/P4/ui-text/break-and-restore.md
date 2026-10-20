# Phase 4 UI text round: break and restore

Each change below was made to the code a test covers, the named tests were
run and failed as shown, and the code was restored (`git status` clean
afterwards). Apple M4 Pro, metal, 2026-10-20.

| Broken | Failed |
|---|---|
| og-render's `FontLayout::walk` skips the kerning pair | og-ui `text_gfx`: text-kerning differs, RMSE 0.18440, 5.943 % outliers (text-bidi, drawn with placements, still matched) |
| `FontMetrics::visual_at` lays every line out left to right in logical order (no bidi) | og-ui `rtl`: `paragraphs_order_their_runs_by_the_bidi_algorithm`, `arabic_words_are_shaped_into_joined_forms_in_visual_order` |
| rich text's segments neither reverse a right-to-left run's pieces nor lay it out right to left | og-ui `rich`: `runs_of_either_direction_reorder_inside_one_span`, `a_right_to_left_paragraph_runs_its_spans_by_the_bidi_algorithm`; `text_gfx`: text-bidi differs, RMSE 0.05836, 0.755 % outliers |
| the caret never blinks | og-ui `text`: `the_caret_blinks_by_the_tick` |
| key presses keep typing after a typed-text stream arrived | og-ui `text`: `a_typed_text_stream_takes_over_from_the_keys` |
| a composition does not replace the selection as it starts | og-ui `text`: `a_composition_shows_at_the_caret_until_it_commits` |
| labels are never cut | og-ui `text`: `labels_cut_with_an_ellipsis_where_they_do_not_fit` |
| the font cook's error correction disabled (`ErrorCorrectionMode::Disabled`) | og-cook `template`: 474 texel pairs whose interpolated median contradicts the true distance, against 224 with it (the test asserts clashes × 2 < the drawn glyphs) |
| the UI kit's Chinese font cooked with `runtime false` | ui-kit `locales`: "a frame drew 7 character(s) of text its fonts lack (FrameStats::missing_glyphs)" (the greeting's uncooked characters) |
