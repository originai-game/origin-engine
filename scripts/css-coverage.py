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

MISSING = {
    "gradients": "Metal pixel proofs cover nonzero repeating intervals and coincident stops (css_paint::repeating_gradients_repeat_the_stop_interval), and out-of-range non-repeating linear/radial/conic stops (css_paint::nonrepeating_gradients_keep_stops_outside_the_image). Full feature remains missing: subpixel periods and unbounded stop counts are not proved.",
    "image-set": "URL rewriting descends into image-set, but image candidate selection has no implementation/output proof.",
    "interpolate-size": "No intrinsic-size interpolation implementation or output proof.",
    "calc-size": "No calc-size evaluation or animated output proof.",
    "field-sizing": "No field-sizing layout implementation/output proof; fixed control sizing is not content sizing.",
    "shape-outside": "Rectangular floats have layout tests; shape exclusion contours and shape-margin have no output proof.",
    "shapes": "clip-path does not prove shape-outside float exclusion.",
    "table": "Table spans/captions have computed output tests, but border-spacing and border-collapse have no implementation/output proof.",
    "backdrop-filter": "Ordinary staged backdrop chains have fixtures; native glass rejects chains requiring intermediate clamps, drop-shadow and SVG url filters.",
    "shadow-parts": "Host/part/state selector registry is present; all-property part applicability awaits the parts-as-nodes stream and its output tests.",
    "background-image": "Gradient/reference coverage exists; url(coin) resolves an atlas name, not a standard file URL. Icon uses must migrate to SVG file masks.",
}


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
    if len(features) != 454 or (PROVEN.keys() | MISSING.keys()) - features.keys():
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
             "Part applicability is pending the parts-as-nodes stream for every row: an element test "
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
        lines.append(f"| `{key}` | {label} | {state} | Pending parts-as-nodes | {reason} |")
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
