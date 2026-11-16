#!/usr/bin/env python3
"""Generate docs/ui.md's conservative CSS checklist from pinned WebDX data.

No parser or source-name scan grants support. PROVEN contains reviewed output
assertions and their exact scope; everything else remains a missing proof.
Usage: python3 scripts/css-coverage.py [--archive web-features-3.45.0.tgz] [--check]
"""

import argparse
import hashlib
import io
import json
from pathlib import Path
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
URL = "https://registry.npmjs.org/web-features/-/web-features-3.45.0.tgz"
SHA256 = "f1b743f337b877178e5ddc264151e9e6627af26096ba96cddda8ddd7e6907e25"
START = "<!-- css-coverage:start -->"
END = "<!-- css-coverage:end -->"

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
proof("text-decoration text-underline-offset", "css_text", "decoration_lines_sit_where_css_text_decoration_puts_them",
      "Drawn lines: a 5px offset below the baseline at an explicit 4px thickness and colour, overline above the cap height, line-through under the x-height, double two lines a thickness apart, and propagation through a child's none. Auto offset and thickness are the engine's fractions of the em, not the font's post/OS-2 metrics.")
proof("text-stroke-fill", "css_text", "text_stroke_and_fill_paint_as_the_compat_standard_does",
      "Emitted text ink and outline effect: the fill colour over color, a stroke centred on the edge at its width and colour, currentcolor by default, and paint-order: stroke drawing only its outer half under the fill.")
proof("custom-ellipses", "html_text", "text_wrap_styles_and_overflow_strings_lay_out_as_css_does",
      "A one-line clipped text draws its text-overflow string in place of the ellipsis. Two-value (start and end) strings are not covered.")
proof("font-feature-settings font-kerning font-variant font-variant-caps font-variant-numeric font-variant-ligatures font-variant-position",
      "css_text", "font_variants_and_feature_settings_draw_the_fonts_glyphs",
      "Drawn glyph ids and advances match Noto Sans's own GSUB/hmtx: small-caps and all-small-caps, oldstyle, slashed-zero and diagonal fractions, super/sub digits, no-common-ligatures, the font-variant shorthand, inheritance, font-kerning: none's unkerned advances and font-feature-settings applied after a variant; each line measures as drawn. A cooked font draws the forms its import cooks or renders at run time. petite-caps, unicase and titling-caps without the font's features, and synthesised small caps or super/subscripts, are not covered.")
proof("font-variant-east-asian", "css_text", "east_asian_variants_take_the_fonts_full_width_forms",
      "Drawn glyphs of Noto Sans SC: full-width takes its one-em fwid forms of A, 1 and !, and proportional-width leaves proportional Latin as it is. The JIS and simplified/traditional and ruby forms are mapped but not asserted (the subset has none).")
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
proof("prefers-color-scheme prefers-contrast prefers-reduced-data prefers-reduced-transparency inverted-colors scripting resolution display-mode dynamic-range color-gamut update interaction",
      "documents", "preference_and_display_media_features_follow_the_theme_and_surface",
      "Under two settings (light, standard contrast, opaque, 1dppx, sRGB, SDR, mouse; and dark, high contrast, reduced transparency, 2dppx, P3, HDR, touch), each value and the boolean form of prefers-color-scheme, prefers-contrast (less never), prefers-reduced-transparency, resolution in dppx/x/dpi/dpcm, color-gamut, dynamic-range, hover/any-hover and pointer/any-pointer match the theme and surface. inverted-colors, scripting, update, display-mode and prefers-reduced-data take the one answer the host gives (none, enabled, fast, fullscreen, no-preference).")
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
not_applicable("webxr-dom-overlays", "Styles the DOM overlay of a WebXR session; the engine renders its own UI in its world and has no WebXR DOM layer.")
not_applicable("window-controls-overlay window-drag", "Installed web-app title-bar integration (the OS window's caption buttons and drag regions); a game's UI is drawn inside its own surface, and the host owns its window.")
not_applicable("picture-in-picture", "Styles a video in the browser's floating picture-in-picture window, which is browser chrome outside the game's surface.")
not_applicable("fullscreen", "The Fullscreen API's :fullscreen and its ::backdrop style one element taken full screen by the browser; the game's surface is the screen, and the host decides whether its window is full screen.")
not_applicable("resizable", "@media (resizable) asks whether script may resize the browser window; the host, not a document, owns the game's window.")
not_applicable("ime-mode", "Obsolete, non-standard IME switch that browsers ignore in favour of inputmode; og-ui fields take inputmode and the platform's text input.")

# layout
proof("display static-positioning physical-properties align-content-block fit-content",
      "css_layout", "boxes_take_display_position_insets_and_fit_content_as_css_does",
      "Computed boxes: inline and inline-block boxes share a line and a block takes its own, full width; display: none makes no box and keeps no room; position: static ignores its insets; opposite insets size an auto box and an auto inset follows the other; block align-content: end; width: fit-content is max-content where it fits and the available width where it does not. fit-content on height and the min/max properties is not asserted.")
proof("z-index contain contain-paint overflow-clip", "css_contain", "z_index_orders_paint_and_clip_and_contain_cut_it",
      "Drawn order of three overlapping positioned boxes is z-index -1, auto, 2 against their source order; contain: paint, overflow: clip, contain: strict and contain: content push a clip at their box and contain: none does not; strict sizes as if empty and content keeps its height; overflow: clip makes no scroll container. z-index: 0 and auto order alike (og-ui orders siblings, it keeps no separate stacking context for 0); overflow-clip-margin is a gap of its own.")
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
not_applicable("color-adjust print-color-adjust",
               "Print colour economy: a game draws to its screen, never to paper or a printer's economy mode.")
