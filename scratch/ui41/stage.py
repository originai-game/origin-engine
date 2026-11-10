"""The stage of a fn in og-ui: file default, fn overrides, end_frame regions."""
import re

T, S, L, P, D, F, E, O, C = "tree", "style", "layout", "paint", "docs", "front", "env", "observe", "ctor"

FILE = [
    # documents: the retained model, its lowering and its layer stack
    (r"^src/doc/build/(cascade|parts|layout_props|fields|animate|generated)\.rs$", S),
    (r"^src/doc/build/svg/", D),
    (r"^src/doc/(css|stylesheet|selectors|matching|calc|variables|at_rules|syntax|text_css|paint_css|position_css|scroll_css|image_css|interact_css|contain_css|multicol_css|boxes|media|container|tracks|lists|motion|view_transition)", S),
    (r"^src/doc/", D),
    (r"^src/(layers|slot|bind|markup)\.rs$", D),
    (r"^src/reactive", D),
    # style and motion
    (r"^src/motion/gesture\.rs$", T),
    (r"^src/motion/", S),
    (r"^src/(view_transition|theme|tokens|style|style_file|style_pack|dress|css_color|sound_css|transform3d)\.rs$", S),
    (r"^src/(tokens|style_file)/", S),
    (r"^src/ui/css_tokens\.rs$", S),
    # layout
    (r"^src/layout", L),
    (r"^src/element/frame\.rs$", L),
    (r"^src/ui/(settlement|bounded)\.rs$", L),
    (r"^src/widgets/geometry\.rs$", L),
    (r"^src/world/placement\.rs$", L),
    (r"^src/text", L),
    # paint
    (r"^src/ui/(commands|node_draw|dressing)\.rs$", P),
    (r"^src/(paint|paint_entry|draw|draw_codec|decor|particles|glyphs|colr|shape|svg|text_path)\.rs$", P),
    (r"^src/(paint|draw|svg|rive)/", P),
    (r"^src/rich", L),
    # tree and interaction
    (r"^src/ui/(build|tree|visit|select|resize|descriptors)\.rs$", T),
    (r"^src/(nav|recognize|scroll|edit|vcursor|actions|native_a11y|a11y|input|input_type|id|element|viewport|virtualize|hud_layout|world)\.rs$", T),
    (r"^src/element/", T),
    (r"^src/world/frame\.rs$", T),
    # the immediate front end
    (r"^src/widgets", F),
    (r"^src/ui/(controls|text_controls)\.rs$", F),
    (r"^src/juice\.rs$", F),
    # configuration, output, observers
    (r"^src/ui/configure\.rs$", E),
    (r"^src/ui/(snapshot|inspect|debug)\.rs$", O),
    (r"^src/probe\.rs$", O),
    (r"^src/(ui|lib)\.rs$", E),
]

