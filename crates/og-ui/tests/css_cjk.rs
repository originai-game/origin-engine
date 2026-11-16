//! Ruby, vertical text and emphasis marks on document elements (CSS Ruby
//! 1, CSS Writing Modes 4 §5 and §9, CSS Text Decoration 4 §3), checked on
//! what the frame draws in Noto Sans SC (its own `vmtx` and CJK metrics).
//!
//! Ways these could fail, each covered below:
//! - an annotation set at the base's size, beside it, or on the wrong
//!   side; `ruby-align` spreading at the wrong opportunities; an
//!   annotation hanging further than one of its characters, over text
//!   under `ruby-overhang: none`; `<rp>` drawn; the line not growing for
//!   an annotation its leading cannot hold →
//!   `ruby_lays_out_as_css_ruby_does`
//! - upright or sideways swapped, `glyph-orientation-vertical` or an SVG
//!   1.1 writing mode not mapped, a combined run longer than an em, or
//!   `digits` combining a run longer than its count →
//!   `vertical_text_orients_and_combines_as_css_writing_modes_does`
//! - a mark on punctuation or a space, a mark of the wrong shape, size or
//!   side, or a line that does not make room for its marks →
//!   `emphasis_marks_sit_where_css_text_decoration_puts_them`

use crate::support::Doc;
use og_ui::*;

const SC: &str = "tests/fixtures/ui-text/assets/fonts/NotoSansSC-Regular-subset.ttf";
/// Noto Sans SC's hhea ascent and descent, of its 1000 units to the em.
const ASCENT: f32 = 1.160;
const DESCENT: f32 = 0.288;

/// A document of `body` under `css`, 400 px wide, its text 20 px Noto
/// Sans SC.
fn doc(body: &str, css: &str) -> Doc {
    let mut font = crate::features::font_at(SC);
    font.handle = crate::support::test_font().handle;
    let text = format!(
        "<meta name=\"component\" content=\"t\">\n<div><div id=\"Root\">{body}</div></div>\n<style>#Root {{ width: 400px; }} p {{ margin: 0; font-size: 20px; }} {css}</style>"
    );
    let d = Doc::with_font(&text, &[], font);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    d
}

/// A drawn glyph command: its text, pen, baseline, size, transform and ink.
#[derive(Clone, Debug)]
struct Glyphs {
    text: String,
    x: f32,
    baseline: f32,
    size: f32,
    transform: [f32; 4],
}

/// The glyph commands drawn inside element `id` (their line boxes'
/// middles inside it), in drawing order.
fn glyphs(d: &Doc, id: &str) -> Vec<Glyphs> {
    let r = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| {
            let (cmd, transform) = match &it.cmd {
                DrawCmd::StyledText { text, effects } => (&**text, effects.transform),
                c => (c, [1.0, 0.0, 0.0, 1.0]),
            };
            match cmd {
                DrawCmd::Text {
                    text,
                    x,
                    baseline,
                    size,
                    bounds,
                    ..
                } if (r.y..r.y + r.height).contains(&(bounds.y + bounds.height / 2.0))
                    && (r.x..r.x + r.width).contains(&(bounds.x + bounds.width / 2.0)) =>
                {
                    Some(Glyphs {
                        text: text.clone(),
                        x: *x,
                        baseline: *baseline,
                        size: *size,
                        transform,
                    })
                }
                _ => None,
            }
        })
        .collect()
}

fn one(d: &Doc, id: &str, text: &str) -> Glyphs {
    let all = glyphs(d, id);
    all.iter()
        .find(|g| g.text == text)
        .cloned()
        .unwrap_or_else(|| panic!("{text} in {id}: {all:?}"))
}

/// A drawn shape: centre, half extent, transform, filled, ink.
type Mark = ([f32; 2], [f32; 2], [f32; 4], bool, [f32; 4]);

/// The shapes drawn inside element `id`.
fn marks(d: &Doc, id: &str) -> Vec<Mark> {
    let r = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if (r.y - 0.5..r.y + r.height + 0.5).contains(&shape.center.y)
                    && (r.x - 0.5..r.x + r.width + 0.5).contains(&shape.center.x) =>
            {
                let og_draw::UiPaint::Solid(ink) = shape.paint else {
                    return None;
                };
                Some((
                    [shape.center.x, shape.center.y],
                    [shape.half.x, shape.half.y],
                    shape.transform,
                    shape.mode == og_draw::UiMode::Fill,
                    ink,
                ))
            }
            _ => None,
        })
        .collect()
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.1
}

const STRAIGHT: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
/// A sideways run: turned a quarter clockwise.
const TURNED: [f32; 4] = [0.0, 1.0, -1.0, 0.0];

