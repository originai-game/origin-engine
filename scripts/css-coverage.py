#!/usr/bin/env python3
"""Generate docs/css-coverage.md, the conservative CSS checklist, from pinned WebDX data.

The file is wholly generated (docs/ui.md §8 says what its states mean).

No parser or source-name scan grants support. PROVEN contains reviewed output
assertions and their exact scope; everything else remains a missing proof.
Usage: python3 scripts/css-coverage.py [--archive web-features-3.45.0.tgz] [--check]
The archive is read from the host's gate cache, fetched into it once; exit
status 3 means it is neither cached nor fetchable, so nothing was checked.
"""

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
URL = "https://registry.npmjs.org/web-features/-/web-features-3.45.0.tgz"
SHA256 = "f1b743f337b877178e5ddc264151e9e6627af26096ba96cddda8ddd7e6907e25"
# One download per host, by the pinned hash (scripts/lock.sh's gate cache).
CACHE = Path(os.environ.get("OG_GATE_CACHE") or Path.home() / ".cache/og-gate") / f"web-features-{SHA256}.tgz"
UNAVAILABLE = 3
CHECKLIST = ROOT / "docs/css-coverage.md"

# Feature: (test module, test function, assertion scope). A feature row's
# support claim is limited to this scope, not every BCD subfeature it groups.
PROVEN = {}


def proof(features, module, test, scope):
    for feature in features.split():
        if feature in PROVEN:
            raise ValueError(f"duplicate proof: {feature}")
        PROVEN[feature] = (module, test, scope)


# Feature: one-line reason it cannot apply to a game engine's UI (print,
# cross-document navigation, WebXR, browser chrome). Neither a gap nor support.
NOT_APPLICABLE = {}


def not_applicable(features, reason):
    for feature in features.split():
        if feature in NOT_APPLICABLE:
            raise ValueError(f"duplicate not applicable: {feature}")
        NOT_APPLICABLE[feature] = reason


proof("aspect-ratio box-sizing width-height min-max-width-height fit-content-function min-max-content logical-properties padding",
      "css_layout", "the_box_model_sizes_as_css_does",
      "Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths.")
proof("flexbox flexbox-gap", "css_layout", "flex_lays_out_as_css_flexbox_does",
      "Computed grow/shrink shares, reverse/order, safe alignment, wrapping, row and column gaps.")
proof("flexbox-flex-wrap-balance", "css_layout", "balanced_flex_lines_are_even",
      "Computed line breaks of four equal items under wrap and balance, and balance through flex-flow.")
proof("grid subgrid", "css_layout", "grid_lays_out_as_css_grid_does",
      "Computed named/negative/spanning tracks, auto-fit/fill, fr minimums, areas, alignment and inherited subgrid tracks.")
proof("display-contents display-flow-root two-value-display display-table", "css_layout", "display_lays_out_as_css_display_does",
      "Computed contents flattening, inline flow-root, margin boundary, table spans and caption position; not border collapse/spacing.")
proof("multi-column column-fill column-span column-breaks", "css_layout", "multi_column_lays_out_as_css_multicol_does",
      "Computed balanced columns, width/count, spanning blocks, auto fill, explicit breaks and split text lines.")
proof("contain-size contain-inline-size contain-intrinsic-size", "css_contain", "size_containment_sizes_as_if_empty",
      "Computed empty size, intrinsic replacement dimensions and inline-only containment.")
proof("content-visibility", "css_contain", "content_visibility_skips_contents_off_screen",
      "Computed offscreen placeholder/remembered sizes, re-entry and hidden accessibility contents.")
proof("resize", "css_contain", "a_resize_handle_drags_the_size_onto_the_element",
      "Pointer drags change computed dimensions subject to axis, overflow and max-width.")
proof("object-fit object-position image-rendering", "css_interact", "images_take_object_fit_position_and_inherited_image_rendering",
      "Emitted image paint has contain fit, right-edge offset and inherited nearest sampling.")
proof("individual-transforms", "css_interact", "translate_percentages_move_by_the_elements_own_box",
      "Percentage translate changes computed box and hit testing; this proof covers translate, not individual rotate/scale.")
proof("conic-gradients", "css_paint", "repeating_gradients_repeat_the_stop_interval",
      "GPU pixel assertions for both halves of all four repeating conic quarters.")
proof("masks visibility", "css_paint", "masks_hide_invisible_boxes_and_fade_image_content",
      "GPU pixels prove hidden masked boxes do not paint, visible descendants retain the mask, and gradient masks fade both div and translucent image content.")
proof("border-image", "css_paint", "flex_ratios_absolute_fades_and_state_border_images_render",
      "A real button hover changes rendered border pixels from a shorthand source to a longhand gradient source; this does not prove all border-image modes.")
proof("focus-within", "css_interact", "focus_within_follows_real_focus_between_retained_ancestors",
      "Real clicks transfer focus between nested retained branches; computed widths prove shared ancestors stay focused and old ancestors reset. Parts and the legacy immediate builder are not covered.")
proof("object-view-box", "control_props", "an_image_shows_its_object_view_box",
      "Both build paths emit asymmetric inset/calc crop UVs; the removed -og-uv property is rejected.")
proof("list-style", "html_text", "list_markers_follow_css_and_renumber_on_reorder",
      "Both build paths assert marker text/shapes, position, inheritance and keyed reorder. Explicit initial resetting is tracked separately.")
proof("media-queries media-query-range-syntax prefers-reduced-motion", "documents", "media_queries_follow_dimensions_preferences_and_nested_intersections",
      "Computed widths at six viewport/preference combinations, including inclusive/exclusive range boundaries and nested queries.")
proof("nth-child nth-child-of empty", "documents", "pushed_screens_match_sibling_and_structural_selectors_on_instantiated_rows",
      "Computed widths/heights after keyed reorder and conditional removal prove filtered nth-child, nth-last-child, sibling and empty matching.")
proof("custom-properties", "custom_properties", "bound_custom_properties_feed_the_cascade",
      "Computed sizes follow bound/inherited variables, important precedence, null fallback and keyed insertion.")
proof("safe-area-inset", "custom_properties", "safe_area_insets_clear_the_hosts_covered_control",
      "Computed fixed and normal-flow positions prove viewport-relative env values, the engine root's existing safe-area padding, and covered-control inset updates in portrait/landscape. Insets do not subtract ancestor padding.")
proof("calc min-max-clamp attr round-mod-rem em-unit lh", "custom_properties", "calc_reduces_once_its_variables_are_known",
      "Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof.")
proof("borders outlines", "box_decor", "borders_pad_the_box_and_outlines_take_no_room",
      "Computed child offsets prove border widths/styles and logical sides affect layout; outline/offset take no room.")
proof("overscroll-behavior scroll-snap", "box_decor", "overscroll_contains_and_scroll_padding_insets_snaps",
      "Wheel input proves nested containment and scroll-padding changes the final mandatory snap offset.")
proof("scrollbar-color", "box_decor", "a_wheel_stops_at_snap_stop_always_and_scrollbar_color_inherits",
      "Emitted thumb is inherited red; repeated wheel input stops at the mandatory intermediate snap area.")
proof("overflow overflow-shorthand", "box_decor", "overflow_scrolls_and_clips",
      "Wheel input changes only the auto axis; computed viewport/content extents and offsets are asserted.")
proof("isolation mix-blend-mode clip-path clip-path-boxes rect-xywh shape-function", "css_effects", "css_effects_match_their_reference_frames",
      "References at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles.")
proof("filter svg-filters", "css_filters", "css_filters_match_their_reference_frames",
      "References at 1280/390 compare SVG filter graphs, primitive tiles and CSS function/url composition; this does not prove native glass backdrop chains.")
proof("font-size rem font-shorthand", "control_props", "root_relative_text_sizes_follow_the_root_not_the_parent",
      "Both builders emit 1.75rem at 24.5/36.75 px under 100/150 percent text scale, distinct from 1.75em under a 10px parent; shorthand is checked too.")
proof("text-shadow", "control_props", "scene_text_shadows_follow_the_font_size",
      "Both builders emit the same shadows as independently resolved px values for em/lh and inherited font size; none removes them.")
