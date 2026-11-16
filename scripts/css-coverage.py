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
import re
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


# Feature: one-line reason no game needs it, though it could apply to a
# game engine's UI. Closed without an implementation; neither a gap nor support.
NOT_NEEDED = {}


def not_needed(features, reason):
    for feature in features.split():
        if feature in NOT_NEEDED:
            raise ValueError(f"duplicate not needed: {feature}")
        NOT_NEEDED[feature] = reason


not_needed("webvtt",
           "WebVTT text tracks and ::cue style captions a <video> draws. A game's subtitles are `$captions` and the "
           "built-in subtitles screen, a document that documents and style packs already style.")
not_needed("time-relative-selectors",
           ":past and :future match WebVTT cues during playback; subtitles are `$captions` and the subtitles screen, "
           "so there are no cues to match.")
not_needed("mathml",
           "Mathematical markup (display: math, math-style, math-depth). Game UI does not typeset formulas, and no "
           "component, style pack or guide needs it.")
not_needed("speak speak-as",
           "Aural rendering of text by a speech synthesiser. The screen-reader bridge speaks the accessibility tree "
           "through the platform, and no game steers pronunciation from CSS.")
not_needed("target-text",
           "Text fragments (#:~:text=) and ::target-text mark a passage a URL names, for browsers sharing links into "
           "pages. A game's links name ids, and a passage it wants marked is a custom highlight (::highlight()).")
not_needed("text-decoration-spelling-grammar",
           "spelling-error and grammar-error draw the lines a UA's spelling and grammar checker puts under text. "
           "The engine has no spell checker, so nothing would ever produce them; a word puzzle marks a wrong word "
           "with an underline style of its own.")
not_needed("forced-colors",
           "Forced colours mode replaces an author's colours with the OS's high-contrast palette. A game draws its own "
           "surface and offers contrast through its style and prefers-contrast; (forced-colors) answers none, as "
           "the engine is never in that mode, and forced-color-adjust is refused.")
not_needed("url-cross-origin url-referrer-policy",
           "cross-origin() and referrer-policy() set a remote url()'s CORS mode and Referer. The engine sends no "
           "cookies, credentials or referrer (natively it sends no Origin or Referer; the web Worker fetches with "
           "credentials omit and no-referrer), so they protect nothing; url() refuses them.")
not_needed("link-parameters",
           "Passes param() values through an image URL into an external SVG's var(). A game's SVG is inline markup "
           "or a cooked image, styled by its document, and no component or style pack parameterises one by URL.")


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
      "Real clicks transfer focus between nested retained branches; computed widths prove shared ancestors stay focused and old ancestors reset. Parts are not covered.")
proof("object-view-box", "control_props", "an_image_shows_its_object_view_box",
      "The document emits asymmetric inset/calc crop UVs; the removed -og-uv property is rejected.")
proof("list-style", "html_text", "list_markers_follow_css_and_renumber_on_reorder",
      "Asserts marker text/shapes, position, inheritance and keyed reorder. Explicit initial resetting is tracked separately.")
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
      "GPU pixels at 1280 (css_effects::effects_show): a mix-blend-mode: multiply dot darkens the gradient under it to within 16 of their product; in an isolation: isolate group a dot alone keeps its own colour and the overlap is the two dots' product; rect(8px auto 75% 25%), xywh(10% 10% 60% 70%), shape(), content-box and url(#id) clips (user space and objectBoundingBox) keep the box inside their shape and show the card where they cut, at points the unclipped box covers. "
      "The 1280 and 390 frames are also compared with their references for review.")
proof("filter svg-filters", "css_filters", "css_filters_match_their_reference_frames",
      "GPU pixels at 1280 (css_filters::filters_show): grayscale(1) between two url()s leaves a warm gradient grey; an SVG filter's dilated, blurred gold flood lights the card 4px outside its box; a feColorMatrix channel rotation turns red green; feMorphology erode cuts 6px off an edge; a 60% feFlood composited atop turns red blue. The 1280 and 390 frames, including the other primitives, are compared with their references for review; native glass backdrop chains are not proved.")
proof("font-size rem font-shorthand", "control_props", "root_relative_text_sizes_follow_the_root_not_the_parent",
      "The document emits 1.75rem at 24.5/36.75 px under 100/150 percent text scale, distinct from 1.75em under a 10px parent; shorthand is checked too.")
proof("text-shadow", "control_props", "scene_text_shadows_follow_the_font_size",
      "The document emits the same shadows as independently resolved px values for em/lh and inherited font size; none removes them.")
proof("absolute-positioning relative-positioning fixed-positioning sticky-positioning anchor-positioning",
      "css_layout", "positioning_places_as_css_positioning_does",
      "Asserts relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered.")
proof("writing-mode", "css_layout", "writing_modes_lay_out_as_css_writing_modes_does",
      "Asserts vertical-rl/lr, sideways-lr, flex axes, logical padding/margins, orthogonal flow and vertical float geometry.")
proof("layout-direction-override", "css_layout", "direction_lays_out_as_css_writing_modes_does",
      "Computed flex positions, logical/physical margins and absolute placement distinguish dir and CSS direction on LTR/RTL surfaces.")
proof("vertical-align", "css_layout", "vertical_align_places_inline_boxes_as_css_does",
      "Asserts sup/sub text baselines, inline-block baseline/middle/top, px/percentage offsets and line-box growth.")
proof("float-clear", "css_layout", "floats_lay_out_as_css_floats_do",
      "Computed opposing/stacked floats, paragraph exclusion, flow-root avoidance, clear:both and enclosing float height. Rectangular exclusion only.")
proof("viewport-units", "css_layout", "vi_and_vb_follow_the_roots_writing_mode",
      "Computed vi/vb dimensions follow the root writing mode on an 800x900 viewport, independently of child writing modes. Dynamic/small/large viewport units are not covered.")
proof("border-radius corner-shape", "box_decor", "corners_take_css_radii_and_shapes",
      "The document emits fill/stroke ellipse radii with overlap scaling, percentages, slash syntax and bevel corners.")
proof("box-shadow", "box_decor", "decorations_paint_in_css_order_over_any_draw",
      "Asserts outer shadow offset/blur, inset spread and paint order relative to the fill.")
proof("pointer-events", "css_interact", "the_pointer_passes_through_none_and_hidden",
      "Actual cursor routing through pointer-events:none and visibility:hidden, descendant overrides, URL priority/hotspot and accessibility presence. OS cursor loading is not covered.")
proof("text-align", "css_text", "text_align_places_a_paragraphs_lines",
      "Drawn line positions of one-word paragraphs in 400px boxes: left and start at the left edge in left-to-right text, right and end at the right, center centred; in right-to-left text start (the initial value) at the right, end and left at the left; a paragraph in a centred <div> is centred (inherited), and match-parent takes its right-to-left parent's end as the left. "
      "control_props::a_button_label_takes_text_align proves a button label's default centre, explicit start and menu item alignment. Justification is text-align-last's and text-justify's rows.")
proof("font-family", "runs", "ordered_named_families_preserve_primary_glyphs_and_inherit",
      "Asserts exact glyph/font handles for inherited lists, rich spans, reversed named/generic order, names after generics and quoted generic names. Generics map to engine font roles, not installed system fonts.")
proof("gradients", "css_paint", "repeating_gradients_repeat_the_stop_interval",
      "GPU pixels prove repeating radial intervals with positive/negative starts, all eight conic half-quadrants and collapsed linear stops. Subpixel periods and unbounded stop counts are not covered.")
proof("gradient-interpolation", "css_paint", "gradients_mix_in_the_colour_space_they_name",
      "GPU pixels within deltaEOK 0.02 of CSS Color 4's formulas prove a full `in hsl longer hue` turn, `in hsl` red to blue the short way through magenta, and `in srgb` against the default Oklab. Other spaces and conic/radial forms are seen in review frames, not asserted.")
proof("margin", "html_text", "block_margins_follow_the_ua_stylesheet_and_collapse",
      "Asserts positive/negative sibling and parent-child margin collapse, padding/fixed-height boundaries, noncollapsing flex margins, UA em margins and keyed insertion.")
proof("text-wrap-balance text-wrap-pretty text-overflow", "html_text", "text_wrap_styles_and_overflow_strings_lay_out_as_css_does",
      "Inspects emitted lines: balance reduces width spread at the same line count; pretty avoids the final single-word line; custom overflow text replaces the ellipsis on one clipped line.")
proof("stretch", "css_layout", "stretch_in_a_flex_line_is_its_stretch_fit_size",
      "Computed sizes of `stretch` across a flex line (filling it, the row laid out at the line's width), along one (the container's inner size, not floored by taller content nor shared with a sibling); a block's stretch height is asserted in css_layout::a_stretch_height_in_a_block_fills_its_containing_block. Grid areas and absolute boxes are not asserted.")
proof("text-indent text-indent-each-line text-indent-hanging line-break", "css_layout", "text_indent_and_line_break_lay_out_as_css_text_does",
      "Drawn lines of a shaped face show the first-line, hanging and each-line indents, a right-aligned indented line ending at its box, a max-content box growing by the indent, and line-break: anywhere filling a line mid-word. Strict/loose CJK punctuation rules are Parley's and not asserted; atlas-only and pixel faces are not indented.")
proof("container-queries", "documents", "container_queries_restyle_lists_grids_and_rive_frames_before_paint",
      "Viewport changes immediately change list/grid padding and Rive visibility under a named inline-size query. Pushed-screen path; style/scroll-state queries and container units are not covered.")
proof("scroll-driven-animations", "documents", "view_timelines_follow_the_subject_through_its_scroller",
      "Asserts width and translation at rest and after wheel input for an anonymous view() cover timeline. Named timeline scopes and all range keywords are not covered.")
proof("animations-css cubic-bezier-easing", "documents", "scoped_css_motion_uses_shared_curves_emits_sound_and_restores",
      "A cubic-bezier(0,1,0,1) keyframe animation reaches 7/8 displacement at 1/8 duration; twelve subsequent positions replay exactly after checkpoint restoration. Pushed-screen path, not every animation mode.")
