// ── ui: drawing over the frame ──────────────────────────────────────────────

/// Hands the UI's last frame to the render scene's overlay: screen-space
/// sprites and text in CSS pixels over the finished frame (after every post
/// effect, never tone mapped or blurred), at any camera.
/// `shifts`: how far each anchored element moves on a frame drawn between
/// ticks, by key (`crate::present::interpolate::ui_shifts`).
pub(crate) fn emit(rs: &mut super::extract::RenderState, shifts: &og_core::IdMap<u64, glam::Vec2>) {
    let Some(ui) = rs.ui.clone() else {
        return;
    };
    let ui = ui.lock();
    // The player's colour vision filter over the whole frame (docs/ui.md
    // §3.2), whether or not the UI draws anything this frame.
    rs.scene.look.color_filter = ui.theme().color_filter();
    let frame = ui.frame();
    let world_frame = ui.world_frame();
    if frame.items.is_empty() && world_frame.batches.is_empty() {
        return;
    }
    // The frame lays out in og-ui units and emits CSS px (docs/ui.md
    // §15.2): the overlay is the CSS surface, the unit times the frame's.
    let (w, h) = (
        frame.viewport.width * frame.unit,
        frame.viewport.height * frame.unit,
    );
    // Scale 0 fits the viewport's height to the target (as a capture at
    // another size needs).
    let overlay = rs
        .scene
        .ui
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(w.max(1.0), h.max(1.0), 0.0));
    let first_text = overlay.texts.len();
    let first_sprite = overlay.sprites.len();
    let first_shape = overlay.ui.shapes.len();
    frame.emit_shifted(
        &og_ui::EmitDesc {
            // Over the game's screen layers up to og_render::UI_LAYER, under
            // those above it (og_render::screen_layer).
            layer: og_render::UI_OVERLAY_LAYER,
            time: ui.seconds() as f32,
        },
        overlay,
        &|key| shifts.get(&key).copied(),
    );
    // Mass elements no element owns (docs/ui.md §1), under the UI.
    world_frame.emit(og_render::UI_OVERLAY_LAYER, overlay);
    crate::present::extract::draw_other_pages(overlay, first_text, &rs.ui_pages);
    // Icons on a strict art palette (super::art): the icon atlas is a
    // distance field the cook leaves as it is, so its tint is moved to
    // the palette here.
    let icons = ui.icons();
    let icon_textures: Vec<og_render::TextureHandle> =
        icons.layers().filter_map(|a| a.texture).collect();
    let icon_fonts: Vec<og_render::FontHandle> = icons
        .layers()
        .filter_map(|a| a.font.as_ref().and_then(|f| f.handle))
        .flat_map(|h| rs.ui_pages.get(&h).cloned().unwrap_or_else(|| vec![h]))
        .collect();
    if let Some(overlay) = rs.scene.ui.overlay.as_mut() {
        super::art::palette_icons(
            &rs.art.frame,
            overlay,
            (first_sprite, first_text, first_shape),
            (&icon_textures, &icon_fonts),
        );
    }
}