FN = {
    # ui/build.rs public builders are the front end
    ("src/ui/build.rs", n): F for n in (
        "open", "close", "container", "row", "column", "panel", "open_panel", "scroll",
        "open_scroll", "leaf", "text_content", "tooltip", "open_layer", "open_layer_keyed",
        "layer_place", "set_draw", "set_draw_dressable", "respond")
}
FN.update({
    ("src/layout/node.rs", "new"): C,
    ("src/layout/node.rs", "inherit_layout"): C,
    ("src/ui/configure.rs", "new"): C,
    ("src/ui/configure.rs", "take_feedback"): O,
    ("src/ui/configure.rs", "take_haptics"): O,
    ("src/ui/configure.rs", "take_sounds"): O,
    ("src/ui/configure.rs", "take_clipboard"): O,
    ("src/ui/configure.rs", "queue_feedback"): T,
    ("src/ui/configure.rs", "begin"): T,
    ("src/juice.rs", "apply_shakes"): S,
    ("src/juice.rs", "apply_bursts"): S,
    ("src/juice.rs", "cover_commands"): P,
    ("src/rich.rs", "rich_commands"): P,
    ("src/rich.rs", "vertical_rich_commands"): P,
    ("src/rich.rs", "rich_text"): F,
    ("src/ui/resize.rs", "resizer_commands"): P,
    ("src/vcursor.rs", "virtual_cursor_commands"): P,
    ("src/hud_layout.rs", "hud_edit_commands"): P,
    ("src/hud_layout.rs", "hud_layout_editor"): F,
    ("src/hud_layout.rs", "hud_apply"): L,
    ("src/hud_layout.rs", "hud_collect"): L,
    ("src/ui/text_controls.rs", "edit_marks"): P,
    ("src/ui/text_controls.rs", "text_area_commands"): P,
    ("src/ui/select.rs", "select_marks"): P,
    ("src/ui/select.rs", "mark_selection"): P,
    ("src/ui/visit.rs", "update_draw_order"): P,
    ("src/ui/finish.rs", "missing_glyphs"): P,
    ("src/ui/finish.rs", "relevel"): P,
    ("src/ui/finish.rs", "splice_levels"): P,
    ("src/ui/finish.rs", "note_relevance"): T,
    ("src/ui/finish.rs", "end"): T,
    ("src/ui/finish.rs", "end_with_layout_queries"): T,
    ("src/ui/finish.rs", "line_box"): L,
    ("src/ui/finish.rs", "prepare_layout_node"): L,
    ("src/ui/finish.rs", "apply_gutter"): L,
    ("src/ui/finish.rs", "touch_min"): T,
    ("src/ui/finish.rs", "content_visibility"): L,
    ("src/ui/finish.rs", "narrow_button"): P,
    ("src/element/frame.rs", "apply_variants"): S,
    ("src/element/frame.rs", "set_props"): F,
    ("src/element/frame.rs", "props_mut"): F,
    ("src/element/frame.rs", "open_with"): F,
    ("src/ui/commands.rs", "apply_world"): L,
    ("src/ui/commands.rs", "world_step"): L,
    ("src/ui/commands.rs", "apply_stagger"): S,
    ("src/ui/commands.rs", "plane_quads"): L,
    ("src/doc/build.rs", "apply_part_appearances"): S,
    ("src/doc/build.rs", "apply_part_appearance"): S,
    ("src/doc/build.rs", "resolve"): S,
    ("src/doc/build.rs", "family_declared"): S,
    ("src/world/frame.rs", "world_place"): L,
    ("src/world/frame.rs", "world_finish"): L,
    ("src/motion/run.rs", "note_drag_and_drop"): T,
    ("src/motion/run.rs", "record_matches"): T,
    ("src/motion/run.rs", "motion_dump_of"): O,
    ("src/layers.rs", "draw_covers"): P,
})

# end_frame (src/ui/finish.rs 247-1397) by region
END_FRAME = [
    (247, 284, T),   # close the build: retire, variants, appearances, resized
    (285, 438, T),   # derive child lists, gutters, visibility, reuse, sync_tree
    (439, 852, L),   # layout and settlement
    (853, 902, L),   # popover arrows
    (903, 910, L),   # orders, baselines, anchors
    (911, 955, S),   # sample, mirror, shakes, stagger, motion, bursts
    (956, 979, L),   # draw order, hud, maps, planes, world
    (980, 1202, T),  # interaction for the next tick
    (1203, 1263, P), # parts, decorations
    (1264, 1397, P), # commands, items, levels
]


def stage(file, fn, line, fn_start=None):
    if file == "src/ui/finish.rs" and fn == "end_frame":
        for a, b, s in END_FRAME:
            if a <= line <= b:
                return s
    if (file, fn) in FN:
        return FN[(file, fn)]
    if fn and re.search(r"(^|_)commands$", fn) and not file.startswith("src/doc/"):
        return P
    for pat, s in FILE:
        if re.search(pat, file):
            return s
    return "?"
