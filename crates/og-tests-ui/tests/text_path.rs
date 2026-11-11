//! Text on a path: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/text_path.rs.

use crate::support;
use support::*;

/// `references/<tier>/text-path-<width>.png` (`support::text_path_scene`,
/// drawn in Chrome too by tests/web/reference-gfx.ts): a title on an arch,
/// a badge's ring of text and a wave at 1280 and 390 wide.
#[test]
#[ignore = "requires GPU"]
fn text_on_a_path_matches_its_reference_frames() {
    with_test_font(|files| {
        for name in ["text-path-1280", "text-path-390"] {
            let (scene, desc) = reference_scene(name, files).unwrap();
            let frame = render_frame(&scene, &desc);
            assert_eq!(frame.stats.missing_glyphs, 0);
            matches_reference(name, &frame.image, &frame.info, Against::OwnTier);
        }
    });
}