proof("absolute-positioning relative-positioning fixed-positioning sticky-positioning anchor-positioning",
      "css_layout", "positioning_places_as_css_positioning_does",
      "Both builders assert relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered.")
proof("writing-mode", "css_layout", "writing_modes_lay_out_as_css_writing_modes_does",
      "Both builders assert vertical-rl/lr, sideways-lr, flex axes, logical padding/margins, orthogonal flow and vertical float geometry.")
proof("layout-direction-override", "css_layout", "direction_lays_out_as_css_writing_modes_does",
      "Computed flex positions, logical/physical margins and absolute placement distinguish dir and CSS direction on LTR/RTL surfaces.")
proof("vertical-align", "css_layout", "vertical_align_places_inline_boxes_as_css_does",
      "Both builders assert sup/sub text baselines, inline-block baseline/middle/top, px/percentage offsets and line-box growth.")
proof("float-clear", "css_layout", "floats_lay_out_as_css_floats_do",
      "Computed opposing/stacked floats, paragraph exclusion, flow-root avoidance, clear:both and enclosing float height. Rectangular exclusion only.")
proof("viewport-units", "css_layout", "vi_and_vb_follow_the_roots_writing_mode",
      "Computed vi/vb dimensions follow the root writing mode on an 800x900 viewport, independently of child writing modes. Dynamic/small/large viewport units are not covered.")
proof("border-radius corner-shape", "box_decor", "corners_take_css_radii_and_shapes",
      "Both builders emit fill/stroke ellipse radii with overlap scaling, percentages, slash syntax and bevel corners.")
proof("box-shadow", "box_decor", "decorations_paint_in_css_order_over_any_draw",
      "Both builders assert outer shadow offset/blur, inset spread and paint order relative to the fill.")
proof("pointer-events", "css_interact", "the_pointer_passes_through_none_and_hidden",
      "Actual cursor routing through pointer-events:none and visibility:hidden, descendant overrides, URL priority/hotspot and accessibility presence. Immediate builder only; OS cursor loading is not covered.")
proof("text-align", "control_props", "a_button_label_takes_text_align",
      "Both builders assert emitted label positions for default centre, explicit start and menu item alignment; parent alignment does not override the button default.")
proof("font-family", "runs", "ordered_named_families_preserve_primary_glyphs_and_inherit",
      "Both builders assert exact glyph/font handles for inherited lists, rich spans, reversed named/generic order, names after generics and quoted generic names. Generics map to engine font roles, not installed system fonts.")
proof("gradients", "css_paint", "repeating_gradients_repeat_the_stop_interval",
      "GPU pixels prove repeating radial intervals with positive/negative starts, all eight conic half-quadrants and collapsed linear stops. Subpixel periods and unbounded stop counts are not covered.")
proof("gradient-interpolation", "css_paint", "gradients_mix_in_the_colour_space_they_name",
      "GPU pixels within deltaEOK 0.02 of CSS Color 4's formulas prove a full `in hsl longer hue` turn, `in hsl` red to blue the short way through magenta, and `in srgb` against the default Oklab. Other spaces and conic/radial forms are seen in review frames, not asserted.")
proof("margin", "html_text", "block_margins_follow_the_ua_stylesheet_and_collapse",
      "Both builders assert positive/negative sibling and parent-child margin collapse, padding/fixed-height boundaries, noncollapsing flex margins, UA em margins and keyed insertion.")
proof("text-wrap-balance text-wrap-pretty text-overflow", "html_text", "text_wrap_styles_and_overflow_strings_lay_out_as_css_does",
      "Both builders inspect emitted lines: balance reduces width spread at the same line count; pretty avoids the final single-word line; custom overflow text replaces the ellipsis on one clipped line.")
proof("stretch", "css_layout", "stretch_in_a_flex_line_is_its_stretch_fit_size",
      "Computed sizes of `stretch` across a flex line (filling it, the row laid out at the line's width), along one (the container's inner size, not floored by taller content nor shared with a sibling); a block's stretch height is asserted in css_layout::a_stretch_height_in_a_block_fills_its_containing_block. Grid areas and absolute boxes are not asserted.")
proof("text-indent text-indent-each-line text-indent-hanging line-break", "css_layout", "text_indent_and_line_break_lay_out_as_css_text_does",
      "Drawn lines of a shaped face show the first-line, hanging and each-line indents, a right-aligned indented line ending at its box, a max-content box growing by the indent, and line-break: anywhere filling a line mid-word. Strict/loose CJK punctuation rules are Parley's and not asserted; atlas-only and pixel faces are not indented.")
proof("container-queries", "documents", "container_queries_restyle_lists_grids_and_rive_frames_before_paint",
      "Viewport changes immediately change list/grid padding and Rive visibility under a named inline-size query. Pushed-screen path; style/scroll-state queries and container units are not covered.")
proof("scroll-driven-animations", "documents", "view_timelines_follow_the_subject_through_its_scroller",
      "Both builders assert width and translation at rest and after wheel input for an anonymous view() cover timeline. Named timeline scopes and all range keywords are not covered.")
proof("animations-css cubic-bezier-easing", "documents", "scoped_css_motion_uses_shared_curves_emits_sound_and_restores",
      "A cubic-bezier(0,1,0,1) keyframe animation reaches 7/8 displacement at 1/8 duration; twelve subsequent positions replay exactly after checkpoint restoration. Pushed-screen path, not every animation mode.")
proof("color", "dressing", "host_font_and_color_reach_button_ink_and_divider_output",
      "Emitted button text and divider fill match authored hex colors; an explicit button part color overrides inherited host ink. This does not cover every color syntax or space.")
proof("image-set", "css_paint", "image_sets_render_density_correct_backgrounds_masks_and_borders",
      "GPU pixels at DPR 1/2/1 prove density-correct background/mask tiles, MIME filtering before duplicate removal (including quoted parameters and malformed-type rejection), zero-resolution transparency, percentage border slices with auto widths and gradient selection. Resolution math covers calc/min/max/clamp with mixed x/dppx/dpi/dpcm units; retained output also tests fractional DPR.")
proof("background-image", "css_paint", "image_urls_resolve_files_not_atlas_names_or_handles",
      "Retained drawing selects the registered relative file and numeric filename, rejects a missing path despite an atlas basename collision, and reports the missing asset. css_paint_matches_its_reference_frames covers background tiling and gradients on the GPU. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks proves cooked SVG pixels, imported masks and inline cursor URL provenance.")
proof("cursor", "css_interact", "plain_boxes_choose_cursors_in_clipped_transformed_paint_order",
      "Both builders assert cursor inheritance over noninteractive boxes, passive overlays above buttons, pointer-events:none, hidden and overflow-clipped children, 90-degree transforms and image hotspots. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks checks cooked SVG cursor pixels on a plain div. css_interact::cursors_apply_only_inside_clip_paths proves, in both builders, cursors only inside circle(), polygon() over content-box, evenodd path(), shape(), objectBoundingBox url(#id) and an ancestor's inset() clip. Clip-path press hit testing and every cursor keyword are not covered.")
proof("field-sizing", "control_props", "fields_size_to_displayed_content_and_respect_constraints",
      "Both builders prove text-input value/placeholder sizing, caret-width empty inputs, textarea wrapping and newline heights, ignored rows, explicit dimensions, min/max width, noninheritance and removal of content sizing. Retained editing and checkpoint restore preserve focus/caret and update height. selects_files_and_vertical_fields_size_to_content proves, in both builders, a select fitting its shown option (fixed fills, explicit width wins), file inputs fitting chosen names while fixed keeps the default label width, and vertical-rl inputs/textareas sizing height to content with a line-wide width and caret-high empty height, wrapping into columns under a fixed height. Listbox selects and fixed sizing in vertical modes are not covered.")
proof("shape-outside shapes", "css_layout", "shape_outside_changes_line_exclusion_not_float_placement",
      "Both builders prove triangular line exclusion, unchanged float stacking/BFC avoidance/clear, and dynamic none/restoration. shape_outside_uses_the_whole_line_band_and_reference_box proves circles, ellipses, insets, margin/border/content boxes, margin-box clipping, degenerate insets and dropping a wide line along a slope. shape_margin_expands_by_distance_and_clips_to_the_float_box proves distance-based shape-margin, containing-block percentages, clipping and dynamic zero restoration. shape_images_wrap_lines_around_their_alpha proves, in both builders, url() alpha contours per pixel row, shape-image-threshold as number and percentage, an empty shape, shape-margin around them, content-box placement and the box kept without alpha; runtime ui_configure::shape_outside_images_take_their_contour_from_cooked_alpha reads a cooked SVG's alpha natively. Gradient images, Basis textures on the web, other syntax, elliptical box corners and writing modes remain unverified.")

