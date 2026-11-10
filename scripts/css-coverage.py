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


proof("aspect-ratio box-sizing width-height min-max-width-height fit-content-function min-max-content logical-properties padding",
      "css_layout", "the_box_model_sizes_as_css_does",
      "Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths.")
proof("flexbox flexbox-gap", "css_layout", "flex_lays_out_as_css_flexbox_does",
      "Computed grow/shrink shares, reverse/order, safe alignment, wrapping, row and column gaps.")
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
      "Metal pixel assertions for both halves of all four repeating conic quarters.")
proof("masks visibility", "css_paint", "masks_hide_invisible_boxes_and_fade_image_content",
      "Metal pixels prove hidden masked boxes do not paint, visible descendants retain the mask, and gradient masks fade both div and translucent image content.")
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
      "Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles.")
proof("filter svg-filters", "css_filters", "css_filters_match_their_reference_frames",
      "Metal references at 1280/390 compare SVG filter graphs, primitive tiles and CSS function/url composition; this does not prove native glass backdrop chains.")
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
      "Metal pixels prove repeating radial intervals with positive/negative starts, all eight conic half-quadrants and collapsed linear stops. Subpixel periods and unbounded stop counts are not covered.")
proof("margin", "html_text", "block_margins_follow_the_ua_stylesheet_and_collapse",
      "Both builders assert positive/negative sibling and parent-child margin collapse, padding/fixed-height boundaries, noncollapsing flex margins, UA em margins and keyed insertion.")
proof("text-wrap-balance text-wrap-pretty text-overflow", "html_text", "text_wrap_styles_and_overflow_strings_lay_out_as_css_does",
      "Both builders inspect emitted lines: balance reduces width spread at the same line count; pretty avoids the final single-word line; custom overflow text replaces the ellipsis on one clipped line.")
proof("container-queries", "documents", "container_queries_restyle_lists_grids_and_rive_frames_before_paint",
      "Viewport changes immediately change list/grid padding and Rive visibility under a named inline-size query. Pushed-screen path; style/scroll-state queries and container units are not covered.")
proof("scroll-driven-animations", "documents", "view_timelines_follow_the_subject_through_its_scroller",
      "Both builders assert width and translation at rest and after wheel input for an anonymous view() cover timeline. Named timeline scopes and all range keywords are not covered.")
proof("animations-css cubic-bezier-easing", "documents", "scoped_css_motion_uses_shared_curves_emits_sound_and_restores",
      "A cubic-bezier(0,1,0,1) keyframe animation reaches 7/8 displacement at 1/8 duration; twelve subsequent positions replay exactly after checkpoint restoration. Pushed-screen path, not every animation mode.")
proof("color", "dressing", "host_font_and_color_reach_button_ink_and_divider_output",
      "Emitted button text and divider fill match authored hex colors; an explicit button part color overrides inherited host ink. This does not cover every color syntax or space.")
proof("image-set", "css_paint", "image_sets_render_density_correct_backgrounds_masks_and_borders",
      "Metal pixels at DPR 1/2/1 prove density-correct background/mask tiles, MIME filtering before duplicate removal (including quoted parameters and malformed-type rejection), zero-resolution transparency, percentage border slices with auto widths and gradient selection. Resolution math covers calc/min/max/clamp with mixed x/dppx/dpi/dpcm units; retained output also tests fractional DPR.")
proof("background-image", "css_paint", "image_urls_resolve_files_not_atlas_names_or_handles",
      "Retained drawing selects the registered relative file and numeric filename, rejects a missing path despite an atlas basename collision, and reports the missing asset. css_paint_matches_its_reference_frames covers background tiling and gradients on Metal. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks proves cooked SVG pixels, imported masks and inline cursor URL provenance.")
proof("cursor", "css_interact", "plain_boxes_choose_cursors_in_clipped_transformed_paint_order",
      "Both builders assert cursor inheritance over noninteractive boxes, passive overlays above buttons, pointer-events:none, hidden and overflow-clipped children, 90-degree transforms and image hotspots. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks checks cooked SVG cursor pixels on a plain div. css_interact::cursors_apply_only_inside_clip_paths proves, in both builders, cursors only inside circle(), polygon() over content-box, evenodd path(), shape(), objectBoundingBox url(#id) and an ancestor's inset() clip. Clip-path press hit testing and every cursor keyword are not covered.")