/// CSS Ruby 1: an annotation (`<rt>`) at half its base's size (HTML's
/// UA sheet) sits on its base's over edge, or under edge with
/// `ruby-position: under` (§4.1); base and annotation share a column as
/// long as the longer, `space-around` (the initial value) sharing the
/// shorter's free room among its CJK justification opportunities with
/// half a share at each end, `start` at the start (§4.3); a longer
/// annotation hangs by half its excess (at most one of its characters)
/// over the text beside it under `ruby-overhang: auto`, by none under
/// `none` (§5.1); `<rp>` is not shown; a line too tight for its
/// annotation grows (§3.6). Down a vertical line the annotation is on the
/// right (line-over).
#[test]
fn ruby_lays_out_as_css_ruby_does() {
    let d = doc(
        r#"<p id="Plain">あ日本です</p>
<p id="Around">あ<ruby>日本<rp>(</rp><rt>にほん</rt><rp>)</rp></ruby>です</p>
<p id="Under">あ<ruby>漢<rt>かんじ</rt></ruby>い</p>
<p id="None">あ<ruby>漢<rt>かんじ</rt></ruby>い</p>
<p id="Start">あ<ruby>日本<rt>に</rt></ruby>い</p>
<p id="Down">あ<ruby>日本<rt>にほん</rt></ruby>い</p>
<p id="Two">あ<ruby>東京<rt>とう</rt><rtc>ひが</rtc></ruby>い</p>
<p id="Cols">あ<ruby><rb>東</rb><rb>京</rb><rt>とう</rt><rt>きょう</rt></ruby>い</p>
<p id="Inter">あ<ruby>中<rt>かん</rt></ruby>い</p>
<p id="Nest">あ<ruby><ruby>東<rt>とう</rt></ruby><rt>ひがし</rt></ruby>い</p>
<p id="Css">あ<span part="r">日<span part="t">にち</span><span part="p">(</span></span>い</p>
<p id="Down2">あ<ruby>東<rt>とう</rt><rtc>ひが</rtc></ruby>い</p>"#,
        "#Under { ruby-position: under; } #None { ruby-overhang: none; } #Start { ruby-align: start; }
#Down, #Down2 { writing-mode: vertical-rl; height: 200px; } #Inter { ruby-position: inter-character; }
#Css::part(r) { display: ruby; } #Css::part(t) { display: ruby-text; } #Css::part(p) { display: none; }",
    );
    // Over the base at half its size, inside the line, which grew for it.
    let base = one(&d, "Around", "日");
    let ni = one(&d, "Around", "に");
    let around = d.get("Around");
    assert_eq!((base.size, ni.size), (20.0, 10.0));
    // Its em box's middle half an annotation em over the base's ascent
    // (each baseline on the pixel grid).
    let middle = ni.baseline - (ASCENT - DESCENT) / 2.0 * 10.0;
    assert!(
        (middle - (base.baseline - ASCENT * 20.0 - 5.0)).abs() <= 1.0
            && middle - 5.0 >= around.y - 0.5,
        "{ni:?} {base:?} {around:?}"
    );
    assert!(around.height > d.get("Plain").height, "{around:?}");
    assert!(
        glyphs(&d, "Around")
            .iter()
            .all(|g| !g.text.contains(['(', ')']))
    );
    // 日本 (40 px) under にほん (30 px): 10 px free over two opportunities
    // and two half shares, so a share is 10/3.
    let share = 10.0 / 3.0;
    let xs: Vec<f32> = ["に", "ほ", "ん"]
        .iter()
        .map(|t| one(&d, "Around", t).x)
        .collect();
    let want = [
        20.0 + share / 2.0,
        20.0 + share * 1.5 + 10.0,
        20.0 + share * 2.5 + 20.0,
    ];
    assert!(
        xs.iter().zip(want).all(|(a, b)| close(*a, b)),
        "{xs:?} {want:?}"
    );
    assert!(close(one(&d, "Around", "です").x, 60.0));
    // Under; かんじ (30 px) over 漢 (20 px) hangs 5 px on each side, so
    // い follows 漢 directly.
    let (kan, ka) = (one(&d, "Under", "漢"), one(&d, "Under", "か"));
    assert!(
        ka.baseline > kan.baseline + DESCENT * 20.0,
        "{ka:?} {kan:?}"
    );
    assert!(close(kan.x, 20.0) && close(ka.x, 15.0), "{kan:?} {ka:?}");
    assert!(close(one(&d, "Under", "い").x, 40.0));
    // ruby-overhang: none: the ruby takes the annotation's 30 px.
    assert!(close(one(&d, "None", "漢").x, 25.0) && close(one(&d, "None", "か").x, 20.0));
    assert!(close(one(&d, "None", "い").x, 50.0));
    // ruby-align: start.
    assert!(close(one(&d, "Start", "に").x, one(&d, "Start", "日").x));
    // Down a vertical-rl line: beside the base on its right, the two
    // columns' middles half the base's em and half the annotation's apart.
    let (b, a) = (one(&d, "Down", "日"), one(&d, "Down", "に"));
    let mid = |g: &Glyphs| g.x + g.size / 2.0;
    assert!(close(mid(&a) - mid(&b), 15.0), "{a:?} {b:?}");
    assert!(a.transform == STRAIGHT && a.size == 10.0);
    // に's em starts the column's share in: 日本 runs from 20 to 60 down it.
    let top = d.get("Down").y;
    assert!(
        a.baseline > top + 20.0 && a.baseline < top + 20.0 + 10.0 + share,
        "{a:?}"
    );
    // Two levels, alternate (the initial value): the <rt> over, the
    // <rtc> after it under (§4.1); the <rtc>'s words span the base.
    let (to, tou, hi) = (
        one(&d, "Two", "東"),
        one(&d, "Two", "と"),
        one(&d, "Two", "ひ"),
    );
    // An annotation's em box middle, half its em past the base's edge.
    let em_mid = |g: &Glyphs| g.baseline - (ASCENT - DESCENT) / 2.0 * g.size;
    let over_edge =
        |a: &Glyphs, b: &Glyphs| (em_mid(a) - (b.baseline - ASCENT * 20.0 - 5.0)).abs() <= 1.0;
    assert!(
        over_edge(&tou, &to) && (em_mid(&hi) - (to.baseline + DESCENT * 20.0 + 5.0)).abs() <= 1.0,
        "{tou:?} {to:?} {hi:?}"
    );
    // <rb> columns paired one by one with the <rt>s (§2.3.2): きょう (30
    // px) widens 京's column, centred over it, hanging 5 px after it.
    let (kyo, ki) = (one(&d, "Cols", "京"), one(&d, "Cols", "き"));
    assert!(
        close(one(&d, "Cols", "と").x, 20.0) && close(kyo.x, 45.0) && close(ki.x, 40.0),
        "{kyo:?} {ki:?}"
    );
    assert!(close(one(&d, "Cols", "い").x, 65.0));
    // inter-character: upright right of the base in a column of its own,
    // half an em wide, its characters stacked about the base's middle.
    let (chu, ka, n) = (
        one(&d, "Inter", "中"),
        one(&d, "Inter", "か"),
        one(&d, "Inter", "ん"),
    );
    assert!(
        close(ka.x, 40.0)
            && ka.size == 10.0
            && ka.transform == STRAIGHT
            && (n.baseline - ka.baseline - 10.0).abs() <= 1.0
            && ((ka.baseline + n.baseline) / 2.0 - chu.baseline).abs() < 20.0,
        "{chu:?} {ka:?} {n:?}"
    );
    assert!(close(one(&d, "Inter", "い").x, 50.0));
    // Nested ruby: the outer annotation over the inner one (§3.1.2).
    let (inner, outer) = (one(&d, "Nest", "と"), one(&d, "Nest", "ひ"));
    let nest = one(&d, "Nest", "東");
    assert!(
        over_edge(&inner, &nest) && (inner.baseline - outer.baseline - 10.0).abs() <= 1.0,
        "{inner:?} {outer:?}"
    );
    // display: ruby and ruby-text on a text's ::part spans, none hiding one.
    let (nichi, ni) = (one(&d, "Css", "日"), one(&d, "Css", "に"));
    assert!(
        ni.size == 10.0 && over_edge(&ni, &nichi),
        "{nichi:?} {ni:?}"
    );
    assert!(glyphs(&d, "Css").iter().all(|g| !g.text.contains('(')));
    // Down a column, the second level on its left (line-under).
    let (b, over, under) = (
        one(&d, "Down2", "東"),
        one(&d, "Down2", "と"),
        one(&d, "Down2", "ひ"),
    );
    assert!(
        close(mid(&over) - mid(&b), 15.0) && close(mid(&b) - mid(&under), 15.0),
        "{b:?} {over:?} {under:?}"
    );
}

/// CSS Writing Modes 4: `text-orientation: upright` stands Latin up an
/// em apart, `sideways` turns even CJK a quarter clockwise (§5.1); SVG
/// 1.1's `glyph-orientation-vertical` is `text-orientation` (0deg
/// upright, 90deg sideways, §5.1.3) and `tb`/`lr-tb` are `vertical-rl`
/// and `horizontal-tb` (§3.2.1); `text-combine-upright` sets a run side
/// by side in one em, squeezed to fit (§9.1.2): under `all` the whole
/// text, under `digits` each run of at most its count of digits.
#[test]
fn vertical_text_orients_and_combines_as_css_writing_modes_does() {
    let d = doc(
        r#"<p id="Up">AB</p><p id="Side">日A</p><p id="G0">AB</p><p id="G90">日A</p>
<p id="Tb">AB</p><p id="Lr">AB</p><p id="All">123</p><p id="Dig">年10月100</p>
<p id="Span">年<span part="y">AB</span>月</p>"#,
        "#Up, #Side, #G0, #G90, #All, #Dig, #Span { writing-mode: vertical-rl; height: 100px; }
#Span::part(y) { text-combine-upright: all; }
#Up { text-orientation: upright; } #Side { text-orientation: sideways; }
#G0 { glyph-orientation-vertical: 0deg; } #G90 { glyph-orientation-vertical: 90deg; }
#Tb { writing-mode: tb; height: 100px; } #Lr { writing-mode: lr-tb; }
#All { text-combine-upright: all; } #Dig { text-combine-upright: digits 2; }",
    );
    for id in ["Up", "G0"] {
        let (a, b) = (one(&d, id, "A"), one(&d, id, "B"));
        assert!(
            a.transform == STRAIGHT
                && b.transform == STRAIGHT
                && close(b.baseline - a.baseline, 20.0),
            "{id}: {a:?} {b:?}"
        );
    }
    for id in ["Side", "G90"] {
        let g = glyphs(&d, id);
        assert!(
            g.len() == 1 && g[0].text == "日A" && g[0].transform == TURNED,
            "{id}: {g:?}"
        );
    }
    // tb lays out as vertical-rl (one column a line wide, its Latin
    // sideways); lr-tb as horizontal-tb.
    let tb = d.get("Tb");
    assert!(tb.height == 100.0 && tb.width < 40.0, "{tb:?}");
    assert_eq!(one(&d, "Tb", "AB").transform, TURNED);
    let lr = d.get("Lr");
    assert!(lr.width == 400.0 && one(&d, "Lr", "AB").transform == STRAIGHT);
    // all: 123 is wider than an em, so squeezed across into one.
    let all = glyphs(&d, "All");
    assert!(all.len() == 1 && all[0].text == "123", "{all:?}");
    let [sx, 0.0, 0.0, 1.0] = all[0].transform else {
        panic!("{all:?}")
    };
    assert!(sx > 0.0 && sx < 1.0);
    assert!(d.get("All").height == 100.0);
    // digits 2: 10 takes one em between 年 and 月; 100 is three digits and
    // lies sideways.
    let (nen, ten, tsuki) = (
        one(&d, "Dig", "年"),
        one(&d, "Dig", "10"),
        one(&d, "Dig", "月"),
    );
    assert!(
        close(ten.baseline - nen.baseline, 20.0) && close(tsuki.baseline - ten.baseline, 20.0),
        "{nen:?} {ten:?} {tsuki:?}"
    );
    assert!(ten.transform[1..] == [0.0, 0.0, 1.0] && ten.transform[0] <= 1.0);
    assert_eq!(one(&d, "Dig", "100").transform, TURNED);
    // all on an inline span: its AB upright in one em between 年 and 月,
    // squeezed across.
    let (nen, ab, tsuki) = (
        one(&d, "Span", "年"),
        one(&d, "Span", "AB"),
        one(&d, "Span", "月"),
    );
    assert!(
        close(ab.baseline - nen.baseline, 20.0) && close(tsuki.baseline - ab.baseline, 20.0),
        "{nen:?} {ab:?} {tsuki:?}"
    );
    assert!(
        ab.transform[1..] == [0.0, 0.0, 1.0] && ab.transform[0] < 1.0,
        "{ab:?}"
    );
}

/// CSS Text Decoration 4 §3: a mark for each character but punctuation
/// and separators, centred on it at half its size (§3.1), over its
/// content area across a line or under it with `under`, right of its
/// column down one or left with `left` (§3.4); `filled` alone is a circle
/// across and a sesame down, `open` strokes it; a string draws its
/// character; `text-emphasis-color` inks it, `currentcolor` by default.
/// The line grows for marks its leading cannot hold, as for ruby.
#[test]
fn emphasis_marks_sit_where_css_text_decoration_puts_them() {
    let d = doc(
        r#"<p id="Plain">日本、年 月</p><p id="Dot">日本、年 月</p><p id="Under">日本</p>
<p id="Open">日</p><p id="Str">日本</p><p id="Down">日本</p><p id="Left">日</p>
<div lang="zh-Hans"><p id="Zh">日本</p><p id="Ja" lang="ja">日本</p><p id="ZhDown">日</p></div>"#,
        "#Dot { text-emphasis: dot #ff0000; } #Under { text-emphasis: filled; text-emphasis-position: under; color: #0000ff; }
#Open { text-emphasis: open; } #Str { text-emphasis: \"*\"; }
#Down, #Left, #ZhDown { writing-mode: vertical-rl; height: 100px; text-emphasis: filled; } #Left { text-emphasis-position: over left; }
#Zh, #Ja { text-emphasis: filled; }",
    );
    // Not on 、 or the space: four dots, centred on 日, 本, 年 and 月,
    // 0.15 of the mark's 10 px em across, red, 5 px over the content area.
    let dots = marks(&d, "Dot");
    let base = one(&d, "Dot", "日本、年 月").baseline;
    let at = |c: &str| {
        let i = "日本、年 月".find(c).expect("char");
        let before = &"日本、年 月"[..i];
        // CJK one em each, the space Noto Sans SC's 0.224 em.
        before
            .chars()
            .map(|c| if c == ' ' { 0.224 * 20.0 } else { 20.0 })
            .sum::<f32>()
            + 10.0
    };
    let want: Vec<f32> = ["日", "本", "年", "月"].iter().map(|c| at(c)).collect();
    assert_eq!(dots.len(), 4, "{dots:?}");
    for (m, x) in dots.iter().zip(want) {
        assert!(close(m.0[0], x), "{dots:?}");
        // The text's baseline is drawn on the pixel grid.
        assert!(
            (m.0[1] - (base - ASCENT * 20.0 - 5.0)).abs() <= 0.5,
            "{m:?} {base}"
        );
        assert!(m.1 == [1.5, 1.5] && m.3 && crate::support::near(m.4, Srgba::rgb(255, 0, 0)));
    }
    assert!(d.get("Dot").height > d.get("Plain").height);
    // Under, in the text's colour: a filled circle under its descent.
    let under = marks(&d, "Under");
    let ub = one(&d, "Under", "日本").baseline;
    assert!(
        under.len() == 2
            && under
                .iter()
                .all(|m| (m.0[1] - (ub + DESCENT * 20.0 + 5.0)).abs() <= 0.5
                    && m.1 == [3.5, 3.5]
                    && crate::support::near(m.4, Srgba::rgb(0, 0, 255))),
        "{under:?}"
    );
    // open: stroked.
    assert!(marks(&d, "Open").iter().all(|m| !m.3));
    // A string: its character drawn at 10 px, centred on each.
    let stars: Vec<Glyphs> = glyphs(&d, "Str")
        .into_iter()
        .filter(|g| g.text == "*")
        .collect();
    assert!(
        stars.len() == 2 && stars.iter().all(|g| g.size == 10.0),
        "{stars:?}"
    );
    // Down a column: sesames right of it (left with `left`), one per
    // character, a quarter em past the glyph's half em.
    for (id, side) in [("Down", 1.0), ("Left", -1.0)] {
        let g = one(&d, id, "日");
        let mid = g.x + 10.0;
        let m = marks(&d, id);
        let count = if id == "Down" { 2 } else { 1 };
        assert_eq!(m.len(), count, "{id}: {m:?}");
        assert!(
            m.iter()
                .all(|m| close(m.0[0], mid + side * 15.0) && m.1[1] > m.1[0] && m.2 != STRAIGHT),
            "{id}: {m:?} {g:?}"
        );
    }
    // The position from the content language (§3.4's UA sheet): under
    // right in Chinese, over right in Japanese inside it.
    let zb = one(&d, "Zh", "日本").baseline;
    let zh = marks(&d, "Zh");
    assert!(
        zh.len() == 2 && zh.iter().all(|m| m.0[1] > zb),
        "{zh:?} {zb}"
    );
    let jb = one(&d, "Ja", "日本").baseline;
    let ja = marks(&d, "Ja");
    assert!(
        ja.len() == 2 && ja.iter().all(|m| m.0[1] < jb - ASCENT * 20.0),
        "{ja:?} {jb}"
    );
    let g = one(&d, "ZhDown", "日");
    let m = marks(&d, "ZhDown");
    assert!(
        m.len() == 1 && close(m[0].0[0], g.x + 10.0 + 15.0),
        "{m:?} {g:?}"
    );
}
