//! Shaping's cost for tests/evidence/E6/i18n-costs.json: an Arabic label
//! of the UI kit shaped and put in visual order with the kit's Noto Sans
//! Arabic subset, and a Latin label (no shaping: passed through), per call.
//!
//!     cargo run --release -p og-ui --example shape_costs > tests/evidence/E6/i18n-costs.json

use og_i18n::Direction;
use og_ui::shape::{Shaper, visual};
use std::sync::Arc;
use std::time::Instant;

fn per_call_us(n: u32, f: impl Fn() -> String) -> f64 {
    let t = Instant::now();
    let mut len = 0;
    for _ in 0..n {
        len += f().len();
    }
    assert!(len > 0);
    t.elapsed().as_secs_f64() * 1e6 / f64::from(n)
}

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf");
    let t = Instant::now();
    let sh = Shaper::new(Arc::new(std::fs::read(path).expect("the kit's Arabic font"))).expect("rustybuzz reads it");
    let load_ms = t.elapsed().as_secs_f64() * 1000.0;
    let all = |_c: char| true;
    let arabic = "هل تريد حذف هذا الحفظ؟ لا يمكن التراجع";
    let mixed = "المستوى 1-2 الطاحونة القديمة";
    let latin = "Delete this save? This cannot be undone";
    let ar = per_call_us(2000, || visual(arabic, Direction::Rtl, Some(&sh), &all));
    let mx = per_call_us(2000, || visual(mixed, Direction::Rtl, Some(&sh), &all));
    let la = per_call_us(20000, || visual(latin, Direction::Ltr, Some(&sh), &all));
    println!(
        "{{\n  \"shaperLoadMs\": {load_ms:.3},\n  \"arabicLabelUs\": {ar:.2},\n  \"arabicWithDigitsUs\": {mx:.2},\n  \"latinLabelUs\": {la:.3},\n  \"arabicLabelChars\": {}\n}}",
        arabic.chars().count()
    );
}