proof("color", "css_text", "color_inks_text_and_is_currentcolor",
      "Drawn text ink of paragraphs whose color is #f00, rgb(0 128 255), hsl(120 100% 25%), rebeccapurple and rgb(0 0 255 / 50%) (alpha 0.5), a child paragraph and its span inheriting a parent's #00ff00, and a box's background-color: currentcolor filled in its own color. "
      "dressing::host_font_and_color_reach_button_ink_and_divider_output proves button text and a divider. Other colour spaces and functions are their own rows'.")
proof("image-set", "css_paint", "image_sets_render_density_correct_backgrounds_masks_and_borders",
      "GPU pixels at DPR 1/2/1 prove density-correct background/mask tiles, MIME filtering before duplicate removal (including quoted parameters and malformed-type rejection), zero-resolution transparency, percentage border slices with auto widths and gradient selection. Resolution math covers calc/min/max/clamp with mixed x/dppx/dpi/dpcm units; retained output also tests fractional DPR.")
proof("background-image", "css_paint", "image_urls_resolve_files_not_atlas_names_or_handles",
      "Retained drawing selects the registered relative file and numeric filename, rejects a missing path despite an atlas basename collision, and reports the missing asset. css_paint_matches_its_reference_frames covers background tiling and gradients on the GPU. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks proves cooked SVG pixels, imported masks and inline cursor URL provenance.")
proof("cursor", "css_interact", "plain_boxes_choose_cursors_in_clipped_transformed_paint_order",
      "Asserts cursor inheritance over noninteractive boxes, passive overlays above buttons, pointer-events:none, hidden and overflow-clipped children, 90-degree transforms and image hotspots. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks checks cooked SVG cursor pixels on a plain div. css_interact::cursors_apply_only_inside_clip_paths proves cursors only inside circle(), polygon() over content-box, evenodd path(), shape(), objectBoundingBox url(#id) and an ancestor's inset() clip. Clip-path press hit testing and every cursor keyword are not covered.")
proof("field-sizing", "control_props", "fields_size_to_displayed_content_and_respect_constraints",
      "Proves text-input value/placeholder sizing, caret-width empty inputs, textarea wrapping and newline heights, ignored rows, explicit dimensions, min/max width, noninheritance and removal of content sizing. Retained editing and checkpoint restore preserve focus/caret and update height. selects_files_and_vertical_fields_size_to_content proves a select fitting its shown option (fixed fills, explicit width wins), file inputs fitting chosen names while fixed keeps the default label width, and vertical-rl inputs/textareas sizing height to content with a line-wide width and caret-high empty height, wrapping into columns under a fixed height. Listbox selects and fixed sizing in vertical modes are not covered.")
proof("shape-outside shapes", "css_layout", "shape_outside_changes_line_exclusion_not_float_placement",
      "Proves triangular line exclusion, unchanged float stacking/BFC avoidance/clear, and dynamic none/restoration. shape_outside_uses_the_whole_line_band_and_reference_box proves circles, ellipses, insets, margin/border/content boxes, margin-box clipping, degenerate insets and dropping a wide line along a slope. shape_margin_expands_by_distance_and_clips_to_the_float_box proves distance-based shape-margin, containing-block percentages, clipping and dynamic zero restoration. shape_images_wrap_lines_around_their_alpha proves url() alpha contours per pixel row, shape-image-threshold as number and percentage, an empty shape, shape-margin around them, content-box placement and the box kept without alpha; runtime ui_configure::shape_outside_images_take_their_contour_from_cooked_alpha reads a cooked SVG's alpha natively. Gradient images, Basis textures on the web, other syntax, elliptical box corners and writing modes remain unverified.")

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
proof("font-style font-synthesis font-synthesis-weight font-synthesis-style", "css_text", "font_synthesis_and_oblique_angles_draw_as_css_fonts_says",
      "Drawn commands of styled text in a one-face font: a 700 italic thickened and slanted, font-synthesis-weight: none and font-synthesis: style slanted only; oblique 30deg and bare oblique sheared by tan 30deg and tan 14deg; italic slanted; font-synthesis-style: none upright. Synthesis happens only where the text is a span (a style, decoration or stroke the plain path cannot draw): plain text at weight 700 is not thickened yet. A family's real faces are font-face's proof.")
proof("counter-reset-reversed", "css_text", "reversed_counters_start_where_their_increments_end",
      "Drawn ::before counters: reversed(c) with no value starts at CSS Lists 3 §4.4.2's sum (3, 2, 1 for three -1 items; 12, 11, 10, 9 around a counter-set of 10), an inner reset's scope taking its following siblings, and reversed(c) 5. The start is found as the run passes the scope, so readers take it from the following tick (the first frame counts from 0). The implicit list-item counter's -1 on reversed lists is HTML's ol reversed, not this counter.")
proof("symbols-function counter-style marker", "css_text", "list_style_types_take_counter_styles_symbols_and_strings",
      "Drawn list markers and ::before text: symbols() cyclic, alphabetic, numeric and the default symbolic, in list-style-type, the list-style shorthand, counter() and counters() (a fixed style past its symbols falling back to decimal); a string list-style-type; an @counter-style cyclic name in list-style-type and in ::marker's counter(list-item, …) in a <ul>, whose content and color replace the bullet's. The drawn marker leaves out its suffix's trailing space (the marker gap stands for it). @counter-style's other descriptors (additive-symbols, negative, pad, prefix, range, fallback, extends) are unit-tested in generated::tests, not drawn; speak-as and image symbols are not read; the CSSCounterStyleRule API is a script API with no counterpart; ::marker takes content and color only, not animations.")
proof("first-letter first-line", "css_text", "first_letter_and_first_line_style_their_runs",
      "Drawn runs' ink and size: ::first-letter takes the first letter, number or symbol cluster with the Ps/Pe/Pi/Pf/Po punctuation around it (“¿Q, (1), e with a combining acute, and A. from a ::before), none for a leading dash, and its font-size draws that run only; ::first-line inks the first line as the unstyled twin breaks it and not the second, from the tick after layout; ::before's and the element's own inks draw beside them. Only a text element's own text: a block's ::first-letter or ::first-line does not reach into its first child's text, and a ::first-line font change does not relayout its line.")
proof("display-list-item", "css_text", "display_list_item_makes_any_box_a_list_item",
      "Probe markers: <div>s with display: list-item draw decimal markers 1, 2, 7, 8 from the run's list-item counter (an explicit counter-increment: list-item 5 replacing the implicit 1), disclosure-closed and disclosure-open draw the theme's closed and open disclosure chevrons (Counter Styles 3 §6.4's ▸ and ▾ shapes), and an <li> set to display: block draws none (li's list-item display is the UA sheet's). css_text::text_list_items_and_summaries_draw_their_markers: a text-only <p> and <span> draw outside markers 1. and 2. before their first line and an inside one draws \"3.\\u00a0\" as its text's first run in the ::marker colour; <summary> (UA display: list-item) draws disclosure-closed, disclosure-open under an open <details>, flips on a click, and draws none under list-style: none. <legend> is not built as a list item.")
proof("text-decoration-skip-ink text-decoration-skip-ink-all", "css_text", "underlines_skip_the_ink_of_glyphs_they_cross",
      "Drawn underline pieces at 40px against Noto Sans's and Noto Sans SC's outlines read with fontTools: auto cuts the line where g, y and p put ink in its band, a thickness to either side (merging g's and y's), none draws it whole, a line-through never breaks, auto crosses CJK (Han and kana and Hangul, and Common characters in their run) and all cuts it. Variable instances are outlined at their location. Horizontal lines only: vertical text's lines and the ::selection decoration is not cut; a glyph from a face without its file is crossed.")
proof("text-decoration-skip-spaces", "css_text", "decorations_skip_the_spaces_skip_spaces_names",
      "Drawn underline and line-through pieces at 40px against Noto Sans's HarfBuzz advances on a white-space: pre line: the initial start end leaves its leading and trailing spaces undecorated, all also the inner ones on both lines and the 4px letter spacing before each skipped run, none decorates every space. Spacers are Zs but U+202F (U+00A0 included), word separators CSS Text 3's. Line start and end are the visual ends of a horizontal rich line; vertical text and the ::selection decoration do not skip.")
proof("text-decoration-line-blink", "css_text", "blink_draws_no_line",
      "text-decoration-line: blink parses without a warning and draws its text with no line, and underline blink draws the underline alone: the text does not blink, as Text Decoration 4 §2.1 allows a conforming UA.")
proof("text-decoration-selection", "user_select", "selection_ink_lines_and_shadows_cover_only_the_selected_run",
      "Drawn quads: p::selection's text-decoration: underline #00ff00 draws one green line under the selected word only, its top 0.12 em below the baseline and 0.07 em thick (Text Decoration 4's auto metrics) within a pixel. Only underline in that test; ::selection's line skips neither ink nor spaces.")
proof("hyphens hyphenate-character hyphenate-limit-chars", "css_text", "hyphens_auto_breaks_words_at_the_languages_hyphenation_points",
      "Drawn lines of a 90px paragraph at 20px against TeX's US English patterns (pyphen agrees) and HarfBuzz widths: hyphens: auto breaks hyphenation into hyphen- | ation once the locale is en-US and not before, manual (initial) keeps it whole; an inherited hyphenate-character: \"=\" shows = at the break; hyphenate-limit-chars auto 2 6 leaves only hy- | phenation and 12 no break. Dictionaries are hypher's hyph-utf8 tries shipped in the latin and cyrillic-greek font packs, cooked in only for the game's locales when it declares hyphens: auto (og-cook's strings test checks the choice and the warning for a locale with none), and picked by the game's locale (og.i18n), not an element's lang attribute; the test installs en from the latin pack and sees no break before; a word already holding a soft hyphen keeps only its own. A span's own hyphenate-character inside a paragraph is not read.")
proof("font-face", "runs", "a_font_face_family_takes_its_faces_by_weight_and_range",
      "Glyph faces per character for @font-face weight ranges matched in CSS Fonts 4 §5.2's order, unicode-range and a declared family in a document's font-family list. src resolution is runtime ui_configure's; font-style, font-stretch, format() and tech() descriptors are not covered.")