proof("accent-color", "css_paint", "accent_color_inks_the_built_in_controls_and_what_they_draw_on_it_reads",
      "A checked box, a switch that is on, a chosen radio, a range's filled track and a progress fill draw the accent set in <style>, in the style attribute or on an ancestor, in both schemes and through a scheme switch (light-dark()); an unaccented sibling keeps the style's primary; the check mark reads at least 4.5:1 on a light and a dark accent. css_paint::accent_color_is_derived_as_the_styles_primary_seed proves the accent is the style's derivation of it as --seed-primary, its tick at least 4.5:1, in both schemes. A pack's own ::part art over the accent is not covered.")
proof("caret-color", "css_paint", "caret_color_paints_a_focused_fields_caret",
      "A focused single-line field's drawn caret takes its own caret-color, an inherited one, currentColor as the field's text colour, and auto as the style's primary. Text areas are not asserted.")
proof("color-mix-variadic contrast-color", "css_paint", "contrast_color_and_variadic_color_mix_reach_the_frame",
      "A three-colour color-mix() draws the three's average (CSS Color 5 §3.3) and a mix with no space is OKLab's; contrast-color(var(--panel)) draws white text on the dark panel and black once the scheme turns light. og-css's color_mix_takes_any_number_of_colours covers omitted and under-100% percentages, one colour and all-0% (transparent); contrast_color_picks_the_ink_that_reads_most holds the pick at √21:1 or more over a grid of grounds.")

# text and fonts
proof("before-after content attr-contents counters counter-set quotes", "css_text", "generated_content_counts_quotes_and_reads_attributes",
      "Drawn ::before/::after text proves reset/increment/set order with an upper-roman counter, nested counters() including a reset's following-sibling scope, depth-paired quotes, attr() and a later content: none. Element content replacement, url() images and ::before inside a box whose first content is text (drawn on a line of its own) are not covered.")
proof("text-transform", "css_text", "text_transform_rewrites_the_drawn_text",
      "Drawn text of uppercase (ß to SS), lowercase, capitalize after a hyphen, full-width (space to U+3000), full-size-kana and none. The language-sensitive (Turkic, Greek, Dutch IJ) cases are unit-tested in text_css, not here.")
proof("line-height letter-spacing", "css_text", "line_height_and_letter_spacing_compute_as_css_does",
      "Line box heights for an inherited number, an inherited percentage and an em length declared after the font size; letter-spacing's em computed once and inherited as px, added after each of three characters.")
proof("word-break", "css_text", "word_break_all_breaks_inside_words",
      "break-all fills 50px lines inside a word that normal keeps whole. keep-all on CJK text is not asserted.")
proof("text-decoration text-underline-offset text-underline-position", "css_text", "decoration_lines_sit_where_css_text_decoration_puts_them",
      "Drawn lines: a 5px offset below the baseline at an explicit 4px thickness and colour, from-font offset and thickness at Noto Sans's post underline, under at its hhea descent and a length offset from there, overline above the cap height, line-through under the x-height, double two lines a thickness apart, and propagation through a child's none. Auto offset and thickness are the engine's fractions of the em; left and right (vertical text) are not covered.")
proof("word-spacing", "css_text", "word_spacing_widens_each_word_separator",
      "Drawn glyph positions: a px and an em word-spacing after each space and no-break space, none inside a word, and the em computed once and inherited as px. Text in a fixed-advance or pixel font (laid out without Parley) is not covered.")
proof("text-stroke-fill", "css_text", "text_stroke_and_fill_paint_as_the_compat_standard_does",
      "Emitted text ink and outline effect: the fill colour over color, a stroke centred on the edge at its width and colour, currentcolor by default, and paint-order: stroke drawing only its outer half under the fill.")
proof("custom-ellipses", "html_text", "text_wrap_styles_and_overflow_strings_lay_out_as_css_does",
      "A one-line clipped text draws its text-overflow string in place of the ellipsis. Two-value (start and end) strings are not covered.")
proof("font-feature-settings font-kerning font-variant font-variant-caps font-variant-numeric font-variant-ligatures font-variant-position",
      "css_text", "font_variants_and_feature_settings_draw_the_fonts_glyphs",
      "Drawn glyph ids and advances match Noto Sans's own GSUB/hmtx: small-caps and all-small-caps, oldstyle, slashed-zero and diagonal fractions, super/sub digits, no-common-ligatures, the font-variant shorthand, inheritance, font-kerning: none's unkerned advances and font-feature-settings applied after a variant; each line measures as drawn. A cooked font draws the forms its import cooks or renders at run time. petite-caps, unicase and titling-caps without the font's features, and synthesised small caps or super/subscripts, are not covered.")
proof("font-variant-east-asian", "css_text", "east_asian_variants_take_the_fonts_full_width_forms",
      "Drawn glyphs of Noto Sans SC: full-width takes its one-em fwid forms of A, 1 and !, and proportional-width leaves proportional Latin as it is. The JIS and simplified/traditional and ruby forms are mapped but not asserted (the subset has none).")
proof("white-space white-space-collapse tab-size text-wrap", "css_text", "white_space_and_tab_size_process_the_source_as_css_text_does",
      "Drawn lines of a literal text: normal, pre, pre-line, pre-wrap (spaces hang), break-spaces (spaces keep their room), preserve-spaces, tabs to tab-size 4 and the default 8, and text-wrap: nowrap against wrap. Text with inline markup or bound expressions keeps HTML-collapsed white space; a tab-size length and text-wrap-style: stable are not covered.")
proof("overflow-wrap word-break-break-word", "css_text", "overflow_wrap_breaks_a_word_only_when_it_cannot_fit",
      "Drawn lines at 60px: anywhere, break-word and word-break: break-word move a long word to its own line, then break it within the box, while normal overflows; min-content is the widest letter under anywhere and the whole word under break-word.")
proof("text-align-last text-justify", "css_text", "justification_and_the_last_line_align_as_css_text_does",
      "Drawn line edges at 200px: justified lines end at the box edge with their room at spaces (auto) or one equal extra between every two characters (inter-character); none and auto's last line stay at the start; text-align-last right, center and justify-all. Plain horizontal text only: rich paragraphs and vertical lines are not justified here, and CJK inter-character spacing under auto is not covered.")
proof("font-face", "runs", "a_font_face_family_takes_its_faces_by_weight_and_range",
      "Glyph faces per character for @font-face weight ranges matched in CSS Fonts 4 §5.2's order, unicode-range and a declared family in a document's font-family list. src resolution is runtime ui_configure's; font-style, font-stretch, format() and tech() descriptors are not covered.")

# values, units, media and at-rules
proof("abs-sign calc-constants exp-functions trig-functions q-unit", "custom_properties", "math_functions_and_constants_take_their_mathematical_values",
      "abs(), sign() (of a negative and of 0), pi, e, infinity (under min()) and NaN (censored to 0, CSS Values 4 §10.9), pow() with a negative exponent, sqrt(), hypot(), log() with and without a base, exp(), sin()/cos()/tan() over deg, rad, grad, turn and a bare number, asin()/acos()/atan() and atan2() in the second quadrant, read through their mathematical values to ±0.5px; 101.6Q is 1in. og-css's own tests cover stepped round() and the other units.")
proof("cap ch ex ic rcap rch rex ric rlh", "custom_properties", "font_relative_units_read_the_faces_own_metrics",
      "With Inter Display as the body face, 1ch, 1ex and 1cap at 100px are its OS/2 `0` advance, x-height and cap height, 1ic is 1em (it has no 水: CSS Values 4 §6.1.1's fallback); the r- forms and rlh read the root's size and line height (the body type step). The ratios are the body face's: an element whose font-family names another face still reads the body face's.")
