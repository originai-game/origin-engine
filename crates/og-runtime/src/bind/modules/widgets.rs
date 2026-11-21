// ── ui: drawing over the frame ──────────────────────────────────────────────

/// Hands the UI's last paint to the render scene: the new core's arena,
/// drawn by the paint pass at the UI's layer (og-render ui/paint.rs), taken
/// when it is newer, each font slot given its font's pages.
pub(crate) fn emit(rs: &mut super::extract::RenderState) {
    // The player's colour vision filter over the whole frame (docs/ui.md
    // §3.2), whether or not the UI draws anything this frame.
    rs.scene.look.color_filter = rs.color_filter;
    if let Some(next) = rs.ui_next.as_mut()
        && next.paint.generation > rs.scene.ui.paint.generation
    {
        std::mem::swap(&mut next.paint, &mut rs.scene.ui.paint);
    }
    for slot in &mut rs.scene.ui.paint.fonts {
        if let Some(pages) = slot.first().and_then(|h| rs.ui_pages.get(h))
            && slot != pages
        {
            slot.clone_from(pages);
        }
    }
}