proof("font-weight", "css_fonts", "font_weight_picks_the_face_and_synthesises_a_missing_bold",
      "Drawn font handle and thickening per element: bold, 650 and 550 take a family's 700 face, 300 its 400; bolder over 400 and lighter over 700 step by Fonts 4 §2.2's table; a family with only a 400 face draws 600 and 700 thickened twice one device pixel apart, at the thickening a rich span at 600 draws, in place of the style's textWeight, and 400 and 500 plain; font-synthesis-weight: none draws it plain. A variable face's wght axis is font-variation-settings' proof.")
proof("font-stretch font-width", "css_fonts", "font_stretch_takes_the_family_face_of_the_nearest_width_in_css_order",
      "Drawn font handle of a family with 75 %, 100 % and 125 % faces: condensed takes 75 %, 95 % the narrower 75 % before the nearer 100 % and 105 % the wider 125 % (Fonts 4 §5.2), ultra-condensed with nothing narrower the nearest wider, font-width: expanded as font-stretch, an inherited width; the text measures in the face it draws; the @font-face font-stretch and font-width descriptors parse to one width or a range. Width matching applies to @font-face families (a style's role faces carry no width); a variable face's wdth axis is font-variation-settings' proof.")
proof("display-ruby ruby-align ruby-overhang ruby-position", "css_cjk", "ruby_lays_out_as_css_ruby_does",
      "Drawn glyphs of HTML <ruby>/<rt> in Noto Sans SC: the annotation at half size on the base's over edge, or under edge with ruby-position: under; space-around's shares between CJK characters and start; a longer annotation hanging half its excess under ruby-overhang: auto and none under none; <rp> not drawn; the line growing for it; beside the base on the right in vertical-rl.; <rb> columns paired one by one with <rt>s and the widened column's annotation centred; an <rtc> under the base as alternate's second level, spanning it, and left of a vertical-rl column; inter-character upright in its own half-em column right of the base; nested ruby's outer annotation stacked past the inner; display: ruby, ruby-text and none on ::part spans, and display: ruby and ruby-text in a plain <span>'s style (css_cjk::inline_elements_are_matched_and_styled_as_css_does, which also proves class, id, attribute, :last-of-type, child and forced :hover selectors on a text's inline elements styling their words; css_cjk::inline_elements_fragment_on_their_lines_and_links_hover_as_their_a proves a wrapped link's two fragments, its hits on them alone and pointer :hover on its <a>; css_cjk::inline_edges_take_room_at_their_start_and_end proves a span's margin, border and padding moving the words after its start and end by their sum, only on the lines holding them, its fragment the border box outside the content area with its line unchanged, box-decoration-break: clone holding both edges on each of four lines, and a vertical-rl element's room down its column). ruby-merge, ruby-overhang: spaces and Bopomofo tone placement are not covered.")
proof("text-orientation glyph-orientation-vertical writing-mode-svg-values text-combine-upright", "css_cjk", "vertical_text_orients_and_combines_as_css_writing_modes_does",
      "Drawn glyphs in Noto Sans SC: upright Latin an em apart and sideways CJK turned a quarter clockwise, glyph-orientation-vertical 0deg and 90deg as upright and sideways, tb as vertical-rl and lr-tb as horizontal-tb, text-combine-upright: all squeezing 123 into one em and digits 2 combining 10 in one em between 年 and 月 while 100 lies sideways, all on a ::part span setting its AB upright in one em between 年 and 月.")
proof("text-emphasis", "css_cjk", "emphasis_marks_sit_where_css_text_decoration_puts_them",
      "Drawn marks: a dot on each CJK character but 、 and a space, centred at half size 5 px over the content area in its own colour; filled under the descent in the text's colour; open stroked; a string's character at half size; a sesame right of a vertical-rl column and left with over left; the line growing for them; under in a lang=zh element and over in a lang=ja one inside it, right of a Chinese column (the §3.4 UA rules). a ruby's base marked outside its annotation (across, and right of it down a column), its <rt> unmarked.")
proof("font-variation-settings font-optical-sizing", "css_fonts", "a_variable_face_is_drawn_at_the_axes_css_sets",
      "Laid-out advances of a variable test font's glyphs, one per axis: font-weight 650 sets wght, font-stretch 75% wdth (each clamped to the face's declared range: a face declared 400 draws 700 at wght 400, synthesised bold, and an @font-face without the descriptor parses as auto), font-size 43px and 24px opsz under font-optical-sizing: auto and its default under none, font-variation-settings GRAD 50 and an inherited GRAD 100, normal resetting it, and wght 100 overriding font-weight 900; each instanced text is drawn with its instance's own font (blank on the frame before the host makes it), asks the host for its glyphs, and a 700 its face's wght range reaches is not synthesised. COLR glyphs of an instance paint at the default location; a mixed text's fallback fonts, widget part text and MVAR metrics are not instanced.")
proof("font-family-system font-family-ui font-family-math", "css_fonts", "generic_families_take_the_style_faces_that_suit_them",
      "Drawn font handle per element: system-ui and ui-sans-serif the style's body face, ui-monospace and ui-rounded its own mono and rounded faces; ui-serif and math, which no engine face suits, match nothing and the list goes on (math, \"Fancy\" takes Fancy; ui-serif, ui-monospace the mono face); in a style with only a body face ui-monospace and ui-rounded go on to the next family while monospace, which always matches, takes body's. There are no OS fonts: the style's faces are the system's. No math face ships, so math layout (stretchy operators, MATH table) is not covered.")
proof("colrv0 colrv1", "css_fonts", "colr_glyphs_paint_their_layers_and_gradients_in_colour",
      "Pixels of a test colour font's glyphs from the COLR painter the cook and run-time colour fonts share: a COLRv0 glyph's entry-0 square under its entry-1 right half, a COLRv1 linear gradient red at its left edge, green at its right and half each between, a 0xFFFF layer painted black with a white twin for the text's colour, and a glyph without a paint not painted. Variable COLR (at an instance's location) and the GPU's draw of the colour pages are not covered here.")
proof("font-palette", "css_fonts", "font_palette_draws_a_colour_face_in_the_palette_css_picks",
      "The CPAL colours each text's instance is drawn in: normal the cooked face, light and dark the palettes the font marks, an @font-palette-values with base-palette 1 and override-colors (one past the palette ignored), the last rule of a name only where its font-family lists the text's family, a base-palette past the font's palettes its first, inherited, and in a rich text; A of the light instance painted blue under yellow. The host painting the instance's glyphs in its palette is og-runtime's a_palette_instance_paints_its_colour_glyphs_in_its_palette. Only run-time colour fonts (runtime true) change palette; @font-palette-values in style packs are not read.")
proof("font-palette-animation", "css_fonts", "font_palettes_mix_and_animate_entry_by_entry",
      "The CPAL colours each text's instance is drawn in: palette-mix(in srgb, light 25%, dark) and palette-mix(in srgb, normal, dark) entry by entry (normal the font's first palette), a font-palette animation from light to dark held at 50% the OKLab mix of each entry (116 163 255 and 99 99 0, not sRGB's 128s), and a running one drawn every frame with an instance whose font the host already made while its own waits. Transitions share the animation's interpolation; the first frame of a palette with no instance made yet still draws blank.")
proof("font-variant-alternates", "css_fonts", "font_variant_alternates_take_the_feature_values_named",
      "Drawn Noto Sans glyph ids: stylistic() sets salt (I 904), styleset() one ssNN per value (ss03 turns ƒ into 1231, ss04 I into 904), each name from the last @font-feature-values that lists the text's family (case insensitive), a rule for another family and an undefined name ignored, inherited, through the font-variant shorthand, and font-feature-settings applied after it. character-variant(), swash(), ornaments() and annotation() map to cvNN, swsh and cswh, ornm and nalt but no shipped font has them; historical-forms is hist. @font-feature-values in style packs is not read.")
proof("font-language-override", "css_fonts", "font_language_override_and_lang_pick_the_fonts_local_forms",
      "Drawn Noto Sans glyph ids: lang=\"ro\" draws the locl form of Ş (285 to 326), inherited from a ro-RO ancestor, an empty lang clears it, font-language-override: \"ROM\" draws it without a lang, \"MAH\" over lang=ro draws Marshallese Ļ (898) and not Romanian Ş, and a paragraph Parley lays out (text-indent) is shaped in its lang too. A tag no language shapes in first (no BCP 47 equivalent) applies outside Parley only; the shaping-only fallback of mixed text is not covered.")
proof("font-variant-emoji", "css_fonts", "font_variant_emoji_picks_a_colour_or_text_face",
      "Drawn face per run, a test text font with outline ⌚ and © beside Noto's COLRv1 emoji: normal and unicode draw © in the text face and ⌚ (Emoji_Presentation) in the colour face, text draws both as text, emoji both in colour; U+FE0F after © draws it in colour and U+FE0E after ⌚ as text, the selector winning over font-variant-emoji: text; inherited, and on a rich span. A role whose cooked atlas has © switches to its mixed font for an emoji-presented one (not covered by the test, whose faces render at run time); an emoji face cooked without its file (no COLR to read) counts as a text face.")
proof("font-size-adjust", "css_fonts", "font_size_adjust_sizes_each_face_by_its_metric",
      "Drawn size and start of each face's run, EB Garamond (x-height 0.407, from its OS/2 table) with a Noto Sans SC fallback (0.543) at 20 px: 0.5 draws them at 24.57 and 18.42 px with the CJK run starting at Garamond's x advance at its size, from-font draws Garamond at 20 and the fallback at 14.99, ch-width 0.6 sizes Garamond by its \"0\" (0.497) and SC, which has none, by half an em; inherited, none turns it off, and a rich text's runs are sized and placed alike. Line heights keep the text's size (a face's extents scale); vertical text and the ic metrics are not covered.")
proof("font-metric-overrides", "css_fonts", "font_face_metric_overrides_set_the_faces_size_and_line_metrics",
      "Element heights, drawn sizes and baselines of Noto Sans faces under parsed @font-face rules at 20 px with line-height: normal: 1.2 without overrides (24 px); ascent 90 %, descent 30 % and line gap 20 % a 28 px line with the baseline 20 px down; a line gap of 10 % alone the font's hhea ascent and descent (1.069, 0.293) plus it; size-adjust 150 % draws at 30 px two x advances (0.529) wide at it; size-adjust 200 % scales an ascent of 50 % and no descent or gap to a 20 px line with its baseline at the foot; font-size-adjust 0.5 on a size-adjusted face draws at 20 × 0.5 ÷ 0.536. The overrides apply to @font-face families (a style's role faces have none); line-height: normal reads the face for a label's own text and its inline elements, not a widget part's.")
