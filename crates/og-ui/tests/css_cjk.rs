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
use og_core::glam::Vec2;
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
    color: [f32; 4],
    /// Where its first glyph is drawn: its pen and its placement.
    ink: f32,
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
                    color,
                    placements,
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
                        color: *color,
                        ink: *x + placements.first().map_or(0.0, |p| p.x * *size),
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
/// A text's inline elements are elements (CSS 2 §9.2.2): class, id,
/// attribute, structural, child and state selectors match them, their CSS
/// styles their words, `display` gives any of them a ruby role, and a
/// text's `::part(name)` still styles the element naming its part.
///
/// Ways it could fail: a rule reaching only plain `<b>` (the tag's UA
/// style) and not a classed or attributed element; `:last-of-type`
/// counted among the paragraph's blocks rather than its inline siblings;
/// a hover that restyles nothing until something else rebuilds the text;
/// ruby only from `<ruby>` tags; a part rule lost under the element's.
#[test]
fn inline_elements_are_matched_and_styled_as_css_does() {
    let mut d = doc(
        r#"<p id="P">あ<span class="gold" id="G">日</span><b data-k="1">本</b><b>で</b><a id="L" href="r">す</a><span part="big">漢</span><span style="display: ruby">東<span style="display: ruby-text">とう</span></span></p>"#,
        ".gold { color: #ff0000; } #G { font-size: 30px; } [data-k] { color: #00ff00; }
b:last-of-type { color: #0000ff; } p > a { color: #ff00ff; } a:hover { color: #00ffff; }
p::part(big) { font-size: 40px; }",
    );
    let ink = |d: &Doc, t: &str| {
        let c = one(d, "P", t).color;
        [c[0] > 0.5, c[1] > 0.5, c[2] > 0.5]
    };
    assert_eq!(d.get("P").text, "あ日本です漢東");
    assert_eq!(one(&d, "P", "あ").size, 20.0);
    assert_eq!(one(&d, "P", "日").size, 30.0);
    assert_eq!(ink(&d, "日"), [true, false, false]);
    assert_eq!(ink(&d, "本"), [false, true, false]);
    assert_eq!(ink(&d, "で"), [false, false, true]);
    assert_eq!(ink(&d, "す"), [true, false, true]);
    assert_eq!(one(&d, "P", "漢").size, 40.0);
    let (base, top) = (one(&d, "P", "東"), one(&d, "P", "と"));
    assert_eq!((base.size, top.size), (20.0, 10.0));
    assert!(top.baseline < base.baseline - 10.0, "{top:?} {base:?}");
    let link = format!("{}/L", d.get("P").path);
    d.ui.force_state(&link, "interaction", Some("hover"));
    d.step(InputFrame::default());
    d.step(InputFrame::default());
    assert_eq!(ink(&d, "す"), [false, true, true]);
    assert_eq!(ink(&d, "で"), [false, false, true]);
}

/// An inline element is one fragment per line it has words on (CSS 2
/// §9.4.2): its box their union, a link hit on each fragment and not
/// across the lines between, and the pointer over a link hovering its
/// `<a>` (`a:hover`).
///
/// Ways it could fail: one hit over the union, so a press on the words
/// before a wrapped link follows it; the element's box its text's, or one
/// line's; a link's hover kept on the text's link id and never reaching
/// the `<a>`'s rules.
#[test]
fn inline_elements_fragment_on_their_lines_and_links_hover_as_their_a() {
    let mut d = doc(
        r#"<p id="P" style="width: 100px">あい<a id="L" href="r">日本東京中漢</a>です</p>"#,
        "a { color: #ff0000; } a:hover { color: #00ff00; }",
    );
    let p = d.get("P");
    // あい日本東 | 京中漢です: the link from 40 to 100 on the first line
    // and from 0 to 60 on the second.
    let (ri, kyo) = (one(&d, "P", "日本東"), one(&d, "P", "京中漢"));
    assert!(
        close(ri.x - p.x, 40.0) && close(kyo.x - p.x, 0.0),
        "{ri:?} {kyo:?}"
    );
    assert!(kyo.baseline > ri.baseline + 10.0);
    let l = d.get("L");
    assert!(
        close(l.x, p.x)
            && close(l.width, 100.0)
            && l.y >= p.y
            && l.y + l.height <= p.y + p.height + 0.5,
        "the union of its two fragments: {l:?} in {p:?}"
    );
    let first = Vec2::new(p.x + 10.0, ri.baseline - 8.0);
    let second = Vec2::new(p.x + 10.0, kyo.baseline - 8.0);
    let events = d.click_at(first);
    assert!(!events.iter().any(|e| e.kind == "link"), "{events:?}");
    let events = d.click_at(second);
    assert!(
        events.iter().any(|e| e.kind == "link" && e.name == "r"),
        "{events:?}"
    );
    let green = |d: &Doc| {
        let c = one(d, "P", "日本東").color;
        c[1] > 0.5 && c[0] < 0.5
    };
    d.step(InputFrame {
        pointer: Vec2::new(p.x + 50.0, p.y + p.height + 100.0),
        pointer_inside: true,
        ..Default::default()
    });
    d.step(InputFrame::default());
    assert!(!green(&d));
    for _ in 0..3 {
        d.step(InputFrame {
            pointer: second,
            pointer_inside: true,
            ..Default::default()
        });
    }
    assert!(green(&d), "{:?}", one(&d, "P", "日本東"));
}

/// An inline element's margin, border and padding (CSS 2 §8.1, §10.3.1,
/// §10.6.1): at its start and end they take room on the line, before its
/// first character and after its last, only on the lines holding them
/// (`box-decoration-break: slice`); across the line they grow its
/// fragments, its border boxes, and not the line. Down a vertical line its
/// start is its top.
///
/// Ways it could fail: room drawn after the first character rather than
/// before it; the start's room on every line a wrapped element is on, or
/// none past a break; the margin inside its border box; a fragment as
/// high as its content area, or a line grown by its padding; a vertical
/// line's room across it.
#[test]
fn inline_edges_take_room_at_their_start_and_end() {
    let d = doc(
        r#"<p id="Plain">あい<u>日本</u>です</p><p id="P">あい<span id="S">日本</span>です</p>
<p id="W" style="width: 100px">あい<span id="T">日本東京中漢</span>です</p>
<p id="V">あ<b id="VS">日</b>本</p>
<p id="C" style="width: 100px">あい<span id="CS" class="c">日本東京中漢</span>です</p>"#,
        "span { margin: 0 4px 0 3px; border-left: 2px solid #f00; padding: 5px 8px 7px 6px; }
#V { writing-mode: vertical-rl; height: 200px; }
b { font-weight: normal; margin: 3px 0 4px; padding: 6px 2px 8px 3px; }
u { text-decoration: none; color: #f00; } .c { box-decoration-break: clone; }",
    );
    // Start: margin 3, border 2, padding 6; end: padding 8, margin 4.
    let p = d.get("P");
    // Its first and last characters are shaped apart (CSS Text 3 §7.3).
    let (ri, hon, de) = (one(&d, "P", "日"), one(&d, "P", "本"), one(&d, "P", "です"));
    assert!(
        close(ri.ink - p.x, 40.0 + 11.0)
            && close(hon.ink - p.x, 40.0 + 11.0 + 20.0)
            && close(de.x - p.x, 40.0 + 40.0 + 23.0),
        "{ri:?} {hon:?} {de:?}"
    );
    let s = d.get("S");
    let top = ri.baseline - ASCENT * 20.0 - 5.0;
    let bottom = ri.baseline + DESCENT * 20.0 + 7.0;
    assert!(
        close(s.x - p.x, 43.0)
            && close(s.x + s.width - p.x, 99.0)
            && close(s.y, top)
            && close(s.y + s.height, bottom),
        "{s:?}: 43 to 99, {top} to {bottom}"
    );
    // Its line is the plain one (the boxes' heights snapped to pixels).
    let plain = d.get("Plain").height;
    assert!((p.height - plain).abs() <= 1.0, "{p:?} {plain}");
    // あい + 11 + 日本 | 東京中漢 + 12 | です: the start's room on the first
    // line only, the end's on the second.
    let w = d.get("W");
    let (ri, to, de) = (
        one(&d, "W", "日"),
        one(&d, "W", "東京中"),
        one(&d, "W", "です"),
    );
    assert!(
        close(ri.ink - w.x, 51.0) && close(to.ink - w.x, 0.0) && close(de.x - w.x, 0.0),
        "{ri:?} {to:?} {de:?}"
    );
    assert!(de.baseline > to.baseline + 10.0 && to.baseline > ri.baseline + 10.0);
    let t = d.get("T");
    assert!(
        close(t.x - w.x, 0.0) && close(t.x + t.width - w.x, 91.0),
        "the union of 43..91 and 0..88: {t:?}"
    );
    // clone: each line holds both edges, 11 before and 12 after: あい +
    // 11 + 日 + 12 | 11 + 本東京 + 12 | 11 + 中漢 + 12 + で | す.
    let c = d.get("C");
    let at = |t: &str| {
        let g = one(&d, "C", t);
        (g.ink - c.x, g.baseline)
    };
    let (ni, hon, chu, de) = (at("日"), at("本東京"), at("中"), at("で"));
    assert!(
        close(ni.0, 51.0) && close(hon.0, 11.0) && close(chu.0, 11.0) && close(de.0, 63.0),
        "{ni:?} {hon:?} {chu:?} {de:?}"
    );
    assert!(hon.1 > ni.1 + 10.0 && chu.1 > hon.1 + 10.0 && de.1 == chu.1);
    // Room is drawn as nothing.
    assert!(glyphs(&d, "C").iter().all(|g| !g.text.is_empty()));
    let cs = d.get("CS");
    assert!(
        close(cs.x - c.x, 3.0) && close(cs.x + cs.width - c.x, 79.0),
        "each fragment inside its margins: {cs:?}"
    );
    // Down the line: start 3 + 6 above 日, end 8 + 4 below it.
    let (a, ni, hon) = (one(&d, "V", "あ"), one(&d, "V", "日"), one(&d, "V", "本"));
    assert!(
        close(ni.baseline - a.baseline, 29.0) && close(hon.baseline - ni.baseline, 32.0),
        "{a:?} {ni:?} {hon:?}"
    );
    let (v, vs) = (d.get("V"), d.get("VS"));
    assert!(
        close(vs.y - v.y, 23.0) && close(vs.height, 6.0 + 20.0 + 8.0),
        "{vs:?} in {v:?}"
    );
    assert!(close(vs.width, 20.0 + 3.0 + 2.0), "{vs:?}");
}

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
<div lang="zh-Hans"><p id="Zh">日本</p><p id="Ja" lang="ja">日本</p><p id="ZhDown">日</p></div>
<p id="Rb">あ<ruby>日本<rt>にほん</rt></ruby></p><p id="RbDown">日<ruby>本<rt>ほん</rt></ruby></p>"#,
        "#Dot { text-emphasis: dot #ff0000; } #Under { text-emphasis: filled; text-emphasis-position: under; color: #0000ff; }
#Open { text-emphasis: open; } #Str { text-emphasis: \"*\"; }
#Down, #Left, #ZhDown { writing-mode: vertical-rl; height: 100px; text-emphasis: filled; } #Left { text-emphasis-position: over left; }
#Zh, #Ja, #Rb { text-emphasis: filled; } #RbDown { writing-mode: vertical-rl; height: 100px; text-emphasis: filled; }",
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
    // A ruby's base takes marks outside its annotation (§3.4), one
    // annotation em (10 px) further out than あ's; its <rt> takes none.
    let rb = one(&d, "Rb", "あ").baseline;
    let mut m = marks(&d, "Rb");
    m.sort_by(|a, b| a.0[0].total_cmp(&b.0[0]));
    let edge = rb - ASCENT * 20.0 - 5.0;
    assert!(
        m.len() == 3
            && close(m[0].0[0], 10.0)
            && (m[0].0[1] - edge).abs() <= 0.5
            && close(m[1].0[0], 30.0)
            && close(m[2].0[0], 50.0)
            && m[1..].iter().all(|m| (m.0[1] - (edge - 10.0)).abs() <= 0.5),
        "{m:?} {rb}"
    );
    // Down a column: right of 本's annotation, 10 px past 日's mark.
    let g = one(&d, "RbDown", "日");
    let mut m = marks(&d, "RbDown");
    m.sort_by(|a, b| a.0[1].total_cmp(&b.0[1]));
    assert!(
        m.len() == 2 && close(m[0].0[0], g.x + 25.0) && close(m[1].0[0], g.x + 35.0),
        "{m:?} {g:?}"
    );
}

/// Each drawn character inside element `id` and its glyph's origin and
/// advance, CSS px.
fn pens(d: &Doc, id: &str) -> Vec<(char, f32, f32)> {
    let r = d.get(id);
    let mut out = Vec::new();
    for it in &d.ui.frame().items {
        let cmd = match &it.cmd {
            DrawCmd::StyledText { text, .. } => &**text,
            c => c,
        };
        if let DrawCmd::Text {
            text,
            x,
            size,
            bounds,
            placements,
            ..
        } = cmd
            && (r.y..r.y + r.height).contains(&(bounds.y + bounds.height / 2.0))
        {
            let mut pen = x - r.x;
            for (c, p) in text.chars().zip(placements) {
                out.push((c, pen + p.x * size, p.advance * size));
                pen += p.advance * size;
            }
        }
    }
    out
}

/// The drawn characters of line `n` (from 0) of element `id`, by their
/// baseline.
fn line(d: &Doc, id: &str, n: usize) -> String {
    let mut rows: Vec<(f32, String)> = Vec::new();
    for g in glyphs(d, id) {
        match rows.iter_mut().find(|(b, _)| (b - g.baseline).abs() < 1.0) {
            Some((_, t)) => t.push_str(&g.text),
            None => rows.push((g.baseline, g.text.clone())),
        }
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    rows.get(n).map(|r| r.1.clone()).unwrap_or_default()
}

/// The origin and advance of character `c`'s `nth` glyph in `id`.
fn pen(d: &Doc, id: &str, c: char, nth: usize) -> (f32, f32) {
    let all = pens(d, id);
    all.iter()
        .filter(|p| p.0 == c)
        .nth(nth)
        .map(|p| (p.1, p.2))
        .unwrap_or_else(|| panic!("{c} #{nth} in {id}: {all:?}"))
}

/// CSS Text 4 §8.5 `text-spacing-trim`, in Noto Sans SC at 20 px, whose
/// fullwidth punctuation is one em (20 px) and whose `halt` sets it half
/// an em, kerning its blank half (the start of an opening bracket, the
/// end of a closing one and, in a Chinese font, of its colons and dots).
/// `normal` is `space-start trim-adjacent allow-end`: a line's first
/// opening bracket is spaced; an opening one after an opening one, and a
/// closing one before a closing one or a dot, are trimmed (§8.5.2); a
/// closing one ends a line trimmed only where it would not otherwise fit
/// (§8.5). `trim-all` trims every one; `space-all` none; `trim-start` and
/// `space-first` trim a line's first, `space-first` not on the first line.
///
/// Ways it could fail: `hwid` glyphs or an advance cut on the wrong side
/// (an opening bracket keeps its blank half and loses its ink); pairs
/// trimmed across a line break or not at all; the first line's bracket
/// trimmed under `space-first`; `allow-end` never pulling the next line's
/// `日。` back, or trimming a closing bracket that fits; `space-end`
/// trimming.
#[test]
fn fullwidth_punctuation_trims_as_text_spacing_trim_says() {
    let d = doc(
        r#"<p id="N">（「日」）。水</p>
<p id="All" style="text-spacing-trim: trim-all">（「日」）。水</p>
<p id="Sp" style="text-spacing-trim: space-all">（「日」）。水</p>
<p id="TS" style="text-spacing-trim: trim-start; width: 100px">（日日日日（日</p>
<p id="SF" style="text-spacing-trim: space-first; width: 100px">（日日日日（日</p>
<p id="AE" style="width: 110px">日日日日日。日日</p>
<p id="Fit" style="width: 120px">日日日日日。日日</p>
<p id="SE" style="text-spacing-trim: space-end; width: 110px">日日日日日。日日</p>"#,
        "",
    );
    let near =
        |(x, w): (f32, f32), at: f32, width: f32| (x - at).abs() < 0.6 && (w - width).abs() < 0.01;
    // normal: （ spaced at the line's start; 「 after （, 」 before ） and
    // ） before 。 trimmed; 。 before 水 not.
    for (c, n, at, w) in [
        ('（', 0, 0.0, 20.0),
        ('「', 0, 10.0, 10.0),
        ('日', 0, 30.0, 20.0),
        ('」', 0, 50.0, 10.0),
        ('）', 0, 60.0, 10.0),
        ('。', 0, 70.0, 20.0),
        ('水', 0, 90.0, 20.0),
    ] {
        assert!(near(pen(&d, "N", c, n), at, w), "{c}: {:?}", pens(&d, "N"));
    }
    // trim-all: every bracket and dot half an em; an opening one's ink
    // starts where its box does.
    assert!(
        near(pen(&d, "All", '（', 0), -10.0, 10.0),
        "{:?}",
        pens(&d, "All")
    );
    assert!(
        near(pen(&d, "All", '。', 0), 60.0, 10.0),
        "{:?}",
        pens(&d, "All")
    );
    assert!(
        near(pen(&d, "All", '水', 0), 70.0, 20.0),
        "{:?}",
        pens(&d, "All")
    );
    assert!(
        near(pen(&d, "Sp", '水', 0), 120.0, 20.0),
        "{:?}",
        pens(&d, "Sp")
    );
    // trim-start trims each line's first bracket; space-first all but the
    // first line's. The second （, spaced in the line, would not fit
    // after 日日日日, so it starts the second line.
    assert_eq!(line(&d, "TS", 1), "（日");
    assert!(
        near(pen(&d, "TS", '（', 0), -10.0, 10.0),
        "{:?}",
        pens(&d, "TS")
    );
    assert!(
        near(pen(&d, "TS", '（', 1), -10.0, 10.0),
        "{:?}",
        pens(&d, "TS")
    );
    assert_eq!(line(&d, "SF", 1), "（日");
    assert!(
        near(pen(&d, "SF", '（', 0), 0.0, 20.0),
        "{:?}",
        pens(&d, "SF")
    );
    assert!(
        near(pen(&d, "SF", '（', 1), -10.0, 10.0),
        "{:?}",
        pens(&d, "SF")
    );
    // allow-end: 日。 (no break before 。) fits the 110 px line only with
    // 。 trimmed (100 + 20 + 10), so it is; on a 120 px line it fits
    // spaced; space-end leaves it to the next line.
    assert_eq!(line(&d, "AE", 0), "日日日日日。");
    assert!(
        near(pen(&d, "AE", '。', 0), 100.0, 10.0),
        "{:?}",
        pens(&d, "AE")
    );
    assert_eq!(line(&d, "Fit", 0), "日日日日日。");
    assert!(
        near(pen(&d, "Fit", '。', 0), 100.0, 20.0),
        "{:?}",
        pens(&d, "Fit")
    );
    assert_eq!(line(&d, "SE", 0), "日日日日");
    assert_eq!(line(&d, "SE", 1), "日。日日");
}

/// CSS Text 4 §8.4 `text-autospace`: `normal` puts an eighth of an
/// ideograph's advance (Noto Sans SC's 水 at 20 px: 2.5 px) between an
/// ideograph and a Latin letter or a digit on one line, inside the
/// character before, not where a space already is; none at a line's
/// break, so the room never pushes a character to the next line;
/// `replace` sets a U+0020 between them at that width; `punctuation`
/// spaces French's `:` (a no-break space) and `?` (a narrow one, here a
/// fifth of an em: the face has neither U+202F nor U+2009).
///
/// Ways it could fail: the room on the wrong side or a quarter em;
/// spacing kept at a line's end, breaking `日日日` early; ideographic
/// punctuation taken for an ideograph; `no-autospace` or
/// `ideograph-alpha` ignored; `replace` adding to the space.
#[test]
fn autospace_spaces_ideographs_from_latin_as_text_autospace_says() {
    let d = doc(
        r#"<p id="N">日A日1日。A</p>
<p id="Off" style="text-autospace: no-autospace">日A日1日</p>
<p id="Alpha" style="text-autospace: ideograph-alpha">日A日1日</p>
<p id="Sp">日 A</p>
<p id="Rep" style="text-autospace: replace">日 A</p>
<p id="W" style="width: 60px">日日日Ab</p>
<p id="Fr" lang="fr" style="text-autospace: punctuation">a:b?</p>"#,
        "",
    );
    let at = |id: &str, c: char, n: usize| pen(&d, id, c, n).0;
    // 日 20, A 12.16 (608 units), 1 11.1 (555 units), each + 2.5.
    for (c, n, x) in [
        ('A', 0, 22.5),
        ('日', 1, 37.16),
        ('1', 0, 59.66),
        ('日', 2, 73.26),
    ] {
        assert!((at("N", c, n) - x).abs() < 0.6, "{c}: {:?}", pens(&d, "N"));
    }
    // 。 is no ideograph: nothing between it and A.
    let full_stop = pen(&d, "N", '。', 0);
    assert!(
        (at("N", 'A', 1) - full_stop.0 - full_stop.1).abs() < 0.6,
        "{:?}",
        pens(&d, "N")
    );
    assert!(
        (at("Off", 'A', 0) - 20.0).abs() < 0.6,
        "{:?}",
        pens(&d, "Off")
    );
    assert!(
        (at("Alpha", 'A', 0) - 22.5).abs() < 0.6,
        "{:?}",
        pens(&d, "Alpha")
    );
    assert!(
        (at("Alpha", '1', 0) - 57.16).abs() < 0.6,
        "{:?}",
        pens(&d, "Alpha")
    );
    // A space keeps its own width (224 units, 4.48 px); replace sets it
    // at 2.5 px.
    assert!(
        (at("Sp", 'A', 0) - 24.48).abs() < 0.6,
        "{:?}",
        pens(&d, "Sp")
    );
    assert!(
        (at("Rep", 'A', 0) - 22.5).abs() < 0.6,
        "{:?}",
        pens(&d, "Rep")
    );
    // Three ideographs fill the 60 px line: no room after the third.
    assert_eq!(line(&d, "W", 0), "日日日");
    assert_eq!(line(&d, "W", 1), "Ab");
    // a 11.26 + 4.48 (U+00A0), : 5.56, b 12.36 + 4 (an em / 5).
    assert!(
        (at("Fr", ':', 0) - 15.74).abs() < 0.6,
        "{:?}",
        pens(&d, "Fr")
    );
    assert!(
        (at("Fr", '?', 0) - 37.66).abs() < 0.6,
        "{:?}",
        pens(&d, "Fr")
    );
}

/// CSS Text 4 §9.2.1: hanging changes fit and alignment, but never
/// admits a following glyph into an end punctuation's zero advance.
#[test]
fn hanging_punctuation_hangs_only_at_the_requested_line_edges() {
    let d = doc(
        r#"<p id="F" style="hanging-punctuation: first; width: 100px">「日日」</p>
<p id="FE" style="hanging-punctuation: force-end; width: 100px">日日日日日。日</p>
<p id="AE" style="hanging-punctuation: allow-end; width: 100px">日日日日日。日</p>
<p id="AF" style="hanging-punctuation: allow-end; width: 120px">日日日日日。日</p>
<p id="FF" style="hanging-punctuation: force-end; width: 120px">日日日日日。日</p>
<p id="L" style="hanging-punctuation: last; width: 100px; text-align: right">“Hi”</p>
<p id="N" style="hanging-punctuation: none; width: 100px">「日日」</p>
<p id="P" style="hanging-punctuation: first"><span style="padding-left: 5px">「日日」</span></p>
<p id="T" style="hanging-punctuation: first; text-spacing-trim: trim-all">「日日」</p>"#,
        "",
    );
    for (id, c, x, advance) in [
        ("F", '「', -20.0, 20.0),
        ("F", '日', 0.0, 20.0),
        ("N", '日', 20.0, 20.0),
        ("P", '日', 25.0, 20.0),
        ("T", '日', 0.0, 20.0),
        ("FE", '。', 100.0, 0.0),
        ("AE", '。', 100.0, 0.0),
        ("AF", '。', 100.0, 20.0),
        ("FF", '。', 100.0, 0.0),
        ("L", '”', 100.0, 0.0),
    ] {
        let actual = pen(&d, id, c, 0);
        assert!(
            (actual.0 - x).abs() < 0.6 && (actual.1 - advance).abs() < 0.6,
            "{id} {c}: {actual:?}, expected {x}, {advance}"
        );
    }
    for id in ["FE", "AE", "AF", "FF"] {
        assert_eq!(line(&d, id, 1), "日", "{id}: {:?}", pens(&d, id));
        let days = pens(&d, id).into_iter().filter(|p| p.0 == '日').count();
        assert_eq!(days, 6, "no character lost or doubled: {id}");
    }
}