proof("viewport-unit-variants", "css_layout", "small_large_and_dynamic_viewport_units_read_their_own_viewport",
      "sv*, lv* and dv* (w, h, i, b, min, max) read the small, large and dynamic viewports the host reports, distinct from each other, and bare v* the dynamic one. vi and vb following the root's writing mode: css_layout::vi_and_vb_follow_the_roots_writing_mode.")
proof("inherit-value initial-value unset-value revert-value", "documents", "css_wide_keywords_inherit_reset_and_revert",
      "inherit takes the parent's width and font size; initial gives width auto and font-size medium (the root's size); unset inherits font-size and resets width; revert rolls a p's margin back to the user agent's 1em. revert-layer: custom_properties::a_var_that_resolves_to_revert_layer_rolls_the_cascade_back.")
proof("registered-custom-properties", "custom_properties", "registered_custom_properties_take_their_syntax_initial_value_and_computed_length",
      "@property: inherits false starts from the initial value under a parent that sets it, a value not matching <length> or <number> takes the initial value, a registered length computes to px where it is set (its child of another font size inherits the px), a \"*\" property with no initial value leaves var()'s fallback. CSS.registerProperty() is a script API a document has no scripts for.")
proof("container-style-queries container-scroll-state-queries", "documents", "container_style_and_scroll_state_queries_follow_their_container",
      "style(--t: dark) reads the nearest ancestor's computed value, a bound one re-read when its store changes, an unset property never matching; scroll-state(scrollable: bottom/top) and (scrolled: bottom) follow a wheel to the scroller's end. snapped and stuck are evaluated in og-ui's container unit tests, not asserted on output.")
proof("supports charset import", "documents", "supports_charset_and_import_conditions_gate_their_rules",
      "@supports declarations taken and refused, not/and/or, selector(), font-format() and font-tech(); rules after @charset apply; @import with layer() (an unlayered rule and a later rule in the layer win over it), a media condition and supports() true and false.")
proof("prefers-color-scheme prefers-contrast prefers-reduced-data prefers-reduced-transparency inverted-colors scripting resolution display-mode display-state dynamic-range color-gamut update interaction",
      "documents", "preference_and_display_media_features_follow_the_theme_and_surface",
      "Under two settings (light, standard contrast, opaque, 1dppx, sRGB, SDR, mouse; and dark, high contrast, reduced transparency, 2dppx, P3, HDR, touch), each value and the boolean form of prefers-color-scheme, prefers-contrast (less never), prefers-reduced-transparency, resolution in dppx/x/dpi/dpcm, color-gamut, dynamic-range, hover/any-hover and pointer/any-pointer match the theme and surface. inverted-colors, scripting, update, display-mode, display-state and prefers-reduced-data take the one answer the host gives (none, enabled, fast, fullscreen, fullscreen, no-preference).")
proof("device-queries resolution-compat supports-compat device-posture viewport-segments custom-media-queries", "documents", "device_compat_posture_and_custom_media_queries_answer_the_surface",
      "At two surfaces (800×900 2dppx fine, 500×900 1dppx coarse): device-width/height/aspect-ratio with min-/max- read the play surface; -webkit-device-pixel-ratio with min-/max- is resolution in dppx; (-webkit-transform-3d) is 1; device-posture is continuous and each viewport-segments count 1 (no host reports a fold; env(viewport-segment-*) is undefined, so it takes its fallback; the Viewport and DevicePosture script APIs have no scripts to serve). @custom-media names a query list, true and false, used before its definition, inside another and under not/or; the last definition wins; an undefined or self-referring name fails to cook.")
proof("supports-at-rule supports-named-feature", "documents", "supports_charset_and_import_conditions_gate_their_rules",
      "at-rule() is true for @container and @custom-media and false for @charset and @page, also under @import supports(); named-feature() is true for gamut-mapping only (anchors do not follow transforms; single-axis scroll containers are not established), false for an unknown name; env() is true for the safe-area and HUD insets, not viewport segments.")
proof("container-name-queries style-query-range-syntax", "documents", "container_style_and_scroll_state_queries_follow_their_container",
      "@container <name> with no condition applies under the named container and not without one; style() ranges compare a property with a literal, two properties, a two-sided range and a length against calc(0.125in), and a length against a number is false. if(style()) ranges are not covered.")
proof("calc-size", "css_layout", "calc_size_scales_its_basis_size",
      "width and height: max-content, min-content, fit-content, stretch and a block's auto width, an auto height in an auto-height parent, a percentage basis, a nested calc-size(), max-width after, content-box padding, and a flex row item's width, each the basis size through a calculation linear in size; non-linear calculations, size outside calc-size() and calc-size() in min-/max- bounds fail to cook. Not covered: flex-basis and bounds (refused), a flex item's cross axis (it aligns flex-start, CSS Flexbox 1 §8.3, not asserted), and animating to it (interpolate-size).")
not_applicable("url-cross-origin url-integrity url-referrer-policy",
               "Request modifiers for a network fetch: a game's url() names a file of its own cooked package, "
               "which is never fetched across origins, so CORS, subresource integrity and referrers do not arise.")
not_applicable("namespace",
               "A document's elements are in one namespace (a .svelte document is not XML), so @namespace prefixes "
               "select nothing apart; the rule is accepted and has no effect.")

# animation and interaction
proof("transitions transition-behavior starting-style linear-easing steps-easing display-animation animation-composition",
      "motion", "transitions_and_easings_sample_the_specs_curves_by_the_tick",
      "Laid-out widths sampled every tick for 64 ticks (60 a second, within 0.5 px) match independently worked curves for linear, ease (bisection-solved), steps(4, jump-start), steps(2, jump-both), linear() with a placed point and a 100ms delay; allow-discrete holds display and content-visibility shown for the whole 200ms where normal flips at once; an entrance starts from @starting-style; replace/add/accumulate of keyframe scale(3) over scale(2) give 3, 6 and 4; a mid-way checkpoint replays the frame exactly. Transitions of width only; other animated types are covered by the og-ui unit tests, not this proof.")
proof("transforms2d transform-box", "css_interact", "transforms_place_and_hit_by_the_specs_matrices",
      "Probe bounds match spec-derived matrices for rotate() about the default centre and a written corner origin, matrix(a..f), skewX(), and transform-box content-box versus border-box; the cursor follows the rotated box, not the laid-out one. fill-box/stroke-box/view-box (SVG boxes) are not asserted.")
proof("transforms3d", "css_interact", "transforms_place_and_hit_by_the_specs_matrices",
      "perspective with perspective-origin, translateZ() and rotateY() project bounds as d/(d−z) about the origin, and the pointer hits the projected quad (inverse of the homography), not the laid-out box. backface-visibility and preserve-3d sorting are not asserted.")
proof("view-transitions view-transition-class cross-document-view-transitions", "motion", "view_transitions_morph_a_named_element_by_its_groups_timing",
      "A layer replacing another is the navigation @view-transition { navigation: auto } opts into (without it nothing transitions): a view-transition-name group draws both images on lerp(old box, new box, easing) every tick, 400ms linear from ::view-transition-group(*.card) when view-transition-class is card and the UA's 0.25s ease when it is none. Cross-document means the engine's layer navigations, not page loads; the default root cross-fade and covers are tested in og-ui's layers unit tests, not here.")
proof("touch-action scroll-behavior", "css_interact", "touch_action_filters_pans_and_scroll_behavior_glides",
      "A diagonal touch drag on a coarse pointer pans only what the touched element allows: pan-y drops x, pan-left keeps only a finger moving left, pan-y inside pan-x intersects to none, auto pans both, and a mouse ignores it; a scroll into view under scroll-behavior: smooth glides monotonically and settles on the target within three seconds, while auto lands on the first tick. The used value intersects every ancestor's (the spec stops at the scroll container); pinch-zoom and manipulation are not asserted.")
proof("grid-animation clip-path-animatable", "motion", "grid_tracks_and_clip_paths_transition_by_their_values",
      "Every tick of a 600ms linear transition: grid-template-columns 100px 200px → 200px 100px lays out the first track at 100 + 100t (±0.5 px), and clip-path inset(0) → inset(50px) bounds where the cursor applies at 50t from each edge. Other track lists (repeat(), fr) and clip shapes are not asserted.")
