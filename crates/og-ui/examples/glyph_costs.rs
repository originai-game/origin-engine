//! Run-time glyphs' cost for tests/evidence/P4/ui-text/NOTES.md: the UI
//! kit's Chinese font, each character of a sample rendered from its
//! outlines at 32 px/em, range 4 (the cook's geometry), per glyph.
//!
//!     cargo run --release -p og-ui --example glyph_costs

use og_ui::glyphs::render_glyph;
use std::time::Instant;

fn main() {
    let ttf = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansSC-Regular-subset.ttf"
    ))
    .expect("the kit's Chinese font");
    let sample = "欢迎回来旅人晨星おかえりなさい鬱龍警響議護顧驚體";
    let mut per = Vec::new();
    for c in sample.chars() {
        let t = Instant::now();
        let g = render_glyph(&ttf, c, 32.0, 4.0);
        let us = t.elapsed().as_secs_f64() * 1e6;
        if g.is_some() {
            per.push((c, us));
        }
    }
    let mean = per.iter().map(|p| p.1).sum::<f64>() / per.len() as f64;
    let max = per
        .iter()
        .cloned()
        .fold(('?', 0.0), |a, b| if b.1 > a.1 { b } else { a });
    println!(
        "{{\n  \"glyphs\": {},\n  \"meanUs\": {mean:.1},\n  \"maxUs\": {:.1},\n  \"maxChar\": \"{}\"\n}}",
        per.len(),
        max.1,
        max.0
    );
}