proof("font-synthesis-small-caps", "css_fonts", "small_caps_and_super_and_subscripts_are_synthesised_where_the_face_lacks_them",
      "Drawn text and size per command at 20 px: EB Garamond, which has smcp, draws small-caps Ab whole at 20 px; Inter, which has none, draws A at 20 and b as B at 16 (80 %), inherited; all-small-caps draws AB at 16; font-synthesis-small-caps: none draws Ab at 20. The face's forms are found by shaping each letter with and without the feature (HarfRust). Petite caps and vertical text are not covered.")
proof("font-synthesis-position", "css_fonts", "small_caps_and_super_and_subscripts_are_synthesised_where_the_face_lacks_them",
      "Drawn text, size and baseline at 20 px: Noto Sans, which has sups forms of 2 and n, draws super 2n at 20 px on the paragraph's baseline; 2x (no form of x) is drawn whole at 16.4 px (82 %) with its baseline 20/3 px above the paragraph's; Inter, which has no subs, draws sub 2 at 16.4 px 4 px below; font-synthesis-position: none draws it at 20 px on the baseline.")
not_applicable("font-display", "A document's @font-face faces are cooked into its bundle and loaded when the document is defined, before its first frame, so there is no block, swap or failure period for font-display to choose.")

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
proof("all", "documents", "all_sets_every_property_but_direction",
      "all: inherit takes the parent's width, height and font size; all: initial gives width auto (0 with no content) and the root's font size over a class's 10px; all: unset then width: 2em reads the inherited 20px, and width: 2em then all: unset loses; custom properties and direction (rtl under all: initial) are kept. revert and revert-layer through all are not asserted.")
proof("function", "css_layout", "custom_functions_return_their_result",
      "Widths from @function calls: --double(21px) returns <length> 42px; CSS Mixins 1 §2.3's --add-a-b-c in px is 321px (the element's --a, the --b argument, the local --c over its argument); §4.1's result under @media is 16px at 800px and 20px at 1200px; a default argument gives 7px, a braced calc() argument 5px, a nested call 10px; a call in a custom property is inherited as its 30px result; a typed parameter given red and too many arguments leave width unset instead of an earlier 70px. Undashed names, a default not matching its type, a non-custom descriptor, a malformed returns and a repeated parameter fail at cook. @mixin/@apply and typed parameters computing (2em to px) are not built.")
proof("container-anchor-position-queries", "css_layout", "anchored_queries_read_the_applied_fallback",
      "Descendants of container-type: anchored boxes below a 40x20 anchor at (50, 150) in a 300x200 stage: one that overflows takes flip-block, so anchored(fallback: flip-block) and the boolean anchored(fallback) style its child; --above applied matches --above but not --above flip-inline; a box that fits above matches fallback: none and not (fallback); a top area fallback matches fallback: top (the other axis any) but not bottom any. Unknown features and values fail at cook. The applied entry is published from the frame that laid it out, so a change restyles on the next frame; logical area keywords read as horizontal-tb, as anchor positioning itself places them.")
proof("if", "css_layout", "if_takes_the_first_branch_whose_test_holds",
      "Widths from if(): media(width >= 600px) takes 100px at 800px and the else branch's 50px at 500px; style(--mode: narrow) on the element wins over the inherited wide; supports(display: grid) holds and supports(display: nonsense) does not; (not media(…)) and style(…) flips with the viewport; a nested if() resolves; a custom property's if() resolves where it is computed, so a child setting --k: b inherits the parent's 40px branch; style(--foo: bar) inside --foo is false (9px); no branch holding leaves width unset instead of an earlier 70px. Branches without a colon, unknown tests and an unparenthesised not … and … fail at cook. A branch spread from a variable (...var(--clause)) takes 60px and 6px with the viewport.")
proof("sibling-count", "documents", "sibling_index_and_count_follow_inserted_and_removed_rows",
      "Keyed rows' width calc(sibling-index() * 10px) and height calc(100px / sibling-count()) are 10px for the first of four (25px tall); a row inserted before the second makes the later rows 40px and 50px wide and all 20px tall on that frame, and removing the first moves them back. motion::sibling_index_staggers_transition_delays: calc(sibling-index() * 100ms) as a transition delay, in the shorthand and the longhand, staggers three items along (t - 100k) / 300. og-css's sibling_index_and_count_are_the_element_s_integers covers math with them and an element with no known place (1 of 1). Pseudo-elements and shadow-tree scoping are not asserted.")
proof("progress-function", "css_layout", "progress_and_random_resolve_on_their_element",
      "width: calc(progress(…) * L): 50px between 0px and 200px is 0.25, 300px clamps to 1, no-clamp gives 1.5, and 1em reads the element's font size; og-css's progress_and_random_evaluate covers the low clamp, times, start = end (0) and mixed types refused. css_layout::percent_progress_and_random_item_resolve: progress(50%, 0px, 200px) * 100px reads its basis at layout (50px in 200px, 100px clamped in 600px, 150px no-clamp), progress(100px, 0px, 100%) * 50% is 50px in 400px, and content-box padding adds once after it.")
proof("random-function", "css_layout", "progress_and_random_resolve_on_their_element",
      "Two elements' random(0px, 100px) widths differ and stay in range; a --k key with a step is shared and lands on a step; fixed 0.25 gives 125px; values hold through a re-resolve and in a second UI. og-css's random_keys_share_and_separate_values covers element-, property- and index-scoping, auto, fixed and uniformity over 900 elements; progress_and_random_evaluate the step epsilon, max below min and a mixed-unit linear form. random_keys_name_their_property_and_document: keys simplify as §9.4.1 says (auto to element-scoped ua-P-I, property-scoped to ua-P, index counted before a shorthand expands and kept in its longhands), and the document (the mount) is part of the cache name. random-item(): css_layout::percent_progress_and_random_item_resolve picks a shared --pick item and a braced item; random_item_picks_its_item covers floor(R × N), braces, nesting, empty items, a string with a comma and uniformity over 900 elements.")
proof("url-integrity", "remote_images", "remote_images_load_under_their_request_modifiers",
      "integrity(\"sha384-…\") with the bytes' hash draws the image, a wrong hash draws nothing and warns with the bytes' hash, and metadata with a wrong sha256 beside a right sha384 loads (only the strongest algorithm counts). "
      "og-net's http unit tests cover base64url and unknown algorithms.")
not_applicable("namespace",
               "A document's elements are in one namespace (a .svelte document is not XML), so @namespace prefixes "
               "select nothing apart; the rule is accepted and has no effect.")
proof("dynamic-range-limit", "css_paint", "dynamic_range_limit_sets_the_frames_peak",
      "A document root's dynamic-range-limit is the frame's (Ui::range_limit, the most limiting of several roots); on an 8x display standard ends it at 1x, constrained at 2x (one stop), no-limit and none declared at 8x, "
      "an even mix at 2^1.5 and the spec's nested dynamic-range-limit-mix() at 2^1.9 (normalized weights mixed in stops); a mix without a percentage or over 100 % is refused. og-render's display unit test checks the transfer's clamp and the tone curves end at that peak per keyword. "
      "css_paint::dynamic_range_limit_follows_its_roots proves the frame's limit follows a root's class change (no-limit, standard, no-limit) and that a second mounted document's constrained root holds the frame at 2x. "
      "A child element's own limit does not cap its content apart from the rest of the frame; the display pass that ends the frame at the peak runs only on an HDR surface, so no pixel is read here.")

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
      "p::selection's color, text-decoration and text-shadow draw on the selected word's glyphs only, clipped to the run, with the rest of the line in its own ink and one underline under the word. Its background is judged in the GPU frames (user_select::the_selection_matches_its_reference_frames). user_select::selection_styles_inherit_from_ancestors_property_by_property proves highlight inheritance (CSS Pseudo-Elements 4 §3.5): a paragraph with no ::selection rule takes its background fill and red ink from its parent's rule over :root's, and a paragraph outside that parent takes :root's background with no red ink.")
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
proof("appearance", "control_props", "appearance_none_drops_a_controls_own_frame",
      "Emitted fills and positions: under appearance: none a single-line field, a <textarea> and an open <dialog> fill nothing inside their box where their auto twins draw a frame, the field's text drops its frame's inset (12px in to 4) and the dialog's child sits at its corner where auto pads it. "
      "control_props::dialogs_close_as_closedby_says checks a modal dialog's. appearance applies to dialogs, fields and selects (base-select is customizable-select's row); the compat keywords (button, checkbox, menulist-button, textfield…) are not taken.")
not_applicable("autofill", "og's fields are engine-drawn; no browser or OS autofill fills them, so there is no autofilled state to style.")
proof("caret-shape", "css_paint", "caret_shape_draws_a_bar_block_or_underscore_over_the_next_character",
      "A focused 20px field with its caret before \"i\": bar and auto draw the same thin bar at the insertion point; block is the bar's full height and exactly the font's advance of \"i\" wide, starting at the bar and drawn before the text; underscore is that advance wide, at most 1px thick, its bottom on the bar's bottom, drawn after the text. At the end of the text both take 1ch (the advance of \"0\") at the bar's position. The shape inherits from a column. Text areas, right-to-left text and the caret shorthand are not covered; the block does not recolour the character it covers.")
proof("overflow-anchor", "css_interact", "scroll_anchoring_keeps_the_view_when_content_above_changes",
      "Offsets of a 100px scroller of 40px rows scrolled to 100: a 50px block inserted above takes it to exactly 150 with R3 drawn where it was, and its removal back to 100; the anchor (R2) growing below its top moves nothing; overflow-anchor: none on the container, or offset 0, leaves the offset and moves the rows; overflow-anchor: none on R2 makes R3 the anchor, so R2 growing 40px moves the offset 40. Block axis of horizontal writing only; snapping containers, floating layers' scrollers and the focused-field priority candidate are not anchored; the anchor is chosen again every frame rather than kept until a scroll; the property-change suppression triggers are not modelled.")