proof("field-sizing", "control_props", "fields_size_to_displayed_content_and_respect_constraints",
      "Both builders prove text-input value/placeholder sizing, caret-width empty inputs, textarea wrapping and newline heights, ignored rows, explicit dimensions, min/max width, noninheritance and removal of content sizing. Retained editing and checkpoint restore preserve focus/caret and update height. Select/file controls and vertical text controls remain missing or unverified.")
proof("shape-outside shapes", "css_layout", "shape_outside_changes_line_exclusion_not_float_placement",
      "Both builders prove triangular line exclusion, unchanged float stacking/BFC avoidance/clear, and dynamic none/restoration. shape_outside_uses_the_whole_line_band_and_reference_box proves circles, ellipses, insets, margin/border/content boxes, margin-box clipping, degenerate insets and dropping a wide line along a slope. shape_margin_expands_by_distance_and_clips_to_the_float_box proves distance-based shape-margin, containing-block percentages, clipping and dynamic zero restoration. Image alpha contours are missing; other syntax, elliptical box corners and writing modes remain unverified.")

MISSING = {
    "default": "documents::boolean_attribute_selectors_follow_svelte_presence proves retained :default follows false/present checked attributes. The immediate builder does not expose :default state; full support is missing.",
    "interpolate-size": "No intrinsic-size interpolation implementation or output proof.",
    "calc-size": "No calc-size evaluation or animated output proof.",
    "table": "Table spans/captions have computed output tests, but border-spacing and border-collapse have no implementation/output proof.",
    "backdrop-filter": "Ordinary staged backdrop chains have fixtures; native glass rejects chains requiring intermediate clamps, drop-shadow and SVG url filters.",
    "shadow-parts": "Host/part/state selector registry is present; all-property part applicability awaits the parts-as-nodes stream and its output tests.",
}

if overlap := PROVEN.keys() & MISSING.keys():
    raise ValueError(f"a proof must state its remaining gaps, not hide MISSING: {sorted(overlap)}")

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
    "Metal pixels prove button::part(base) reselects its background at DPR 1/2/1 while preserving an 8 CSS px tile. Other part image properties are not separately proved.")


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
    if len(features) != 454 or (PROVEN.keys() | MISSING.keys() | PART_PROVEN.keys()) - features.keys():
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
             "A missing proof is not evidence that the implementation is absent. Parser acceptance, "
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
             "`crates/og-ui/tests/` and run in the `og-ui` integration test binary; GPU proofs require Metal.",
             "",
             "| WebDX feature | Baseline | Document elements | Widget ::part | Proof scope or missing reason |",
             "| --- | --- | --- | --- | --- |"]
    for key, feature in sorted(features.items()):
        baseline = feature["status"]["baseline"]
        label = {"high": "Widely", "low": "Newly", False: "Not Baseline"}[baseline]
        if key in PROVEN:
            module, test, scope = PROVEN[key]
            source = ROOT / f"crates/og-ui/tests/{module}.rs"
            if f"fn {test}(" not in source.read_text():
                raise ValueError(f"missing proof entry point: {module}::{test}")
            state = "Supported (scope →)"
            reason = f"[`{module}::{test}`](../crates/og-ui/tests/{module}.rs). {scope}"
        else:
            state = "Missing"
            reason = MISSING.get(key, "Output proof not yet reviewed; implementation status unverified.")
        part_state = "Pending parts-as-nodes"
        for module, test, scope in PART_PROVEN.get(key, []):
            source = ROOT / f"crates/og-ui/tests/{module}.rs"
            if f"fn {test}(" not in source.read_text():
                raise ValueError(f"missing part proof entry point: {module}::{test}")
            part_state = "Supported (scope →)"
            reason += f" Part proof: [`{module}::{test}`](../crates/og-ui/tests/{module}.rs). {scope}"
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
    print(f"454 CSS entries: {len(PROVEN)} scoped output proofs; {454 - len(PROVEN)} missing/gap rows")


if __name__ == "__main__":
    main()