not_applicable("paint",
               "The CSS Painting API runs a browser JavaScript paint worklet (CSS.paintWorklet, PaintRenderingContext2D); a document has no worklet runtime, and engine art comes from materials, vectors and images.")
not_applicable("svg-discouraged",
               "SVG 1.1 features SVG 2 removed (xlink: attributes, SVG fonts, SVGPathSeg and other DOM methods, zoomAndPan, baseProfile); documents are SVG 2 markup with no script DOM.")

MISSING = {
    "anchor-positioning-animations": "A transition between anchor() values jumps: interpolation treats two anchor() calls as discrete, and calc() takes anchor() only as anchor(…) + px.",
    "motion-path": "No offset-path, offset-distance, offset-rotate, offset-anchor or offset-position: the properties are refused.",
    "popover": ":popover-open restyles an open popover (control_props::titles_and_popovers_show_against_their_controls covers the popover attribute), but a popover draws no ::backdrop and its open/close ignores @starting-style and display/overlay allow-discrete transitions.",
    "overlay": "The overlay value holds its shown side in a discrete transition, but nothing keeps a closing popover or dialog in the top layer while its transition runs.",
    "active-view-transition": ":active-view-transition-type() qualifies only ::view-transition-* rules; :active-view-transition does not match elements during a transition (a selectors-side gap).",
    "view-transitions-element-scoped": "No view-transition-scope or element-scoped transitions: a transition captures a whole layer.",
    # layout
    "position": "Gap: an absolutely positioned box is placed against its parent, not its nearest positioned ancestor (CSS Position 3 §2.1), and a fixed box against the viewport even under a transform or containment. Direct-parent cases are positioning_places_as_css_positioning_does.",
    "contain-layout": "Gap: contain: layout makes no containing block for its absolute and fixed descendants (CSS Containment 2 §3.2); they are placed against their parent or the viewport.",
    "line-clamp": "Gap: line-clamp cuts a text's lines only when text-overflow: ellipsis is also set, and does not apply to a <div> block container (CSS Overflow 4 §4.1).",
    "scrollbar-gutter": "Gap: a box with overflow-y: auto or scroll and overflow-x left visible (which computes to auto) reserves no gutter, because a box that may scroll both ways draws its bars over its content.",
    "default": "documents::boolean_attribute_selectors_follow_svelte_presence proves retained :default follows false/present checked attributes. The immediate builder does not expose :default state; full support is missing.",
    "interpolate-size": "No intrinsic-size interpolation implementation or output proof.",
    "calc-size": "No calc-size evaluation or animated output proof.",
    "table": "Table spans/captions have computed output tests, but border-spacing and border-collapse have no implementation/output proof.",
    "backdrop-filter": "Ordinary staged backdrop chains have fixtures; native glass rejects chains requiring intermediate clamps, drop-shadow and SVG url filters.",
    "background-clip-text": "background-clip: text clips to the element's own glyphs only; CSS also clips to its descendants' text, which needs the subtree's text grouped where og-ui draws it (ui/commands.rs).",
    "filter-function": "No filter(<image>, <filter-function>) image: background and mask layers take url(), image-set(), gradients and light-dark() only.",
    "cross-fade": "No cross-fade() image: a box layer draws one image, and a premultiplied sum of several needs layers that hold more than one.",
    "element": "No element() image; feImage's element images exist for SVG filters but no background or border layer reads a live element.",
    "image-orientation": "The cook decodes images without their EXIF orientation and there is no image-orientation property; from-image and none are both unimplemented.",
    "border-shape": "No border-shape: borders, border images, outlines and shadows follow border-radius and corner-shape only.",
    "box-decoration-break": "Inline boxes and column fragments draw no per-fragment decorations, so neither slice nor clone is implemented.",
    "shadow-parts": "Host/part/state selector registry is present; all-property part applicability awaits the parts-as-nodes stream and its output tests.",
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    archive = args.archive.read_bytes() if args.archive else urllib.request.urlopen(URL).read()
    if hashlib.sha256(archive).hexdigest() != SHA256:
        raise ValueError("WebDX archive hash changed")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
        data = json.load(package.extractfile("package/data.json"))["features"]
    features = {key: value for key, value in data.items()
                if any(name.startswith("css.") for name in value.get("compat_features", []))}
    if len(features) != 454 or (PROVEN.keys() | MISSING.keys() | NOT_APPLICABLE.keys() | PART_PROVEN.keys()) - features.keys():
        raise ValueError("feature inventory drift")
    lines = [START,
             "## 8. Standard CSS output-proof checklist",
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
             "For offline use, pass `--archive <web-features-3.45.0.tgz>`; its SHA-256 is pinned in the script. "
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
        part_state = "Pending parts-as-nodes"
        for module, test, scope in PART_PROVEN.get(key, []):
            source = proof_source(module, test, "part proof")
            part_state = "Supported (scope →)"
            reason += f" Part proof: [`{module}::{test}`](../{source}). {scope}"
        lines.append(f"| `{key}` | {label} | {state} | {part_state} | {reason} |")
    lines += [END]
    path = ROOT / "docs/ui.md"
    old = path.read_text()
    before, sep, rest = old.partition(START)
    if not sep:
        raise ValueError("missing checklist markers")
    _, sep, after = rest.partition(END)
    if not sep:
        raise ValueError("missing checklist end marker")
    new = before + "\n".join(lines) + after
    if args.check:
        if old != new:
            raise SystemExit("CSS checklist drift: run scripts/css-coverage.py")
    else:
        path.write_text(new)
    print(f"454 CSS entries: {len(PROVEN)} scoped output proofs; {len(NOT_APPLICABLE)} not applicable; "
          f"{454 - len(PROVEN) - len(NOT_APPLICABLE)} missing/gap rows")


if __name__ == "__main__":
    main()