proof("details-content", "html_controls", "details_content_styles_the_disclosures_content_box",
      "A 300px details with ::details-content { padding: 30px 0 0 20px; background: red } and [open]::details-content green: closed, neither the content nor any box draws; open, the green fill spans the details' x and width below its top, red is not drawn, and the content sits exactly 20, 30 inside it. [open] follows bind:open both ways (closing again draws neither box); the player's toggle is not driven here; a closed box hides whole where CSS hides only its contents; transitions on the box are not asserted.")
proof("will-change", "css_contain", "will_change_makes_what_its_properties_would_and_nothing_else",
      "Drawn order and a fixed box's place, one document switching classes: will-change: transform, and left, filter (a list whose stacking entry comes second), keep a z-index: 1 child under its parent's later sibling and hold a position: fixed; left: 3px; top: 4px child at 3, 4 inside the element; opacity keeps the child in but leaves the fixed box at 3, 4 on the viewport; left, scroll-position, contents and auto do neither. will-change naming z-index or position (stacking only with other values) and any rendering hint are not covered.")
proof("scroll-axis-lock", "css_interact", "scroll_axis_lock_keeps_a_drag_to_the_axis_it_starts_along",
      "Scroll offsets of a 200px scroller over 1000x1000 content, read while the drag is held: a first move 40 across and 19 down scrolls 40, 0 and one 19 across and 40 down 0, 40 (within a 2:1 slope the other axis is dropped); 40 by 21 moves both; a drag locked to x by its first move and then turned 50px down never moves y; the next press decides afresh; scroll-axis-lock: none follows every drag and the turn. Mouse drags only (a touch drag takes the same path after touch-action); the wheel and two-finger pans are not locked.")
proof("scroll-initial-target", "css_interact", "a_scroller_starts_at_its_initial_target",
      "In a 100px scroller of 40px rows, the first scroll-initial-target: nearest row at 200px, with a 10px scroll-margin-top and the container's 5px scroll-padding-top, gives an offset of exactly 185 on the container's first frame, the row drawn 15px into the view; a later target loses; one near the end clamps to the 300px range; a target added to a container already shown leaves it at 0. Block-axis start alignment only (scroll-snap-align is not read); containers in floating layers are not covered.")
proof("interpolate-size", "motion", "interpolate_size_moves_to_and_from_intrinsic_keywords",
      "Under interpolate-size: allow-keywords declared on a parent (inherited), height 0px to auto over a 1000ms linear transition lays a box over 200px of content at 200t px every tick (within 0.5 px), and back down the same way; the same declarations without it (numeric-only) snap at once both ways. Transitions of width and height only (calc-size()'s own limit); @keyframes between a length and a keyword stay discrete.")
proof("vertical-form-controls", "html_controls", "vertical_writing_modes_stand_bar_controls_up",
      "A range, a progress bar and a meter under writing-mode: vertical-lr; height: 200px stand 200 tall (a 44 px track, an 8 px bar, a 12 px meter). Left to right a press a quarter down the track (its knob's half kept at each end) sets 25 of 0-100 and centres the knob there, its top end 0; direction: rtl sets 75 and 100; a quarter-full bar or meter fills exactly the top 50 px, or the bottom 50 with rtl, across its whole width. sideways-lr runs up, the reverse (by the same axis, not asserted); keys keep stepping as across; <select> does not stand up, and text controls take their writing mode through css_layout.")
proof("scroll-markers scroll-marker-targets", "css_interact", "scroll_markers_and_buttons_follow_the_scroll_position",
      "A 200px scroller of four 100px sections with scroll-marker-group: before: four ::scroll-marker boxes stand in a ::scroll-marker-group before it in tree order; :target-current, :target-before and :target-after follow the CSS Overflow 5 §3.1.8 example algorithm, whose redistribution puts the sections at 0, 100, 180 and 200 (current at offsets 0, 100, 190 and 200: the first, second, third and last); pressing a marker scrolls its section to the start (100 for the second, 200 for the third) and keeps the pressed one current while the offset stays there; a checkpoint restores the states on its first frame. Block axis of horizontal writing; the group is a row with no default look; tabs mode is parsed but builds the same links group (no roving focus); markers of nested scrollers are their own groups.")
proof("scroll-buttons", "css_interact", "scroll_markers_and_buttons_follow_the_scroll_position",
      "::scroll-button(up) and (down) stand before the scroller after its group; a press pages by 0.85 of the 200px scrollport (200 to 30, then 30 to the 200 end, clamped); ::scroll-button(*):disabled matches up at offset 0 and down at the end and neither in between. Logical directions are read as horizontal-tb left to right; prev/next are not taken; focus order is the tree's.")
proof("scroll-target-group", "css_interact", "scroll_markers_and_buttons_follow_the_scroll_position",
      "A scroll-target-group: auto list's <a href=\"#A\"> and <a href=\"#C\"> match :target-current, :target-before and :target-after for their sections in the scroller, by the same algorithm (A current at 0 and 100; C at 190, by step 6's second clause). Links whose targets are in no scroll container, or in several, are not grouped.")
proof("scroll-triggered-animations", "motion", "scroll_triggered_animations_play_and_reverse_by_their_ranges",
      "A trigger on a 100px subject 400px down a 200px scroller, timeline-trigger: --t view() contain / cover (activation 300-400, active 200-500): a 1000ms linear 0-120px width animation holds at exactly 0 until the offset is 350 (250, in cover only, does nothing), then grows 20px every 10 ticks; play-forwards play-backwards keeps playing at 250 and shrinks 20px every 10 ticks back to a held 0 at 100; replay reset goes to 0 and stays there on exit and restarts from 0 on the next entry; trigger-scope: --t on an ancestor keeps an animation inside at 0; animation-trigger: none plays at once; a checkpoint mid-play replays the same widths. One timeline per trigger (a list of several is not read); event triggers and the AnimationTrigger API are not built; an action takes effect one tick after the trigger changes; animation-trigger on a ::part plays at once.")
not_applicable("webxr-dom-overlays", "Styles the DOM overlay of a WebXR session; the engine renders its own UI in its world and has no WebXR DOM layer.")
not_applicable("window-controls-overlay window-drag", "Installed web-app title-bar integration (the OS window's caption buttons and drag regions); a game's UI is drawn inside its own surface, and the host owns its window.")
not_applicable("picture-in-picture", "Styles a video in the browser's floating picture-in-picture window, which is browser chrome outside the game's surface.")
not_applicable("fullscreen", "The Fullscreen API's :fullscreen and its ::backdrop style one element taken full screen by the browser; the game's surface is the screen, and the host decides whether its window is full screen.")
not_applicable("resizable", "@media (resizable) asks whether script may resize the browser window; the host, not a document, owns the game's window.")
proof("highlight", "css_selectors", "custom_highlights_paint_their_ranges_in_priority_order",
      "The spec's §4.2.5 example on 20px glyphs 1em wide: :root::highlight(Foo) (blue on yellow) over characters 0-6 and :root::highlight(bar) (orange) over 3-9 reach a paragraph by highlight inheritance; at a tied priority, bar, registered later, paints its quad above Foo's, and at priority 1 Foo paints above. The bands span exactly 0-120px and 60-180px, blue glyphs stay inside 0-120px, and the default ink is drawn past it. Two overlapping ranges paint as one band; a collapsed range paints nothing; a range from one paragraph into the next paints only the second paragraph's 40px, where #B::highlight(baz) styles it; removing every highlight clears the marks. A range's ends are text elements by path with character offsets (not DOM nodes and offsets); highlightsFromPoint() and the type attribute are not built; ::highlight() styles the same four properties as ::selection.")
proof("customizable-select", "html_controls", "a_base_select_builds_its_trigger_picker_and_options_from_its_children",
      "select { appearance: base-select } builds from its children: its <button>'s <selectedcontent> shows the bound option's content (\"gamma\"), then the chosen rich option's (\"alpha one\"), and a select with no button shows <option selected>'s label. The picker starts closed; a press opens ::picker(select) with its options in order, the first within 30px under the select, and the select matches :open (7px margin). option:checked and option:disabled size the options 60px and 30px against 40px. A press on the disabled option changes nothing and leaves the picker open; one on option A writes \"a\" (its value, not its label), fires changed and closes the picker; :checked moves to A; a press outside closes the picker with no write; an unbound select keeps the player's choice. ::picker-icon and ::checkmark are not built: they wait for control documents' pseudo= elements (engine-gaps). The picker's keyboard behaviour (arrows, typeahead) is not asserted here; the native select's is (html_controls::a_select_binds_its_value_by_keys_typeahead_pad_and_pointer).")
proof("file-selector-button", "html_controls", "file_selector_button_is_the_file_inputs_button_box",
      "Laid-out boxes and painted quads of <input type=\"file\"> with 4px padding and a blue background: its ::file-selector-button sits 4px inside it, shows \"Choose file…\", and the rule's red background fills exactly the button's box while the input keeps its blue. The rule's 10px inline padding makes the button as wide as a <button> with the same label and padding (the padding is the button plate's, not added to it), and its 6px margin-right moves the names exactly 6px further than an unstyled input's. The names read \"No file chosen\" and then the chosen file; a press on the names opens the picker; the input is the only stop and the button has no role of its own. A host without a file picker builds no file input (html_text::a_file_input_builds_only_where_the_host_has_a_picker). State pseudo-classes after ::file-selector-button (:hover) are not taken: it hovers and presses with its input.")
proof("view-transitions-element-scoped", "motion", "scoped_view_transitions_move_only_their_subtrees_names",
      "Laid-out quads and paint order across og.ui.startViewTransition on #Box plus a state change in the same tick: #Box's red view-transition-name: hero element sits every tick on lerp(old box, new box, t / 400ms) from the component document's own ::view-transition-group(hero) rule (no @view-transition opt-in), with both images while it runs and one after; a name outside #Box and one under a view-transition-scope: all element inside it snap to their new places; while it runs the group's quads come after #Box's background and its children's and before the element that follows #Box; a state saved at tick 10 and restored replays ticks 11 to 29 identically. The API is og.ui.startViewTransition(path) returning a bool, not Element.startViewTransition() with a ViewTransition object (no updateCallback, ready or finished promises, transitionRoot or activeViewTransition); the root is not given layout containment.")
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
proof("column-wrap", "css_layout", "column_height_wraps_columns_into_rows",
      "Computed boxes and rules: columns: 2 / 40px with a 10px column gap and 6px row gap in 210px makes 100px columns 40px tall; seven 20px blocks fill four columns in two rows (the fifth at 0, 46), the container is 86 tall, and each row draws its own 2px rule 40 tall in its gap; column-wrap: nowrap puts the overflow column beside at 220 and keeps 40; a row with 10px of content stays 40 tall. A spanner crossing a row boundary and wrap without a column-height (an open issue) are not asserted.")