proof("user-select", "user_select", "presses_select_words_lines_and_whole_elements_where_allowed",
      "Real presses: a double press selects the word, a triple the line, user-select: all the whole element at one press, none and a button's label nothing, with the I-beam only over selectable text. og-ui's default is none (a game's text is not selectable unless it says so), not CSS's auto.")
proof("selection", "user_select", "selection_ink_lines_and_shadows_cover_only_the_selected_run",
      "p::selection's color, text-decoration and text-shadow draw on the selected word's glyphs only, clipped to the run, with the rest of the line in its own ink and one underline under the word. Its background is judged in the GPU frames (user_select::the_selection_matches_its_reference_frames).")
proof("backdrop", "html_controls", "the_backdrop_pseudo_element_fills_the_scrim",
      "A modal dialog's ::backdrop draws the scrim in its background and lays it out by its box properties (padding-top moves the centred panel). html_controls::the_backdrop_inherits_from_its_dialog proves it inherits from the dialog: var() takes the dialog's custom property over its column's, and the column's through the dialog when the dialog sets none. A popover's ::backdrop is not drawn (the popover row); currentColor there is not asserted; fullscreen's is not applicable.")
proof("smooth", "css_interact", "images_take_object_fit_position_and_inherited_image_rendering",
      "image-rendering: smooth on an image under a pixelated parent emits a linearly sampled image paint, where the parent's value makes its sibling's nearest.")
proof("video-dynamic-range", "css_interact", "video_dynamic_range_follows_the_surface",
      "(video-dynamic-range: standard) matches on standard and extended-range surfaces, (… high) only on one with Viewport::high_range, and a change of surface restyles on the next frame.")
proof("placeholder", "css_interact", "placeholder_takes_its_pseudo_elements_color",
      "An empty input's and an empty textarea's placeholder text draws in ::placeholder's color while the field sets its own color, which the typed value of a filled input draws in. ::placeholder takes color only; other properties are refused.")
proof("anchor-positioning-animations", "motion", "anchor_insets_transition_by_their_resolved_lengths",
      "Every tick of a 600ms linear transition, left: anchor(--a right) → anchor(--b right) lays the box at the lerp of the two resolved lengths, right: anchor() mirrors it from the far side, width: anchor-size(--a width) → anchor-size(--c width) moves 40 → 100 px the same way, and an anchor moved mid-way is followed by its weight. A position-area change is not interpolated (it is discrete).")
proof("popover overlay", "motion", "popovers_enter_from_starting_style_and_leave_through_overlay",
      "A popovertarget toggle: the layer shows from the frame :popover-open matches and its width and opacity follow @starting-style's 0 to the open 200px and 1 on the 1000ms linear curve every tick (within 0.5 px and 0.02); closing with overlay 500ms allow-discrete keeps the layer up for 30 ticks while its width runs 200 to 100 on the same curve (whole pixels), then removes it; a styled ::backdrop covers the whole 800x600 page while the layer is up; a checkpoint taken during the hold replays every frame exactly. control_props::titles_and_popovers_show_against_their_controls covers popovertarget, light dismiss and interestfor hints. overlay holds a popover; a modal dialog's close keeps its own fade, not an authored overlay transition.")
proof("motion-path", "motion", "offset_paths_move_the_anchor_along_the_path",
      "In a 400x200 containing block, a 20x10 box's anchor (its transform origin, or offset-anchor: 0 0) lands on hand-worked points within 0.5 px: path(\"M0 100 L400 100\") at 400p every tick of a 1000ms linear offset-distance transition, path(\"M200 0 L200 200\") at 50px turned 90deg by offset-rotate: auto, circle(100px) at 25% on (300, 100) heading down, ray(90deg sides) from offset-position 0% 50% at 50% on (200, 100) with a fixed 0deg; the cursor follows the moved box and a checkpoint replays exactly. The containing block is the parent's border box; ray() contain and at <position>, other basic shapes, url() paths, coord-box and the offset shorthand are not supported.")
proof("interactivity", "css_interact", "interactivity_inert_stops_a_subtree_and_overrides_the_attribute",
      "A button inside a box with interactivity: inert sends nothing when pressed; a declared interactivity: auto on an [inert] box makes its button answer (the attribute is HTML's UA rule, which a declaration overrides); an [inert] box alone stays inert and a button outside both answers. Pointer presses only; keyboard focus skipping and find-in-page are not asserted.")
proof("interest-invokers", "control_props", "interest_delays_time_when_a_hint_shows_and_leaves",
      "At 60 ticks a second, an interestfor control with interest-delay: 200ms 300ms shows its hint exactly 18 ticks before one with interest-delay-start: 500ms and hides it exactly 18 ticks after one with interest-delay-end: 0s; a one-value shorthand sets both; the fixed engine latency is at most 3 ticks to show and 2 to hide. control_props::titles_and_popovers_show_against_their_controls covers hover and keyboard-focus interest. The interest and loseinterest events, :interest-source/:interest-target and long-press interest on touch are not covered.")
proof("appearance", "control_props", "dialogs_close_as_closedby_says",
      "appearance: none drops a dialog's own frame: its content sits at the dialog's corner with no padding. dressing::a_tight_field_takes_its_components_text_parts proves it on a field: one line of its text tall with the placeholder at its edge, no inset. appearance applies to dialogs and fields (auto | none); the compat keywords (button, checkbox, menulist-button, textfield…) and base-select are not taken.")
not_applicable("autofill", "og's fields are engine-drawn; no browser or OS autofill fills them, so there is no autofilled state to style.")
proof("caret-shape", "css_paint", "caret_shape_draws_a_bar_block_or_underscore_over_the_next_character",
      "A focused 20px field with its caret before \"i\": bar and auto draw the same thin bar at the insertion point; block is the bar's full height and exactly the font's advance of \"i\" wide, starting at the bar and drawn before the text; underscore is that advance wide, at most 1px thick, its bottom on the bar's bottom, drawn after the text. At the end of the text both take 1ch (the advance of \"0\") at the bar's position. The shape inherits from a column. Text areas, right-to-left text and the caret shorthand are not covered; the block does not recolour the character it covers.")
proof("scroll-initial-target", "css_interact", "a_scroller_starts_at_its_initial_target",
      "In a 100px scroller of 40px rows, the first scroll-initial-target: nearest row at 200px, with a 10px scroll-margin-top and the container's 5px scroll-padding-top, gives an offset of exactly 185 on the container's first frame, the row drawn 15px into the view; a later target loses; one near the end clamps to the 300px range; a target added to a container already shown leaves it at 0. Block-axis start alignment only (scroll-snap-align is not read); containers in floating layers are not covered.")
not_applicable("webxr-dom-overlays", "Styles the DOM overlay of a WebXR session; the engine renders its own UI in its world and has no WebXR DOM layer.")
not_applicable("window-controls-overlay window-drag", "Installed web-app title-bar integration (the OS window's caption buttons and drag regions); a game's UI is drawn inside its own surface, and the host owns its window.")
not_applicable("picture-in-picture", "Styles a video in the browser's floating picture-in-picture window, which is browser chrome outside the game's surface.")
not_applicable("fullscreen", "The Fullscreen API's :fullscreen and its ::backdrop style one element taken full screen by the browser; the game's surface is the screen, and the host decides whether its window is full screen.")
not_applicable("resizable", "@media (resizable) asks whether script may resize the browser window; the host, not a document, owns the game's window.")
not_applicable("target-text", "::target-text styles the text a URL scroll-to-text fragment matched on navigation; a game's documents are not navigated to by URL.")
not_applicable("ime-mode", "Obsolete, non-standard IME switch that browsers ignore in favour of inputmode; og-ui fields take inputmode and the platform's text input.")

# layout
proof("position contain-layout",
      "css_layout", "out_of_flow_boxes_are_placed_in_their_containing_block",
      "Computed boxes: an absolute box with right: 0; bottom: 0 sits in the corner of its nearest position: relative ancestor, past an unpositioned parent with margins; a fixed box with left: 3px; top: 4px sits at 3, 4 in an ancestor with a transform, will-change: transform, contain: layout or contain: paint, and at 3, 4 on the viewport with none. The holder's rect is its border box (og-ui keeps no border widths apart from padding), so insets measure from its outer edge, not the padding edge; position: sticky and anchor-positioned boxes are other rows.")
