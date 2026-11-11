//! The selection: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/user_select.rs.

use crate::support;
use support::*;

/// `references/<tier>/user-select-<width>.png` (`support::user_select_scene`,
/// which Chrome draws too: tests/web/reference-gfx.ts): a drag's selection
/// over a wrapped paragraph and into the next at 1280 and 390 wide.
#[test]
#[ignore = "requires GPU"]
fn the_selection_matches_its_reference_frames() {
    with_test_font(|files| {
        for name in ["user-select-1280", "user-select-390"] {
            let (scene, desc) = reference_scene(name, files).unwrap();
            let frame = render_frame(&scene, &desc);
            assert_eq!(frame.stats.missing_glyphs, 0);
            matches_reference(name, &frame.image, &frame.info, Against::OwnTier);
        }
    });
}