proof("column-pseudo", "css_layout", "column_pseudo_elements_are_snap_areas",
      "Scroll offsets: in a 200px multi-column scroll container with a 20px gap, ::column { scroll-snap-align: start } settles a scroll at the next column (220), center at 165, and scroll-margin-inline-start: 40px at 180; with column_scroll_markers_scroll_to_their_columns, ::column::scroll-marker draws one marker per column in its ::scroll-marker-group (four markers for four columns), and pressing the third scrolls to 440 and the first back to 0; each marker takes :target-before, :target-current and :target-after by its own column (current, after, after, after; then before, before, current, after; then back) and :hover alone. A state-dependent `content` on a column marker, wrapped column rows, right to left and vertical writing modes are not asserted.")
proof("reading-flow", "css_layout", "tab_follows_reading_flow",
      "Tab sequence by focus path: flex-visual follows where order: -1 put an item (C A B), grid-columns walks a 2-column grid down each column (X Z Y W), source-order with reading-order: -1 puts that item first and ignores order, and a display: flex container with reading-flow: normal keeps the source order despite order (which it paints and hits in). Changing only reading-order, or only order, moves the Tab order on the next frame. flex-flow, grid-rows, grid-order, right to left and nested containers are implemented but not asserted; arrow moves stay spatial.")
proof("gap-decorations", "css_layout", "grid_gap_rules_break_as_css_gap_decorations_do",
      "Emitted rule quads of a 3x2 grid (40px columns, 20px rows, 10px column gap, 6px row gap) with an item spanning columns 1 to 3 in row 2: a 2px column rule is centred in each gap; under normal the first gap's rule stops at the spanning item (20 tall) and the second runs through the row gap (46); intersection breaks the second at the row gap (two 20 tall segments); rule-break: none runs both end to end; row-rule draws one 140 wide segment in the row gap. css_layout::gap_rules_in_flex_lines_and_rule_insets also proves a wrapping flex container (a rule between a line's items as tall as the line, none in a one-item line, a row rule across the content box) and rule-inset (a cap's -50% reads the 6px gap its end meets; on the grid, caps 2px in and junctions 3px into the row gap). rule-visibility-items, rule-overlap and lists of rule values are not supported; right to left is not asserted.")
proof("grid-lanes", "css_layout", "grid_lanes_pack_items_within_their_fit_tolerance",
      "Computed item positions in display: grid-lanes with three column lanes: each auto-placed item goes to the least-filled lane, lanes within fit-tolerance (normal 1em, 0, infinite) of it tie, and a tie takes the first lane at or after the last item's end line, else the first. Row lanes (only grid-template-rows given) lay out as a grid; inline-grid-lanes, percentage tolerances and subgrid items in lanes are not asserted.")
proof("margin-trim", "css_layout", "margin_trim_drops_the_margins_at_a_blocks_edges",
      "Computed boxes of a flow-root block: margin-trim: block drops the first in-flow block child's top margin and the last one's bottom margin, keeping the margin between them; block-start drops only the first; an absolutely positioned first child is skipped. Box 4 defines margin-trim for block containers only: the inline keywords and flex/grid trimming (Safari's extension) are refused, and a descendant margin collapsed through the trimmed one is not trimmed.")
proof("table", "css_layout", "border_spacing_separates_table_cells",
      "Computed cell and table boxes: border-spacing: 4px 6px lies between the cells and between the outer cells and the table's padding edge, an inherited single value applies to both axes, border-collapse: collapse puts the cells edge to edge, and a table ignores gap. Spans, captions and row groups are html_tables_lay_out_without_author_css and the computed table tests. Collapsed borders are not merged into one line (each cell draws its own), and the caption lies inside the spacing rather than outside the table box.")
proof("overflow-clip-margin", "css_contain", "overflow_clip_margin_moves_the_clip_edge",
      "Drawn clip rectangles: overflow-clip-margin: 10px moves an overflow: clip box's cut 10px out, border-box 2px measures from the border edge, content-box 3px from the content edge (inside an 8px padding), contain: paint takes it too and overflow: hidden ignores it; the corners round by their radius plus the margin. og-ui cuts a box without the property at its border edge (its border draws over the band to the padding edge).")
proof("anchor-positioning-position-visibility-plurals", "css_layout", "position_visibility_hides_by_each_keyword",
      "Drawn boxes: position-visibility: anchors-valid hides a box whose position-anchor names no element and keeps one whose anchor exists but is clipped; anchors-visible (the default) hides one whose anchor is clipped out of view and not one with no anchor; anchors-valid anchors-visible combines them; always hides for neither. A box whose only missing anchor is in a fallback that did not win stays shown.")
proof("anchor-positioning-transforms", "css_layout", "an_anchored_box_follows_its_anchors_transform",
      "Computed screen boxes: a box at left: anchor(right); top: anchor(top) of an anchor that takes translate: 100px 30px sits against the translated box. The transforms are composed after layout, so it follows one frame late: on the frame the anchor's transform changes it is where the anchor was, on the next where it is. Projective (3D) anchor transforms use the bounds of the projected box.")
proof("display static-positioning physical-properties align-content-block fit-content",
      "css_layout", "boxes_take_display_position_insets_and_fit_content_as_css_does",
      "Computed boxes: inline and inline-block boxes share a line and a block takes its own, full width; display: none makes no box and keeps no room; position: static ignores its insets; opposite insets size an auto box and an auto inset follows the other; block align-content: end; width: fit-content is max-content where it fits and the available width where it does not. fit-content on height and the min/max properties is not asserted.")
proof("z-index contain contain-paint overflow-clip", "css_contain", "z_index_orders_paint_and_clip_and_contain_cut_it",
      "Drawn order of three overlapping positioned boxes is z-index -1, auto, 2 against their source order; contain: paint, overflow: clip, contain: strict and contain: content push a clip at their box and contain: none does not; strict sizes as if empty and content keeps its height; overflow: clip makes no scroll container. z-index: 0 and auto order alike (both are level 0, so a positioned z-index: 0 makes no stacking context). Stacking contexts: css_contain::z_index_paints_in_the_nearest_stacking_context proves z-index 1 lifting a child over its parent's later sibling and -1 putting it under the parent's background, and opacity, transform, filter, mask, clip-path, mix-blend-mode, isolation, will-change, contain: paint and a level of its own each keeping it in, as each comes and goes. Scroll containers, shaped overflow clips, dressed parts and game draws also keep their levels in.")
proof("justify-self-block", "css_layout", "justify_self_aligns_block_level_boxes",
      "Computed x and width of block-level boxes: justify-self end with a fixed width, center with an auto width shrunk to fit-content, auto and normal at the start, and justify-items: end as the default that justify-self: start overrides. Absolutely positioned boxes and right-to-left blocks are not asserted.")
proof("line-clamp", "css_layout", "line_clamp_cuts_a_texts_lines",
      "Computed heights: line-clamp: 2 keeps two lines of a <p> and a <text> without text-overflow, the box those lines high; a text within the limit keeps its lines. A <div> holding text directly refuses line-clamp (it applies to text-bearing elements), and the drawn ellipsis is not asserted.")
proof("overflow-overlay", "css_contain", "overflow_overlay_scrolls_as_auto",
      "Scroll state after a wheel of (20, 30): overflow: overlay and overflow-y: overlay make 40px scroll containers over 300px content that take both axes of it (overflow-y's visible x computes to auto), where overflow: hidden takes none.")
proof("zoom", "css_layout", "zoom_scales_a_subtrees_lengths_as_css_does",
      "Computed screen boxes: zoom: 2 on a content-box 100px box with 5px padding and a 10px top margin makes it 220 wide 20 below its block's top, its child's width: 50% 100 wide at 10, 10, and its text twice its unzoomed size, the next box below it; a zoomed width: 50% stays 200 of 400; zoom: 1.5 inside zoom: 2 makes a 10px box 30; a zoomed flex item's content-box size and its padding floor are its parent's room; zoom: 50% halves and zoom: 0 is 1. The subtree lays out at its own lengths and is drawn scaled from the box's top left (its transforms act inside the zoom); a zoomed box starts its own float context, and a margin collapsing through it keeps only its sum's sign. getComputedStyle-style unscaled values are not exposed.")
proof("frame-sizing", "css_layout", "frame_sizing_follows_an_embedded_documents_content",
      "Computed boxes of <iframe>s embedding the game's own documents: frame-sizing: content-height with a 200px width makes the frame its opted-in document's height (two 20px rows, a line and 10px of padding), and a keyed insert into the bound list grows it by one row; content-width on a document without <meta name=\"responsive-embedded-sizing\"> keeps HTML's 300 x 150; auto with CSS sizes keeps 120 x 40. The frame is its own cascade: the embedder's 40px font-size does not reach its text, its .row min-height rule matches nothing inside, and the style pack's .row applies while the pack's #Page .row does not see the page around the frame. A remote frame's sizing is the same path (css_layout::a_remote_frame_shows_its_fetched_document_and_sends_only_messages sizes one under content-height) and is not part of this proof.")
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
      "Emitted layers read through og-render's own placement land where CSS Backgrounds 3 arithmetic puts them: edge-offset and percentage positions, background-position-x over the shorthand, repeat-x, space and round tiling, cover and contain, content-box origin, padding-box colour clip, fixed attachment from the viewport origin, and per-layer blend modes with the first layer drawn last. background-attachment: local and background-clip: text are not asserted here (css_paint::background_clip_text_cuts_to_the_descendants_text).")
proof("background-clip-border-area", "css_paint", "border_area_paints_a_layer_only_under_the_border",
      "GPU pixels prove a border-area colour and an image layer over a colour paint only under a 6 px transparent border, leaving the padding box empty or the colour beneath. Rounded and shaped corners follow the box's corner geometry but are not asserted.")