proof("scrollbar-gutter scrollbar-width",
      "css_layout", "scrollbar_gutter_and_width_reserve_the_bars_room",
      "Computed content boxes of overflow-y: auto boxes with nothing to scroll: scrollbar-gutter: stable keeps room on the end edge only, stable both-edges the same room on both, scrollbar-width: thin less room and none no room, scrollbar-gutter: auto none. The bar's width is og-ui's own (a theme spacing step), as CSS leaves it to the UA; a box that authors overflow on both axes draws its bars over its content and keeps no gutter, unlike a desktop browser's classic scrollbars.")
proof("contain-style", "css_text", "contain_style_keeps_counters_and_quotes_inside",
      "Drawn ::before text: inside contain: style, counter-increment nests a counter of the subtree's own (counters() reads 1.1, 1.2) and the outer counter goes on from 1 to 2 after it; a quote opened inside leaves the depth after it at 0. counter-set and a counter() read of the outer value inside are not asserted.")
proof("widows-orphans", "css_layout", "multi_column_lays_out_as_css_multicol_does",
      "Computed fragment and container heights of a 15-line text in two balanced columns: by default the break falls after line 8; an inherited widows: 9 moves it to after line 6 and orphans: 10 to after line 10. Breaks in pages and regions are not applicable; a text spanning three or more columns keeps the counts only at its first and last break.")
proof("grid-lanes", "css_layout", "grid_lanes_pack_items_within_their_fit_tolerance",
      "Computed item positions in display: grid-lanes with three column lanes: each auto-placed item goes to the least-filled lane, lanes within fit-tolerance (normal 1em, 0, infinite) of it tie, and a tie takes the first lane at or after the last item's end line, else the first. Row lanes (only grid-template-rows given) lay out as a grid; inline-grid-lanes, percentage tolerances and subgrid items in lanes are not asserted.")
proof("margin-trim", "css_layout", "margin_trim_drops_the_margins_at_a_blocks_edges",
      "Computed boxes of a flow-root block: margin-trim: block drops the first in-flow block child's top margin and the last one's bottom margin, keeping the margin between them; block-start drops only the first; an absolutely positioned first child is skipped. Box 4 defines margin-trim for block containers only: the inline keywords and flex/grid trimming (Safari's extension) are refused, and a descendant margin collapsed through the trimmed one is not trimmed.")
proof("table", "css_layout", "border_spacing_separates_table_cells",
      "Computed cell and table boxes: border-spacing: 4px 6px lies between the cells and between the outer cells and the table's padding edge, an inherited single value applies to both axes, border-collapse: collapse puts the cells edge to edge, and a table ignores gap. Spans, captions and row groups are html_tables_lay_out_without_author_css and the computed table tests. Collapsed borders are not merged into one line (each cell draws its own), and the caption lies inside the spacing rather than outside the table box.")
proof("display static-positioning physical-properties align-content-block fit-content",
      "css_layout", "boxes_take_display_position_insets_and_fit_content_as_css_does",
      "Computed boxes: inline and inline-block boxes share a line and a block takes its own, full width; display: none makes no box and keeps no room; position: static ignores its insets; opposite insets size an auto box and an auto inset follows the other; block align-content: end; width: fit-content is max-content where it fits and the available width where it does not. fit-content on height and the min/max properties is not asserted.")
proof("z-index contain contain-paint overflow-clip", "css_contain", "z_index_orders_paint_and_clip_and_contain_cut_it",
      "Drawn order of three overlapping positioned boxes is z-index -1, auto, 2 against their source order; contain: paint, overflow: clip, contain: strict and contain: content push a clip at their box and contain: none does not; strict sizes as if empty and content keeps its height; overflow: clip makes no scroll container. z-index: 0 and auto order alike (both are level 0, so a positioned z-index: 0 makes no stacking context); overflow-clip-margin is a gap of its own. Stacking contexts: css_contain::z_index_paints_in_the_nearest_stacking_context proves z-index 1 lifting a child over its parent's later sibling and -1 putting it under the parent's background, and opacity, transform, filter, mask, clip-path, mix-blend-mode, isolation, will-change, contain: paint and a level of its own each keeping it in, as each comes and goes. Scroll containers, shaped overflow clips, dressed parts and game draws also keep their levels in.")
proof("justify-self-block", "css_layout", "justify_self_aligns_block_level_boxes",
      "Computed x and width of block-level boxes: justify-self end with a fixed width, center with an auto width shrunk to fit-content, auto and normal at the start, and justify-items: end as the default that justify-self: start overrides. Absolutely positioned boxes and right-to-left blocks are not asserted.")
proof("line-clamp", "css_layout", "line_clamp_cuts_a_texts_lines",
      "Computed heights: line-clamp: 2 keeps two lines of a <p> and a <text> without text-overflow, the box those lines high; a text within the limit keeps its lines. A <div> holding text directly refuses line-clamp (it applies to text-bearing elements), and the drawn ellipsis is not asserted.")
proof("overflow-overlay", "css_contain", "z_index_orders_paint_and_clip_and_contain_cut_it",
      "overflow: overlay and overflow-y: overlay make scroll containers whose view and content heights are auto's.")
not_applicable("frame-sizing",
               "Sizes an <iframe> to its embedded document: a game's UI embeds no other documents.")
not_applicable("page-breaks page-break-aliases page-orientation page-selectors page-setup",
               "Paged media for printing: a game's UI is laid out on a screen, never paginated, so there are no pages to break, select, rotate or set up. Column breaks are multi-column's (column-breaks).")

# colour, paint and effects
proof("alpha rgb hsl hwb lab oklab named-color color-function color-mix relative-color system-color system-color-accentcolor background-color",
      "css_paint", "every_colour_syntax_draws_the_colour_the_spec_gives",
      "Emitted background-color fills match the specs' worked values (#7d2329 for lab/lch/oklab/oklch, hwb by its formula) for hex alpha, rgb() percentages and legacy commas, hsl() turns, color(srgb|srgb-linear), premultiplied color-mix() with transparent, relative hsl/rgb/oklch, and alpha() with and without calc(alpha). System colours (Canvas, CanvasText, AccentColor, AccentColorText, ButtonFace, ButtonBorder, Mark, Highlight) are the style's palette roles; the other system keywords map the same way but are not each asserted. Wide-gamut color() spaces are covered by og-css's P3 tests, not here.")
proof("currentcolor light-dark light-dark-image color-scheme",
      "css_paint", "current_color_and_light_dark_resolve_in_the_element",
      "In both style schemes, currentColor is the declared or inherited colour in a colourless border, an outline, gradient stops, a background and color itself (color-mix(currentColor) inherits); light-dark() of colours and of url() images follows color-scheme light/dark on the element or an ancestor, and the style's scheme under `light dark` or none. color-scheme does not restyle built-in controls or scrollbars beyond their colours.")
proof("background background-attachment background-blend-mode background-clip background-origin background-position background-repeat background-size",
      "css_paint", "background_layers_size_place_repeat_clip_and_blend_as_css_does",
      "Emitted layers read through og-render's own placement land where CSS Backgrounds 3 arithmetic puts them: edge-offset and percentage positions, background-position-x over the shorthand, repeat-x, space and round tiling, cover and contain, content-box origin, padding-box colour clip, fixed attachment from the viewport origin, and per-layer blend modes with the first layer drawn last. background-attachment: local and background-clip: text are not asserted here (text clips the element's own text only).")
proof("background-clip-border-area", "css_paint", "border_area_paints_a_layer_only_under_the_border",
      "GPU pixels prove a border-area colour and an image layer over a colour paint only under a 6 px transparent border, leaving the padding box empty or the colour beneath. Rounded and shaped corners follow the box's corner geometry but are not asserted.")
proof("opacity outline crisp-edges clip path-shape",
      "css_paint", "opacity_outline_crisp_edges_and_clips_draw_as_css_says",
      "Emitted output: opacity as a number and a percentage; outline thin/medium/thick widths (1/3/5 px), dashed, double's two thirds-wide lines spanning 5 px, outline-offset and none; image-rendering: crisp-edges sampling nearest; CSS 2 clip: rect() with commas, spaces and auto edges on absolute boxes only; clip-path path() points and both fill rules. path() for shape-outside, offset-path and the d property is not covered.")
