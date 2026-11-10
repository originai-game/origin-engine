"""Proposed owning state of every field of Ui, State and Node."""

def group(spec):
    out = {}
    for owner, names in spec.items():
        for n in names.split():
            assert n not in out, n
            out[n] = owner
    return out

UI = group({
    "Env": "configured theme fonts ticks_per_second viewport unit direction locale icons cue_icons images style_name haptics hud_max_aspect file_picker toast_anchor",
    "Tree": "nodes kinds prev_kinds render_slots free_slots emitted retiring slot_present persistent owning in_place built anonymous stack floating last_closed tree orders semantics building scopes draws dressable prev_floating",
    "Interaction": "input native_input state nav nav_on players vcursor request_focus gesture resize_moved text_target selected drop_paths drag_paths drop_accepts actions",
    "Style": "part_rules part_sheet root_tokens root_type_tokens part_commits css_sounds timelines pending_plays",
    "Layout": "texts kept layout_mutated",
    "World": "world",
    "Paint": "kept_spans part_draws part_changed decor_draws drawing prev_draws inspection",
    "Docs": "docs reactive",
    "Out": "frame warnings glyphs_missed events feedback haptic_out clipboard stats",
    "Front": "touched kept_texts button_ink accent hud_editor_key cover",
})

STATE = group({
    "Interaction": "hot active focus caret anchor edit_key edit_tick composition edit_scroll caret_upstream kill killing typed_stream click selection select_press histories hover_ticks pointer focus_click scroll virtual_lists scroll_x scroll_info fling wheel_from pinch drag_v thumb resizing resized hits focus_visible scope scope_locked focus_returns nav_off stick typeahead pending players pad_focus pad_stick groups nav_links recognizers tap vcursor hud",
    "Env": "tick",
    "Style": "view_transition anims clocks motion",
    "Docs": "rive",
    "Front": "toasts next_toast touch_owners",
    "Out": "voices feedback_rng feedback_last",
})

NODE = group({
    "Tree": "key id parent children built_children ordered_children out up floats hidden content_slot part_source",
    "Authored": "el props content widget part_host part_box part_geometry probe_text disabled focusable multiline anchor float shape_outside fx nav adjust claim backdrop scope locked shapes decor aux heading a11y stagger accent caret anchor_point arrow",
    "Style": "sampled flip gesture custom",
    "Layout": "gutter authored_box baseline_items table line_box min_content grid fit pref_w min_w pref_h min_h w h x y content_h margin_sets collapse_in through margin_x content_w lines rich_lines links rows layout_size layout_offset layout_step laid_xy laid_rect laid_clip anchor_placed cols col_rules frag stuck cv_skip cv_far remembered float_at clearance wrap_lines float_h beside inline_rows inline_y",
    "Placed": "world screen plane alpha scale inert clip_src clip rect",
    "Interaction": "ring pad_rings selected",
})

# which stage of the pipeline each state belongs to
STAGE_OF = {
    "Env": "env", "Tree": "tree", "Interaction": "tree", "Style": "style", "Layout": "layout",
    "Paint": "paint", "Docs": "docs", "Out": "out", "Front": "front", "World": "tree+layout",
    "Authored": "tree", "Placed": "layout",
}