proof("opacity outline crisp-edges clip path-shape",
      "css_paint", "opacity_outline_crisp_edges_and_clips_draw_as_css_says",
      "Emitted output: opacity as a number and a percentage; outline thin/medium/thick widths (1/3/5 px), dashed, double's two thirds-wide lines spanning 5 px, outline-offset and none; image-rendering: crisp-edges sampling nearest; CSS 2 clip: rect() with commas, spaces and auto edges on absolute boxes only; clip-path path() points and both fill rules. path() for shape-outside, offset-path and the d property is not covered.")
proof("opacity-svg paint-order mask-type mask-border",
      "css_paint", "svg_paint_opacities_order_and_mask_types_draw_as_svg_says",
      "SVG fill-opacity and stroke-opacity (attributes, percentages and CSS) scale only their paint; paint-order draws a shape's stroke under its fill, on the shape or inherited, with omitted paints in default order; a <mask>'s mask-type attribute (alpha vs luminance) changes the emitted gradient mask; mask-border slices a gradient's border area. Text paint-order, mask-type in a stylesheet rule and non-gradient masks are not covered.")
proof("svg", "inline_svg", "inline_svg_matches_its_reference_frames",
      "GPU pixels at 1280 (inline_svg::svg_shows): a 40-unit view box fitted to a 96px box draws an even-odd ring gold and leaves its hole unfilled, and a keyed {#each} bar's url(#Up) gradient runs blue at its foot to green at its top in a met view box. The 1280 and 390 frames, with the polygons, the pathLength dash offset and bound path data, are compared with their references for review. css_paint::svg_paint_opacities_order_and_mask_types_draw_as_svg_says adds paint opacities, paint-order and mask types. The SVG DOM interfaces, <foreignObject>, <a>, <script>, <switch> and systemLanguage are not covered.")
proof("image-orientation", "css_paint", "image_orientation_turns_images_by_their_exif_or_draws_them_as_stored",
      "Emitted images read through og-render's own placement against a 2x1 texture carrying EXIF orientations 1, 3, 6 and 8: from-image turns it as EXIF defines (6 red on top, 8 blue on top, 3 blue on the left) with the turned natural size under object-fit: none; none, inherited from the parent, samples the texels as stored; a background follows the property too. og-cook textures::exif_orientation_is_kept_beside_the_texels_as_stored: the cook keeps the orientation beside unturned texels, a streamed tail too. The deprecated <angle> || flip form is refused; sprites and materials sample as stored; the GPU shader path is not pixel-checked.")
proof("cross-fade", "css_paint", "cross_fade_sums_its_weighted_images_in_a_group",
      "Emitted output: each part placed as the layer places one image, scaled by its weight and added (plus-lighter) in a group composited as one image; weights normalised as the spec says (an omitted one takes what is left, floored at 0; a sum over 100% scaled; under it the rest transparent), colours as solid images, nested cross-fade() flattened; out-of-range percentages and none refused. Parts keep their own natural sizes rather than the weighted average (equal-sized images are exact; a colour fills the positioning area under an auto size), the sum is in og-render's linear premultiplied space rather than the spec's sRGB, and -webkit-cross-fade() is not taken; the GPU sum is not pixel-checked.")
proof("filter-function", "css_paint", "filter_images_run_their_functions_in_order_on_the_image",
      "Emitted output: filter(<image>, ...) draws the layer's image through one layer per stage, the first applied innermost, with Filter Effects 1 §13's matrices (grayscale(1)'s first row, invert then opacity) and blur's sigma; a nested filter() folds into one list; a cross-fade part filtered on its own adds through its stage at its weight; url(#id) and none are refused. A string source is a url. The filtered image keeps the unfiltered one's size and placement; a blur or drop shadow may spill past the image within its painting area; filter(cross-fade()) used as a cross-fade part loses the inner sum. GPU pixels are not checked.")
proof("background-clip-text", "css_paint", "background_clip_text_cuts_to_the_descendants_text",
      "Emitted output: a box whose text is all in two child paragraphs draws its gradient in one group under both paragraphs' glyphs, opaque white without glow (its bottom layer kept on them by source-in, later layers cut to them again), before each paragraph's own text, which draws once over it; a frame that keeps the box's subtree while a sibling changes equals the same state built whole, also under OG_UI_CHECK_BOUNDED. Text drawn by a scene, a list marker or a control's own parts is not asserted; GPU pixels are not checked.")
proof("element", "css_paint", "element_images_draw_the_element_in_each_tile",
      "Emitted output: element(#id) as a background draws the first drawn element of that id in each tile CSS Backgrounds 3 places at its border box size (5px 0 repeat-x over 50 px: tiles at x = -15, 5, 25, 45, each 20x10), without its own transform or its ancestors' opacity and clips, its live drawing on the frame it changes; itself or an ancestor (a cycle) is an invalid image. Frames that keep the element or the siblings after the image's group equal the same state built whole, also under OG_UI_CHECK_BOUNDED. Mask layers take element() the same way but are not asserted; a border image or a cross-fade() part of element() draws nothing, an element() inside the element drawn is empty in the copy, paint sources (a <canvas> or an undrawn <img>) are not read, the decorated box is the border box (no border-image outset), at most 256 tiles draw, and GPU pixels are not checked.")
proof("box-decoration-break", "css_paint", "box_decoration_break_slices_or_clones_a_split_text",
      "Emitted output and computed boxes of a 15-line text with 3 px padding and a 2 px border split by two balanced 80 px columns: slice draws the background and border of the 310 px unbroken box once per piece, moved with its lines and cut at the break (the first piece shows 0 to 165, the second 0 to 145 in the next column, so only the outer edges have borders); clone gives each piece a whole box, its padding and border at the break taken into balancing (columns of 170 and 150, the second piece's first line inside its own padding). css_paint::inline_boxes_draw_on_their_line_fragments proves the inline case on a span wrapped over two lines of 20 px ideographs (fragments 40 to 100 and 0 to 60): its background and border draw under its words, under clone a whole box on each fragment, under slice the 120 px unbroken box from 40 then from -60 cut to each fragment along the line, right to left continued leftward from the first fragment's right edge (-20, then 0), the same after a change elsewhere and under OG_UI_CHECK_BOUNDED. The inline padding and border room in the line is the inline layout's (fragments as wide as their words until it lands); the -webkit- alias is taken but not asserted; border-image, box-shadow and border-radius on a split box follow the same pieces but are not asserted, a layered (filtered or masked) split box composites over the pieces' bounds, and GPU pixels are not checked.")
proof("border-shape", "css_paint", "border_shape_strokes_one_shape_and_fills_between_two",
      "Emitted output of CSS Borders 4 §7.4's examples: circle(50%) on a 100 px box with a 5 px border is one 5 px line centred on a 47.5 px circle over the half-border box, with no rectangle ring; the 200 px diamond pair fills the border between the outer diamond over the border box (vertices 100 px from the centre) and the inner one over the padding box (72 px), even-odd, in the border colour, its background in the outer diamond and its border-radius ignored; the width and colour come from the first styled side in block-start, inline-start, block-end, inline-end order, left to right and right to left. Shadows, background layers and overflow follow the outer shape through the element's shape but are not asserted; overflow is cut at the outer shape (the border, drawn over the children, covers the ring); an inner shadow and the outline follow the box, not the inner shape or the outer contour; dashed and dotted strokes are not asserted, and GPU pixels are not checked.")
proof("backdrop-filter", "css_paint", "glass_takes_any_backdrop_filter_chain",
      "Emitted output: on a glass surface, blur(4px) saturate(0) folds into the glass backdrop (blur 4 px at the glass scale, the grey matrix in it, no staged layer); saturate(0) blur(4px) drop-shadow(2px 3px 4px #000) runs as a staged backdrop layer in order (Filter Effects 1's s = 0 matrix with rows 0.213 0.715 0.072, a 4 px deviation blur, then the shadow's deviation 2 blur, 2 3 offset, black flood, composite in and merge) and the glass draws over it with no blur or matrix of its own, without a warning. Plain backdrop-filter chains, url() filters and their pixels are css_filters::css_filters_match_their_reference_frames' (GPU references, og-tests-ui); the glass-over-staged pixels are not checked on the GPU.")
not_applicable("color-adjust print-color-adjust",
               "Print colour economy: a game draws to its screen, never to paper or a printer's economy mode.")
not_applicable("paint",
               "The CSS Painting API runs a browser JavaScript paint worklet (CSS.paintWorklet, PaintRenderingContext2D); a document has no worklet runtime, and engine art comes from materials, vectors and images.")
not_applicable("svg-discouraged",
               "SVG 1.1 features SVG 2 removed (xlink: attributes, SVG fonts, SVGPathSeg and other DOM methods, zoomAndPan, baseProfile); documents are SVG 2 markup with no script DOM.")

# selectors and cascade
proof("selectors has is where not nth-of-type root scope scope-pseudo case-insensitive-attributes case-sensitive-attributes lang dir-pseudo state link-selectors nesting input-selectors indeterminate default read-write-pseudos placeholder-shown open-pseudo",
      "css_selectors", "selectors_select_as_css_does",
      "Computed widths, each against a decoy in the other state: type, class, id and attribute selectors with the child, adjacent and general sibling combinators; :has(> …); :is(); :not(); :where() matching with zero specificity; :nth-of-type(2) among other tags; a :root custom property; :scope; @scope (.card) to (.inner) stopping at its limit; [k=v i] and [k=v s], with s refusing input[type=\"CHECKBOX\" s] that HTML's case-insensitive type matches without it; :lang(fr) on fr-CA; :dir(rtl); :state(on) bound true and false; :any-link; nesting's &; :checked, :disabled, :enabled and :indeterminate inputs; :default on a checkbox its markup checks and not on one only its binding checks; input:read-only; :placeholder-shown; :out-of-range; details:open. :visited and :link's history are not modelled (every link is unvisited). css_selectors::the_open_attribute_reflects_the_open_state proves [open] on <details> and <dialog> follows the open state as HTML reflects it: toggled by the player, held by bind:open, and a literal open the player closed.")
proof("form-validity-pseudos", "input_types", "html_constraints_readonly_rows_and_label_checkboxes",
      "Computed widths: an empty required field and one under its minlength match :invalid, valid and empty optional fields do not. :valid, :in-range and :user-valid are not asserted here.")