proof("opacity-svg paint-order mask-type mask-border",
      "css_paint", "svg_paint_opacities_order_and_mask_types_draw_as_svg_says",
      "SVG fill-opacity and stroke-opacity (attributes, percentages and CSS) scale only their paint; paint-order draws a shape's stroke under its fill, on the shape or inherited, with omitted paints in default order; a <mask>'s mask-type attribute (alpha vs luminance) changes the emitted gradient mask; mask-border slices a gradient's border area. Text paint-order, mask-type in a stylesheet rule and non-gradient masks are not covered.")
proof("svg", "inline_svg", "inline_svg_matches_its_reference_frames",
      "References at 1280/390 compare inline SVG paths, polygons, a url(#) gradient fill, even-odd fill-rule, a pathLength dash offset and bound path data in CSS-sized boxes. css_paint::svg_paint_opacities_order_and_mask_types_draw_as_svg_says adds paint opacities, paint-order and mask types. The SVG DOM interfaces, <foreignObject>, <a>, <script>, <switch> and systemLanguage are not covered.")
proof("image-orientation", "css_paint", "image_orientation_turns_images_by_their_exif_or_draws_them_as_stored",
      "Emitted images read through og-render's own placement against a 2x1 texture carrying EXIF orientations 1, 3, 6 and 8: from-image turns it as EXIF defines (6 red on top, 8 blue on top, 3 blue on the left) with the turned natural size under object-fit: none; none, inherited from the parent, samples the texels as stored; a background follows the property too. og-cook textures::exif_orientation_is_kept_beside_the_texels_as_stored: the cook keeps the orientation beside unturned texels, a streamed tail too. The deprecated <angle> || flip form is refused; sprites and materials sample as stored; the GPU shader path is not pixel-checked.")
not_applicable("color-adjust print-color-adjust",
               "Print colour economy: a game draws to its screen, never to paper or a printer's economy mode.")
not_applicable("paint",
               "The CSS Painting API runs a browser JavaScript paint worklet (CSS.paintWorklet, PaintRenderingContext2D); a document has no worklet runtime, and engine art comes from materials, vectors and images.")
not_applicable("svg-discouraged",
               "SVG 1.1 features SVG 2 removed (xlink: attributes, SVG fonts, SVGPathSeg and other DOM methods, zoomAndPan, baseProfile); documents are SVG 2 markup with no script DOM.")

# selectors and cascade
proof("selectors has is where not nth-of-type root scope scope-pseudo case-insensitive-attributes case-sensitive-attributes lang dir-pseudo state link-selectors nesting input-selectors indeterminate default read-write-pseudos placeholder-shown open-pseudo",
      "css_selectors", "selectors_select_as_css_does",
      "Computed widths, each against a decoy in the other state: type, class, id and attribute selectors with the child, adjacent and general sibling combinators; :has(> …); :is(); :not(); :where() matching with zero specificity; :nth-of-type(2) among other tags; a :root custom property; :scope; @scope (.card) to (.inner) stopping at its limit; [k=v i] and [k=v s]; :lang(fr) on fr-CA; :dir(rtl); :state(on) bound true and false; :any-link; nesting's &; :checked, :disabled, :enabled, :default and :indeterminate inputs; input:read-only; :placeholder-shown; :out-of-range; details:open. :visited and :link's history are not modelled (every link is unvisited).")
proof("form-validity-pseudos", "input_types", "html_constraints_readonly_rows_and_label_checkboxes",
      "Computed widths: an empty required field and one under its minlength match :invalid, valid and empty optional fields do not. :valid, :in-range and :user-valid are not asserted here.")
proof("cascade-layers", "style_pack", "a_rule_ranks_by_its_tree_context_then_its_layer",
      "A built-in component's own rule (og-components) loses to a pack's (og-pack), a game's own unlayered rule beats it, and !important reverses the layers; a rule's tree context ranks before its layer (a button's own plate over the style's ::part(), a document's ::part() over the plate). Anonymous and nested layers are covered by documents::supports_charset_and_import_conditions_gate_their_rules.")
proof("shadow-parts", "dressing", "a_part_rule_reads_its_hosts_place_in_the_document",
      "A built-in control's parts are a shadow tree under its element: .shop button::part(base) and ul > li:first-child button::part(base) widen only the button they reach, from a document and from a pack, and follow a class change. part lists (one element under two names) are covered by the role parts' tests; exportparts is not built (no component re-exports a part).")
not_applicable("time-relative-selectors",
               "WebVTT cue timing: :past and :future match text-track cues during media playback, and a game's <video> carries no text tracks.")
not_applicable("autonomous-custom-elements",
               "customElements.define() is a script API. A game's components are .svelte documents used by their own tag, with no script to register a class.")

MISSING = {
    "will-change": "will-change on a stacking property draws a group layer, but neither half of its contract is observable yet: a transform does not make a containing block for fixed descendants, and z-index never lifts a child above its parent's later siblings (both routed to their streams).",
    "file-selector-button": "Accepted and styles nothing: a file input's button is not a stylable box.",
    "active-view-transition": ":active-view-transition-type() qualifies only ::view-transition-* rules; :active-view-transition does not match elements during a transition (a selectors-side gap).",
    "view-transitions-element-scoped": "No view-transition-scope or element-scoped transitions: a transition captures a whole layer.",
    "scroll-markers": "No ::scroll-marker or ::scroll-marker-group: the properties and pseudo-elements are refused.",
    "scroll-marker-targets": "No :target-current, :target-before or :target-after (needs scroll markers; the selectors are engine-gaps').",
    "scroll-target-group": "No scroll-target-group: anchor links do not act as scroll markers.",
    "scroll-buttons": "No ::scroll-button(): a scroll container generates no buttons.",
    "scroll-axis-lock": "No scroll-axis-lock property; how og-ui's pan treats a mostly-one-axis drag is not asserted either.",
    "scroll-triggered-animations": "No timeline-trigger or animation-trigger: scroll timelines drive animations (scroll-driven), but none starts or stops one.",
    "overflow-anchor": "No scroll anchoring: content inserted above the scrollport moves what is shown.",
    "details-content": "No ::details-content: a details element's content is not one stylable box.",
    "customizable-select": "::picker(select) styles the open list (control_props::a_select_takes_aria_invalid_and_its_picker_height), but appearance: base-select, ::picker-icon, ::checkmark and <selectedcontent> are not taken.",
    "highlight": "No ::highlight() or highlight registry: only ::selection paints a text range.",
    "webvtt": "No WebVTT captions or ::cue: a video draws no text track.",
    # layout
    "interpolate-size": "No intrinsic-size interpolation implementation or output proof.",
    "backdrop-filter": "Ordinary staged backdrop chains have fixtures; native glass rejects chains requiring intermediate clamps, drop-shadow and SVG url filters.",
    "background-clip-text": "background-clip: text clips to the element's own glyphs only; CSS also clips to its descendants' text, which needs the subtree's text grouped where og-ui draws it (ui/commands.rs).",
    "filter-function": "No filter(<image>, <filter-function>) image: background and mask layers take url(), image-set(), gradients and light-dark() only.",
    "cross-fade": "No cross-fade() image: a box layer draws one image, and a premultiplied sum of several needs layers that hold more than one.",
    "element": "No element() image; feImage's element images exist for SVG filters but no background or border layer reads a live element.",
    "border-shape": "No border-shape: borders, border images, outlines and shadows follow border-radius and corner-shape only.",
    "box-decoration-break": "Inline boxes and column fragments draw no per-fragment decorations, so neither slice nor clone is implemented.",
    "host": ":host and :host() parse and match through the selectors crate, but a component instance's host element is not styled, so no output proves it.",
    "has-slotted": "A component's content is passed as snippets, not <slot> elements, so no :has-slotted output exists.",
    "slot": "A component's content is passed as snippets ({@render}), not <slot> elements, and no <slot> output is proved.",
    "host-context": ":host-context() is not parsed (Selectors 4 dropped it, and Firefox never shipped it).",
    "heading-selectors": ":heading and :heading() are not parsed yet.",
    "revert-rule": "The revert-rule keyword is not read yet.",
    "media-pseudos": "<video> plays footage, but :playing, :paused and the other media pseudo-classes are not parsed yet.",
}

if overlap := PROVEN.keys() & MISSING.keys():
    raise ValueError(f"a proof must state its remaining gaps, not hide MISSING: {sorted(overlap)}")
if overlap := NOT_APPLICABLE.keys() & (PROVEN.keys() | MISSING.keys()):
    raise ValueError(f"a feature is either not applicable or proved/missing: {sorted(overlap)}")

PART_PROVEN = {}


def part_proof(features, module, test, scope):
    for feature in features.split():
        PART_PROVEN.setdefault(feature, []).append((module, test, scope))


part_proof("shadow-parts width-height individual-transforms font-size color opacity",
    "dressing", "button_parts_keep_separate_boxes_and_host_event_ownership",
    "Button base dimensions, independent host/part translate, inherited font size/color, "
    "opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture.")
part_proof("border-radius",
    "box_decor", "border_radius_rounds_a_dressed_button",
    "Emitted button base radius replaces its default in both immediate and retained documents.")
part_proof("font-family",
    "runs", "ordered_named_families_preserve_primary_glyphs_and_inherit",
    "Both builders emit button::part(base) glyphs from ordered named families, including a named fallback after an intervening generic.")
part_proof("shadow-parts width-height padding borders background-color",
    "dressing", "tabs_and_select_parts_keep_boxes_states_and_host_input",
    "Tab indicator height, select group height, list padding+border inset, group text padding and distinct tab/option fills are asserted after real clicks. Semantic roles remain on hosts.")
part_proof("not opacity border-image",
    "dressing", "part_host_negation_and_part_states_select_distinct_paint",
    "Four primary/hover combinations assert exact resolved opacity and border-image presence for button:not(.primary)::part(base). No document-selector claim.")
part_proof("animations-css opacity",
    "dressing", "retained_part_nodes_animate_without_rebuilding_the_host",
    "A retained button part emits alpha 0.2/0.5/0.8 at ticks 0/6/12 for a 200ms linear animation, then paints nothing while idle. Other animation modes are not covered.")
part_proof("image-set", "css_paint", "image_sets_render_density_correct_backgrounds_masks_and_borders",
    "GPU pixels prove button::part(base) reselects its background at DPR 1/2/1 while preserving an 8 CSS px tile. Other part image properties are not separately proved.")


def proof_source(module, test, kind):
    """The file of `module` that defines `test`: og-ui's, or the frame tests' (og-tests-ui)."""
    for crate in ("og-ui", "og-tests-ui"):
        source = f"crates/{crate}/tests/{module}.rs"
        if (ROOT / source).exists() and f"fn {test}(" in (ROOT / source).read_text():
            return source
    raise ValueError(f"missing {kind} entry point: {module}::{test}")


def webdx(path):
    """The pinned archive's bytes, hash-checked on every read: from path,
    else the cache, else fetched and cached. A cached copy with another
    hash is dropped and fetched again."""
    if path:
        data = path.read_bytes()
    else:
        try:
            data = CACHE.read_bytes()
        except OSError:
            data = None
        if data is not None and hashlib.sha256(data).hexdigest() != SHA256:
            print(f"css-coverage: {CACHE} does not match the pinned hash; fetching it again", file=sys.stderr)
            CACHE.unlink()
            data = None
        if data is None:
            try:
                data = urllib.request.urlopen(URL, timeout=60).read()
            except OSError as error:
                print(f"css-coverage: UNCHECKED: no cached WebDX archive and {URL} could not be fetched ({error})",
                      file=sys.stderr)
                sys.exit(UNAVAILABLE)
            if hashlib.sha256(data).hexdigest() == SHA256:
                CACHE.parent.mkdir(parents=True, exist_ok=True)
                part = CACHE.with_name(f"{CACHE.name}.{os.getpid()}")
                part.write_bytes(data)
                part.replace(CACHE)
    if hashlib.sha256(data).hexdigest() != SHA256:
        raise ValueError("WebDX archive hash changed")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    archive = webdx(args.archive)
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
        data = json.load(package.extractfile("package/data.json"))["features"]
    features = {key: value for key, value in data.items()
                if any(name.startswith("css.") for name in value.get("compat_features", []))}
    if len(features) != 454 or (PROVEN.keys() | MISSING.keys() | NOT_APPLICABLE.keys() | PART_PROVEN.keys()) - features.keys():
        raise ValueError("feature inventory drift")
    lines = ["# Standard CSS output-proof checklist",
             "",
             "Generated by `scripts/css-coverage.py` from the proofs it lists; never edit it by hand. "
             "What the checklist is for, and why the dialect is closed, is [the UI contract's §8](ui.md).",
             "",
             "Source: [WebDX web-features 3.45.0](https://www.npmjs.com/package/web-features/v/3.45.0). "
             "All 454 entries with a `css.*` compatibility key are included: 257 widely available (high), "
             "56 newly available (low), and 141 not Baseline. Including the newer entries keeps game-UI "
             "requirements visible rather than treating Baseline as a ceiling.",
             "",
             "**Supported means the output assertion named in the row proves the listed scope.** "
             "It does not claim every syntax or subfeature grouped under that WebDX name. "
             "**Missing means an implementation gap or a missing output proof**, distinguished in the reason. "
             "A missing proof is not evidence that the implementation is absent. "
             "**Not applicable means the feature cannot apply to a game engine's UI** "
             "(print, cross-document navigation, browser chrome), with the reason in the row. Parser acceptance, "
             "a property name in source, and an entire test file are not support evidence. "
             "This is a conservative, incomplete conformance inventory; unresolved rows stay visible.",
             "",
             "Part applicability is proved only in the scopes named below: an element test "
             "does not prove that a widget `::part` lays out, transforms or paints the property. "
             "Inherited, pseudo-element, environment and lifecycle features need their own applicable "
             "part scenarios; there is no blanket all-properties claim.",
             "",
             "Regenerate with `python3 scripts/css-coverage.py`; `--check` rejects document drift. "
             "The archive is fetched once per host into the gate cache (`~/.cache/og-gate`), or passed with `--archive <web-features-3.45.0.tgz>`; its SHA-256 is pinned in the script and checked on every read. "
             "Proof entries are manually reviewed assertions, not name matches. Tests below live in "
             "`crates/og-ui/tests/` (what the UI builds) or `crates/og-tests-ui/tests/` (the frames drawn of it), "
             "each one integration test binary; GPU proofs need a GPU and are judged against the frames recorded on the GPU host (the L4, Vulkan; docs/ci.md §Reference frames).",
             "",
             "| WebDX feature | Baseline | Document elements | Widget ::part | Proof scope or missing reason |",
             "| --- | --- | --- | --- | --- |"]
    for key, feature in sorted(features.items()):
        baseline = feature["status"]["baseline"]
        label = {"high": "Widely", "low": "Newly", False: "Not Baseline"}[baseline]
        if key in PROVEN:
            module, test, scope = PROVEN[key]
            source = proof_source(module, test, "proof")
            state = "Supported (scope →)"
            reason = f"[`{module}::{test}`](../{source}). {scope}"
        elif key in NOT_APPLICABLE:
            state = "Not applicable"
            reason = NOT_APPLICABLE[key]
        else:
            state = "Missing"
            reason = MISSING.get(key, "Output proof not yet reviewed; implementation status unverified.")
        part_state = "Not applicable" if key in NOT_APPLICABLE else "Missing"
        for module, test, scope in PART_PROVEN.get(key, []):
            source = proof_source(module, test, "part proof")
            part_state = "Supported (scope →)"
            reason += f" Part proof: [`{module}::{test}`](../{source}). {scope}"
        lines.append(f"| `{key}` | {label} | {state} | {part_state} | {reason} |")
    new = "\n".join(lines) + "\n"
    if args.check:
        if not CHECKLIST.exists() or CHECKLIST.read_text() != new:
            raise SystemExit("CSS checklist drift: run scripts/css-coverage.py")
    else:
        CHECKLIST.write_text(new)
    print(f"454 CSS entries: {len(PROVEN)} scoped output proofs; {len(NOT_APPLICABLE)} not applicable; "
          f"{454 - len(PROVEN) - len(NOT_APPLICABLE)} missing/gap rows")


if __name__ == "__main__":
    main()