proof("cascade-layers", "css_selectors", "cascade_layers_order_a_documents_rules",
      "Computed widths under a document's own @layer rules: @layer a, b, c; orders them whatever the source order or specificity (a's #Order.probe loses to b's .probe), unlayered rules beat every layer, !important reverses both (a's important beats b's, and a layer's an unlayered one), an anonymous @layer {} takes its place where it first appears, a layer's own rules beat its nested @layer inner, and sublayers declared c.x, c.y rank in that order. "
      "style_pack::a_rule_ranks_by_its_tree_context_then_its_layer proves the engine's own og-components and og-pack layers under a game's rules.")
proof("shadow-parts", "dressing", "a_part_rule_reads_its_hosts_place_in_the_document",
      "A built-in control's parts are a shadow tree under its element: .shop button::part(base) and ul > li:first-child button::part(base) widen only the button they reach, from a document and from a pack, and follow a class change. part lists (one element under two names) are covered by the role parts' tests; A control in a control document exports its parts to the outer host with exportparts, renamed or not, one boundary deeper: control_documents::exportparts_makes_a_nested_part_the_outer_hosts.")
proof("active-view-transition", "css_selectors", "active_view_transition_matches_the_root_while_it_runs",
      "Computed widths through a screen layer's default 250ms view transition: :active-view-transition and :root:active-view-transition-type(slide) descendant rules widen probes on the first frame and not after it ends; a type the screen's @view-transition does not declare (fade) never matches, a list matches by any of its types, the root alone matches (.probe:active-view-transition never does), and a sibling rule matched on the instantiated tree reads the types too.")
proof("user-action-pseudos focus-visible user-pseudos target", "css_selectors", "interaction_pseudo_classes_follow_the_pointer_and_keys",
      "Computed widths, each beside a decoy: :hover under the pointer and :active only while it is down; a clicked button matches :focus but not :focus-visible, and the same button reached by Shift+Tab matches both; an untouched required email field is neither :user-invalid nor :user-valid, typed 'x' it is :user-invalid, typed on to 'x@y.z' it is :user-valid; following an <a href=\"#Goal\"> makes #Goal :target and not its sibling. A field's typed text counts as changed by the player once it is non-empty (no blur is waited for); css_selectors::an_unbound_field_keeps_its_own_value proves the same for a field without bind:value, across a snapshot.")
proof("modal", "css_selectors", "modal_matches_only_a_modal_dialog",
      "Computed widths: a probe inside <dialog modal open> matches :modal .probe, one inside an in-place <dialog open> does not.")
proof("heading-selectors", "css_selectors", "heading_matches_headings_by_level",
      "Computed boxes of <h1>, <h2>, <h3> and a <p> decoy: :heading indents the three headings and not the paragraph, :heading(1, 3) widens h1 and h3 only, and :heading + .probe and :heading(2) + .probe (matched on the instantiated tree) reach the probes after the right headings. :heading(0) is refused.")
proof("revert-rule", "css_selectors", "revert_rule_rolls_back_one_rule",
      "Computed widths: width: revert-rule shows the rule below's width; an earlier declaration of the same rule is rolled back with it; an unlayered rule below wins over a layered one (where revert-layer would pick the layer); a custom property and a var() fallback roll back the same way; font-size: revert-rule restores the rule below's size (read through an em width); with no rule below the property is as never declared while the rule's other declarations apply; a rule that does not match rolls nothing back.")
proof("media-pseudos", "css_selectors", "media_pseudo_classes_follow_the_element",
      "Computed widths of sibling probes of a <video>: :paused while its bind:paused is true and :playing once it is false, :muted with bind:muted, :seeking from a bind:currentTime write until the seek reaches the player. :buffering, :stalled and :volume-locked parse and never match: a player reads a local asset and the engine mixes its own sound.")
proof("host", "css_selectors", "host_selects_the_instance",
      "Computed boxes in two instances of one component: :host's padding and custom property style the root's box and inherit; :host([tone=warn]) .x, :host([wide]) > .x and :host(:not([wide])) .x follow the props the instance passes (a false or unpassed prop is absent) and its store; :host(.featured) reads the instance's class; :host(Card) matches the component's name and :host(Other) never; :host(:nth-child(2)) is the instance's place among its siblings; :host > .x stops at the root's children; :host([tone=warn]) li:nth-child(2), matched on the instantiated tree, holds only under that host. css_selectors::host_reads_the_instances_state proves :host(:hover) and :host(:disabled) follow the root's live state per instance, and css_selectors::host_refusals_name_their_fix the refusals (:host.wide, :host:hover, an attribute with no prop, an id, a component whose root is an instance). An instance and its root are one element: there is no separate host box.")
proof("slot has-slotted", "css_selectors", "slotted_selects_content_rendered_at_a_site",
      "Computed boxes of content passed to four instances of one component, whose {@render} sites are its slots (og has no <slot> element; the element around a site answers for it): ::slotted(p) and .body::slotted(.lead) move the top-level passed elements and not a nested one, the component's own fallback snippet or content at the other site; a ::slotted() rule loses to the content's own document's rule whatever its specificity, wins over it with !important and over the user-agent sheet's margins; .body.open::slotted(p) follows the holder's class; .body:has-slotted and .foot:has-slotted match only where passed content is rendered, not the fallback, and follow content under an {#if} as it comes and goes. css_selectors::slotted_refusals_name_their_fix proves the refusals (a combinator or pseudo-class after ::slotted(), a complex argument, a pseudo-element after it, :host() before it). A component instance passed as content is slotted too: :has-slotted counts it, ::slotted() reads its root and wins over its component's own rules for it; an instance the component wrote itself is not. Elements rendered at two sites are not siblings.")
not_applicable("host-context",
               ":host-context() was dropped from CSS Scoping and Firefox never shipped it; :host() with an ancestor selector in the document covers the case.")
not_applicable("autonomous-custom-elements",
               "customElements.define() is a script API. A game's components are .svelte documents used by their own tag, with no script to register a class.")

MISSING = {
}

if overlap := PROVEN.keys() & MISSING.keys():
    raise ValueError(f"a proof must state its remaining gaps, not hide MISSING: {sorted(overlap)}")
if overlap := NOT_APPLICABLE.keys() & (PROVEN.keys() | MISSING.keys()):
    raise ValueError(f"a feature is either not applicable or proved/missing: {sorted(overlap)}")
if overlap := NOT_NEEDED.keys() & (PROVEN.keys() | MISSING.keys() | NOT_APPLICABLE.keys()):
    raise ValueError(f"a feature is either not needed or proved/missing/not applicable: {sorted(overlap)}")

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
    "Emitted button base radius replaces its default in a mounted document.")
part_proof("font-family",
    "runs", "ordered_named_families_preserve_primary_glyphs_and_inherit",
    "The document emits button::part(base) glyphs from ordered named families, including a named fallback after an intervening generic.")
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
    """The file of `module` that defines `test`: og-ui's, the frame tests' (og-tests-ui), or the runtime's
    (og-runtime: what an engine fetches and loads). The test must assert on what it built: an assert or
    panic in its body or in a function of its file or its crate's test support that it calls. A reference
    frame alone is not one: `matches_reference` reports a moved frame and passes. This finds a test that
    checks nothing; it cannot tell an assertion about the feature from one beside it (a render helper's
    no-warnings assert), which review of the PROVEN entry still owns."""
    for crate in ("og-ui", "og-tests-ui", "og-runtime"):
        source = f"crates/{crate}/tests/{module}.rs"
        if (ROOT / source).exists() and f"fn {test}(" in (ROOT / source).read_text():
            texts = [(ROOT / source).read_text()]
            texts += [f.read_text() for f in sorted((ROOT / f"crates/{crate}/tests/support").rglob("*.rs"))]
            if not asserts(test, functions(texts)):
                raise ValueError(f"{kind} {module}::{test} asserts nothing about its output")
            return source
    raise ValueError(f"missing {kind} entry point: {module}::{test}")


FN = re.compile(r"\bfn (\w+)\s*[<(]")
ASSERTS = re.compile(r"\b(assert(_eq|_ne)?|panic|unreachable)!|\.(unwrap_err|expect_err)\(")
# Calls that check a frame only against its reference, which never fails on a moved frame.
NOT_ASSERTING = {"matches_reference", "check_reference", "report_moved"}


def functions(texts):
    """Every `fn` of the Rust sources by name: its body, to its closing brace (nested functions inside)."""
    out = {}
    for text in texts:
        for m in FN.finditer(text):
            start = text.find("{", m.end())
            depth, end = 0, start
            for end in range(start, len(text)):
                depth += {"{": 1, "}": -1}.get(text[end], 0)
                if depth == 0:
                    break
            out.setdefault(m.group(1), text[start:end + 1])
    return out


def asserts(name, bodies, seen=None):
    seen = set() if seen is None else seen
    if name in seen or name in NOT_ASSERTING or name not in bodies:
        return False
    seen.add(name)
    body = bodies[name]
    return bool(ASSERTS.search(body)) or any(
        asserts(callee, bodies, seen) for callee in re.findall(r"\b(\w+)\s*\(", body) if callee in bodies)


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
    if len(features) != 454 or (PROVEN.keys() | MISSING.keys() | NOT_APPLICABLE.keys() | NOT_NEEDED.keys() | PART_PROVEN.keys()) - features.keys():
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
             "(print, cross-document navigation, browser chrome), with the reason in the row. "
             "**Not needed means the feature could apply to games but no game needs it**: the row is closed "
             "without an implementation, with the reason in the row. Parser acceptance, "
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
             "`crates/og-ui/tests/` (what the UI builds), `crates/og-tests-ui/tests/` (the frames drawn of it) "
             "or `crates/og-runtime/tests/` (what an engine fetches and loads for it), "
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
        elif key in NOT_NEEDED:
            state = "Not needed"
            reason = NOT_NEEDED[key]
        else:
            state = "Missing"
            reason = MISSING.get(key, "Output proof not yet reviewed; implementation status unverified.")
        part_state = "Not applicable" if key in NOT_APPLICABLE else "Not needed" if key in NOT_NEEDED else "Missing"
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
          f"{len(NOT_NEEDED)} not needed; {454 - len(PROVEN) - len(NOT_APPLICABLE) - len(NOT_NEEDED)} missing/gap rows")


if __name__ == "__main__":
    main()
