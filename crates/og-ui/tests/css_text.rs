//! CSS text, fonts and generated content on document elements, checked
//! against the numbers the specifications give: what each line draws, its
//! glyphs and their advances, its decorations and its generated text.
//! The face is Noto Sans (Latin, from the project skeleton's fonts): its
//! own advances, kerning and OpenType features, shaped as a game's cooked
//! font with its file is.
//!
//! Ways these could fail, each covered below:
//! - a counter reset, incremented and set in the wrong order or scope, a
//!   nested `counters()` missing its outer level, a quote pair from the
//!   wrong depth, `attr()` not reading the element, or a later
//!   `content: none` drawing the earlier string →
//!   `generated_content_counts_quotes_and_reads_attributes`
//! - a variant keyword or a feature setting never reaching the shaper,
//!   `font-kerning: none` still kerning, a variant winning over the
//!   settings that come after it, or a line measured without the features
//!   it is drawn with → `font_variants_and_feature_settings_draw_the_fonts_glyphs`,
//!   `east_asian_variants_take_the_fonts_full_width_forms`

use crate::support::Doc;
use og_ui::*;

/// The font at `path` (from the repository's root), shaped from its file,
/// drawn under the test font's handle.
fn face(path: &str) -> UiFont {
    let mut font = crate::features::font_at(path);
    font.handle = crate::support::test_font().handle;
    font
}

const NOTO: &str = "templates/fonts/latin/NotoSans-Latin.ttf";

/// A document of `body` under `css`, laid out 400 px wide in Noto Sans.
fn doc(body: &str, css: &str) -> Doc {
    doc_in(NOTO, body, css)
}

/// [`doc`] in the font at `path`.
fn doc_in(path: &str, body: &str, css: &str) -> Doc {
    let text = format!(
        "<meta name=\"component\" content=\"t\">\n<div><div id=\"Root\">{body}</div></div>\n<style>#Root {{ width: 400px; }} p {{ margin: 0; }} {css}</style>"
    );
    let d = Doc::with_font(&text, &[], face(path));
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    d
}

/// A drawn line: its text, left, baseline, width, size, tracking and
/// glyphs (a shaped form's id, or `None` for its character's own glyph).
#[derive(Clone, Debug)]
struct Line {
    text: String,
    x: f32,
    baseline: f32,
    width: f32,
    size: f32,
    tracking: f32,
    glyphs: Vec<og_draw::GlyphPlacement>,
}

/// The lines drawn inside element `id`, top to bottom.
fn lines(d: &Doc, id: &str) -> Vec<Line> {
    let r = d.get(id);
    let mut out: Vec<Line> =
        d.ui.frame()
            .items
            .iter()
            .filter_map(|it| match it.cmd.unstyled() {
                DrawCmd::Text {
                    text,
                    x,
                    baseline,
                    bounds,
                    size,
                    tracking,
                    placements,
                    ..
                } if (r.y..r.y + r.height).contains(&(bounds.y + bounds.height / 2.0))
                    && (r.x - 0.5..r.x + r.width + 0.5).contains(&bounds.x) =>
                {
                    Some(Line {
                        text: text.clone(),
                        x: *x,
                        baseline: *baseline,
                        width: bounds.width,
                        size: *size,
                        tracking: *tracking,
                        glyphs: placements.clone(),
                    })
                }
                _ => None,
            })
            .collect();
    out.sort_by(|a, b| a.baseline.total_cmp(&b.baseline).then(a.x.total_cmp(&b.x)));
    // A synthesised bold's second copy, one device pixel right, is the
    // same line.
    out.dedup_by(|b, a| a.text == b.text && a.baseline == b.baseline && (b.x - a.x).abs() <= 1.0);
    out
}

/// The text drawn inside `id`, its lines joined by `|`.
fn drawn(d: &Doc, id: &str) -> String {
    lines(d, id)
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

/// CSS Lists 3 §4 orders an element's counter properties reset,
/// increment, set (each `counter-increment` defaulting to 1); a nested
/// reset starts a new counter, scoped to the element, its descendants
/// and its following siblings (§4.4.1), that `counters()` joins with the
/// outer one (§4.5); `quotes` pairs by depth (CSS Generated Content 3 §3.2);
/// `attr()` reads the originating element; the cascade's later
/// `content: none` leaves nothing.
#[test]
fn generated_content_counts_quotes_and_reads_attributes() {
    let d = doc(
        r#"<div id="Sec"><h3 id="H1">a</h3><h3 id="H2">b</h3><h3 id="H3">c</h3><h3 id="H4">d</h3></div>
<div class="list"><p class="item" id="I1">one</p><div class="list"><p class="item" id="I11">x</p><p class="item" id="I12">y</p></div><p class="item" id="I2">two</p></div>
<p id="Q" data-x="7">q</p><p id="N">n</p>"#,
        r#"#Sec { counter-reset: sec 4; }
h3 { counter-increment: sec; margin: 0; font-size: 14px; }
h3::before { content: counter(sec, upper-roman) ". "; }
#H3 { counter-set: sec 10; }
.list { counter-reset: item; }
.item { counter-increment: item; }
.item::before { content: counters(item, ".") " "; }
#Q { quotes: "«" "»" "‹" "›"; }
#Q::before { content: open-quote open-quote; }
#Q::after { content: close-quote close-quote " (" attr(data-x) ")"; }
#N::before { content: "hidden"; }
#N::before { content: none; }"#,
    );
    // 4 + 1, + 1, then set to 10 after its increment, then + 1.
    for (id, text) in [
        ("H1", "V. a"),
        ("H2", "VI. b"),
        ("H3", "X. c"),
        ("H4", "XI. d"),
        ("I1", "1 one"),
        ("I11", "1.1 x"),
        ("I12", "1.2 y"),
        // The inner reset's scope takes its following sibling too (§4.4.1).
        ("I2", "1.3 two"),
        ("Q", "«‹q›» (7)"),
        ("N", "n"),
    ] {
        assert_eq!(drawn(&d, id), text, "#{id}");
    }
}

/// CSS Lists 3 §3.3: `list-style-type` takes a counter style or a
/// string, and CSS Counter Styles 3 §6.1's `symbols()` declares an
/// anonymous one (`symbolic` by default, an empty prefix, a space for
/// suffix): cyclic * † over three items is *, †, *; alphabetic x y is
/// x, y, xx (bijective base 2); numeric 0 1 is 1, 10, 11 (binary);
/// symbolic * + is *, +, ** (through the `list-style` shorthand). A
/// string is the marker as written. An
/// `@counter-style` name draws its own symbols, also through `::marker`'s
/// `counter(list-item, …)` in a `<ul>`, whose items count 1, 2, 3 there
/// too. In `counter()` a `fixed` style past its symbols falls back to
/// `decimal`. The drawn marker leaves out its suffix's trailing space,
/// which its gap stands for.
///
/// Ways it could fail: `symbols()` refused or drawn as `decimal`, its
/// suffix taken as `. ` (`*.`), a wrong default system, the string
/// drawn with quotes, or `list-item` read as 0 in a `<ul>` (B, B, B).
#[test]
fn list_style_types_take_counter_styles_symbols_and_strings() {
    let d = doc(
        r#"<ol id="Cyc"><li>a</li><li>b</li><li>c</li></ol>
<ol id="Alpha"><li>a</li><li>b</li><li>c</li></ol>
<ol id="Num"><li>a</li><li>b</li><li>c</li></ol>
<ol id="Sym"><li>a</li><li>b</li><li>c</li></ol>
<ol id="Named"><li>a</li><li>b</li><li>c</li></ol>
<ul id="Str"><li>a</li><li>b</li></ul>
<ul id="Short"><li>a</li></ul>
<p id="P">x</p>"#,
        r#"@counter-style thumbs { system: cyclic; symbols: "A" "B"; suffix: " "; }
#Cyc { list-style-type: symbols(cyclic "*" "†"); }
#Alpha { list-style-type: symbols(alphabetic "x" "y"); }
#Num { list-style-type: symbols(numeric "0" "1"); }
#Sym { list-style: symbols("*" "+") inside; list-style-position: outside; }
#Named { list-style-type: thumbs; }
#Str { list-style-type: "→ "; }
#Short { list-style: inside "– "; }
#P { counter-reset: c 3; }
#P::before { content: counter(c, symbols(cyclic "*" "+")) counter(c, symbols(fixed "①" "②")) counters(c, ".", symbols(numeric "0" "1")) " "; }"#,
    );
    for (list, markers) in [
        ("Cyc", ["*", "†", "*"]),
        ("Alpha", ["x", "y", "xx"]),
        ("Num", ["1", "10", "11"]),
        ("Sym", ["*", "+", "**"]),
        ("Named", ["A", "B", "A"]),
    ] {
        assert_eq!(d.markers(list), markers, "#{list}");
    }
    assert_eq!(d.markers("Str"), ["→", "→"]);
    assert_eq!(d.markers("Short"), ["–\u{a0}"]);
    assert_eq!(drawn(&d, "P"), "*311 x");
    // A `<ul>` alone: no `<ol>` or `list-style` makes its items count.
    let d = doc(
        r#"<ul id="U"><li>a</li><li>b</li><li>c</li></ul>"#,
        r#"@counter-style thumbs { system: cyclic; symbols: "A" "B"; }
li::marker { content: counter(list-item, thumbs); color: #ff0000; }"#,
    );
    assert_eq!(d.markers("U"), ["A", "B", "A"]);
    // `::marker`'s color inks the marker, not the item.
    let ink = |want: &str| {
        d.ui.frame()
            .items
            .iter()
            .find_map(|it| match it.cmd.unstyled() {
                DrawCmd::Text { text, color, .. } if text == want => Some(*color),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no {want}"))
    };
    assert!(near(ink("B"), Srgba::rgb(255, 0, 0)));
    assert!(!near(ink("b"), Srgba::rgb(255, 0, 0)));
    for bad in [
        "symbols(alphabetic \"x\")",
        "symbols()",
        "12px",
        "thumbs thumbs",
    ] {
        let errors = UiDocument::parse(
            "ui/t.svelte",
            &format!("<meta name=\"component\" content=\"t\"><ul><li>A</li></ul><style>li {{ list-style-type: {bad}; }}</style>"),
        )
        .unwrap_err()
        .join("\n");
        assert!(errors.contains("list-style-type"), "{bad}: {errors}");
    }
}

/// CSS Lists 3 §4.4.2: `reversed(c)` with no value starts at the sum of
/// the negated increments in its scope, up to the first element that
/// sets it (whose value is added instead), plus the last non-zero
/// negated increment once more. With -1 per item, three items count
/// 3, 2, 1. With a set of 10 on the third of four, the start is
/// 1 + 1 + 10 + 1 = 13, so they read 12, 11, 10 (the set), 9. An inner
/// reset's counter takes its element's following siblings too (§4.4.1),
/// so their increments count on it, not on the outer one.
/// An explicit `reversed(c) 5` starts at 5. The start is found as the
/// run passes the scope, so the readers take it from the next tick.
#[test]
fn reversed_counters_start_where_their_increments_end() {
    let mut d = doc(
        r#"<div id="R"><p id="R1">a</p><p id="R2">b</p><p id="R3">c</p></div>
<div id="S"><p id="S1">a</p><p id="S2">b</p><p id="S3">c</p><p id="S4">d</p></div>
<div id="N"><p id="N1">a</p><div id="In"><p id="N2">x</p><p id="N3">y</p></div><p id="N4">b</p></div>
<div id="E"><p id="E1">a</p><p id="E2">b</p></div>"#,
        "#R, #S, #N, #In { counter-reset: reversed(c); } #E { counter-reset: reversed(c) 5; }
p { counter-increment: c -1; } p::before { content: counter(c) \". \"; }
#S3 { counter-set: c 10; }",
    );
    d.step(InputFrame::default());
    for (id, text) in [
        ("R1", "3. a"),
        ("R2", "2. b"),
        ("R3", "1. c"),
        ("S1", "12. a"),
        ("S2", "11. b"),
        ("S3", "10. c"),
        ("S4", "9. d"),
        // The outer counts N1 alone; the inner N2, N3 and N4 after it.
        ("N1", "1. a"),
        ("N2", "3. x"),
        ("N3", "2. y"),
        ("N4", "1. b"),
        ("E1", "4. a"),
        ("E2", "3. b"),
    ] {
        assert_eq!(drawn(&d, id), text, "#{id}");
    }
    d.step(InputFrame::default());
    assert_eq!(drawn(&d, "R1"), "3. a", "a settled count stays");
}

/// CSS Containment 2 §3.3: under `contain: style` a `counter-increment`
/// makes a counter of the subtree's own, nested in the one outside, and
/// leaves the outside one as it was; quotes opened inside are closed again
/// at its end.
///
/// Ways it could fail: the increments inside counting on the outer
/// counter (2, 3, then 4 after the box), or the quote depth leaking out
/// (‘ after the box instead of “).
#[test]
fn contain_style_keeps_counters_and_quotes_inside() {
    let d = doc(
        r#"<div id="L"><p class="i" id="A">a</p><div id="Box"><p class="i" id="B1">b</p><p class="i" id="B2">c</p><p id="Q">q</p></div><p class="i" id="C">d</p><p id="R">r</p></div>"#,
        r#"#L { counter-reset: i; }
.i { counter-increment: i; }
.i::before { content: counters(i, ".") " "; }
#Box { contain: style; }
#Q::before, #R::before { content: open-quote; }"#,
    );
    for (id, text) in [
        ("A", "1 a"),
        ("B1", "1.1 b"),
        ("B2", "1.2 c"),
        ("Q", "\u{201C}q"),
        ("C", "2 d"),
        ("R", "\u{201C}r"),
    ] {
        assert_eq!(drawn(&d, id), text, "#{id}");
    }
}

/// CSS Text 3 §2.1: each transform rewrites what is drawn — every letter
/// cased, each word's first letter titlecased (after a hyphen too), ASCII
/// to its full-width form (a space to U+3000), small kana to full size.
#[test]
fn text_transform_rewrites_the_drawn_text() {
    let d = doc(
        r#"<p id="U">Straße mix</p><p id="L">ÉCOLE Mix</p><p id="C">hello wide-world 3d</p><p id="W">Ab 1!</p><p id="K">ァッ</p><p id="N" class="up">none</p>"#,
        r#"#U { text-transform: uppercase; } #L { text-transform: lowercase; }
#C { text-transform: capitalize; } #W { text-transform: full-width; }
#K { text-transform: full-size-kana; }
.up { text-transform: uppercase; } #N { text-transform: none; }"#,
    );
    for (id, text) in [
        ("U", "STRASSE MIX"),
        ("L", "école mix"),
        ("C", "Hello Wide-World 3d"),
        ("W", "Ａｂ\u{3000}１！"),
        ("K", "アツ"),
        ("N", "none"),
    ] {
        assert_eq!(drawn(&d, id), text, "#{id}");
    }
}

/// CSS Inline 3 §4.3 / CSS 2 §10.8.1: a `<number>` line height is
/// inherited as the number and multiplies each element's own size; a
/// percentage and a length compute to px against the element's own
/// size, wherever its rule declares that size, and are inherited as px.
/// CSS Text 3 §8.2: `letter-spacing` computes to an absolute length,
/// so a larger child keeps its parent's px, and it is added after every
/// character.
#[test]
fn line_height_and_letter_spacing_compute_as_css_does() {
    let d = doc(
        r#"<div id="N"><p id="N1">a</p><p id="N2">b</p></div>
<div id="P"><p id="P1">a</p><p id="P2">b</p></div><p id="X">c</p>
<div id="S"><p id="S1">AVA</p><p id="S2">AVA</p></div><p id="S0">AVA</p>"#,
        "#N { line-height: 2; font-size: 10px; } #N2, #P2 { font-size: 20px; }
#P { line-height: 200%; font-size: 10px; } #X { line-height: 1.5em; font-size: 12px; }
#S { letter-spacing: 0.5em; } #S2 { font-size: 28px; }",
    );
    let h = |id| d.get(id).height;
    assert_eq!((h("N1"), h("N2")), (20.0, 40.0), "2 × each own size");
    assert_eq!(
        (h("P1"), h("P2")),
        (20.0, 20.0),
        "200% of the parent's 10px, inherited as px"
    );
    assert_eq!(h("X"), 18.0, "1.5em of its own 12px, declared after it");
    let (s1, s2, s0) = (
        &lines(&d, "S1")[0],
        &lines(&d, "S2")[0],
        &lines(&d, "S0")[0],
    );
    assert_eq!(s1.tracking * s1.size, 7.0, "0.5em of 14px");
    assert_eq!(s2.tracking * s2.size, 7.0, "the parent's 7px at 28px");
    assert_eq!(s0.tracking, 0.0);
    // Three characters, each followed by 7px.
    assert!(
        (s1.width - s0.width - 21.0).abs() < 0.01,
        "{} vs {}",
        s1.width,
        s0.width
    );
}

/// The thickening and shear each text command of `id` is drawn with:
/// its synthesised weight (0: none) and its effects' transform.
fn synthesised(d: &Doc, id: &str) -> Vec<(f32, [f32; 4])> {
    let r = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| {
            let (text, transform) = match &it.cmd {
                DrawCmd::StyledText { text, effects } => (&**text, effects.transform),
                c => (c, [1.0, 0.0, 0.0, 1.0]),
            };
            match text {
                DrawCmd::Text { bounds, weight, .. }
                    if (r.y..r.y + r.height).contains(&(bounds.y + bounds.height / 2.0)) =>
                {
                    Some((*weight, transform))
                }
                _ => None,
            }
        })
        .collect()
}

/// CSS Fonts 4 §3.1: a weight or a slant no face of the font has (the
/// test face is Noto Sans Regular alone) is synthesised, unless
/// `font-synthesis-weight` or `-style` (or `font-synthesis` without
/// that word) says none. §2.4: an oblique is slanted by its angle,
/// 14deg when it names none. The italic texts here are spans, the path
/// that synthesises.
#[test]
fn font_synthesis_and_oblique_angles_draw_as_css_fonts_says() {
    let d = doc(
        r#"<p id="R">Ab</p><p id="B">Ab</p><p id="Bn">Ab</p><p id="Bs">Ab</p><p id="I">Ab</p>
<p id="O">Ab</p><p id="O14">Ab</p><p id="In">Ab</p>"#,
        "#B, #Bn, #Bs { font-weight: 700; font-style: italic; }
#Bn { font-synthesis-weight: none; } #Bs { font-synthesis: style; }
#I { font-style: italic; } #O { font-style: oblique 30deg; } #O14 { font-style: oblique; }
#In { font-style: italic; font-synthesis-style: none; }",
    );
    let bold = |id| {
        let cmds = synthesised(&d, id);
        assert!(!cmds.is_empty(), "#{id} draws");
        cmds.iter().any(|(w, _)| *w > 0.0)
    };
    let shear = |id| synthesised(&d, id)[0].1[2];
    assert!(!bold("R") && shear("R") == 0.0, "regular");
    assert!(
        bold("B") && shear("B") < 0.0,
        "a bold italic with neither face"
    );
    assert!(
        !bold("Bn") && shear("Bn") < 0.0,
        "font-synthesis-weight: none"
    );
    assert!(!bold("Bs") && shear("Bs") < 0.0, "font-synthesis: style");
    assert!(
        (shear("O") + 30f32.to_radians().tan()).abs() < 1e-4,
        "{}",
        shear("O")
    );
    assert!(
        (shear("O14") + 14f32.to_radians().tan()).abs() < 1e-4,
        "{}",
        shear("O14")
    );
    assert!(shear("I") < 0.0, "italic slants");
    assert_eq!(shear("In"), 0.0, "font-synthesis-style: none");
}

/// CSS Text 3 §8.1: `word-spacing` adds its length after each word
/// separator (a space or a no-break space), never between the letters
/// of a word; it computes to an absolute length, so a child twice the
/// size keeps its parent's px.
#[test]
fn word_spacing_widens_each_word_separator() {
    let d = doc(
        r#"<p id="N">ab cd ef</p><p id="W">ab cd ef</p><p id="E">ab cd ef</p>
<div id="Em"><p id="Big">ab cd ef</p></div><p id="Nb">ab&nbsp;cd</p><p id="Nb0">ab&nbsp;cd</p>
<p id="One">abcdef</p><p id="One0">abcdef</p>"#,
        "#W, #Nb, #One { word-spacing: 10px; } #E { word-spacing: 0.5em; }
#Em { word-spacing: 0.5em; } #Big { font-size: 28px; }",
    );
    // Where each drawn glyph starts, from the pen and its advances.
    let starts = |id| -> Vec<f32> {
        let mut out = Vec::new();
        for l in lines(&d, id) {
            let mut pen = l.x;
            for g in &l.glyphs {
                out.push(pen + g.x * l.size);
                pen += g.advance * l.size;
            }
        }
        out
    };
    let (n, w, e) = (starts("N"), starts("W"), starts("E"));
    assert_eq!((n.len(), w.len()), (8, 8), "{n:?} {w:?}");
    // a b ␠ c d ␠ e f: each glyph after k separators is k × 10px on.
    for (i, k) in [0, 0, 0, 1, 1, 1, 2, 2].into_iter().enumerate() {
        assert!(
            (w[i] - n[i] - 10.0 * k as f32).abs() < 0.05,
            "10px at {i}: {w:?} vs {n:?}"
        );
        assert!(
            (e[i] - n[i] - 7.0 * k as f32).abs() < 0.05,
            "0.5em of 14px at {i}: {e:?}"
        );
    }
    let width = |id| lines(&d, id).iter().map(|l| l.width).sum::<f32>();
    assert!(
        (width("W") - width("N") - 20.0).abs() < 0.05,
        "two separators"
    );
    let big = starts("Big");
    let big0 = starts("N").iter().map(|x| x * 2.0).collect::<Vec<_>>();
    assert!(
        ((big[7] - big[0]) - (big0[7] - big0[0]) - 14.0).abs() < 0.1,
        "the parent's 7px per separator at 28px: {big:?}"
    );
    assert!(
        (width("Nb") - width("Nb0") - 10.0).abs() < 0.05,
        "a no-break space separates"
    );
    assert!(
        (width("One") - width("One0")).abs() < 0.01,
        "no separator, no room"
    );
}

/// CSS Text 3 §5.2: `break-all` lets a line break between any two
/// letters, so a word wider than its 50px box fills each line instead of
/// overflowing; `normal` keeps it whole.
#[test]
fn word_break_all_breaks_inside_words() {
    let d = doc(
        r#"<p id="A">abcdefghij</p><p id="N">abcdefghij</p>"#,
        "#A, #N { width: 50px; } #A { word-break: break-all; }",
    );
    let a = lines(&d, "A");
    let letters = |l: &Line| l.text.replace('\u{200b}', "");
    assert_eq!(
        a.iter().map(letters).collect::<String>(),
        "abcdefghij",
        "{a:?}"
    );
    assert!(a.len() >= 2 && a.iter().all(|l| l.width <= 50.0), "{a:?}");
    let n = lines(&d, "N");
    assert_eq!(n.len(), 1);
    assert!(n[0].width > 50.0, "normal keeps the word whole: {n:?}");
}

/// The decoration lines drawn over and under `id`'s lines (the test
/// spaces its paragraphs): their rects and colours.
fn rules(d: &Doc, id: &str) -> Vec<(Rect, [f32; 4])> {
    let r = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| it.cmd.filled())
        .filter(|(q, _)| {
            q.height <= 8.0 && (r.y - 4.0..r.y + r.height + 10.0).contains(&(q.y + q.height / 2.0))
        })
        .collect()
}

/// CSS Text Decoration 3/4: an explicit `text-underline-offset` puts the
/// underline's top that far below the alphabetic baseline, an explicit
/// thickness is its height, `under` moves it to the font's descent (and
/// a length offsets it from there), `from-font` takes the font's own
/// underline position and thickness;
/// overline runs above the capitals and line-through between the
/// baseline and the x-height; double draws two lines a thickness apart;
/// the colour is the decoration's own. Decorations propagate to the
/// text inside (§2.1): a child's `text-decoration: none` keeps them.
#[test]
fn decoration_lines_sit_where_css_text_decoration_puts_them() {
    let d = doc(
        r#"<p id="Off">under</p><p id="Auto">under</p><p id="Under">under</p>
<p id="Over">under</p><p id="Thru">under</p><p id="Dbl">under</p>
<div id="Prop"><p id="Kept">under</p></div><p id="Font">under</p><p id="Low">under</p>"#,
        "p { margin-bottom: 16px; }
#Off { text-decoration: underline 4px #ff0000; text-underline-offset: 5px; }
#Auto { text-decoration: underline; } #Under { text-decoration: underline; text-underline-position: under; }
#Over { text-decoration: overline; } #Thru { text-decoration: line-through; }
#Dbl { text-decoration: underline double 2px; }
#Prop { text-decoration: underline 2px; } #Kept { text-decoration: none; }
#Font, #Low { font-size: 40px; text-decoration: underline; }
#Font { text-decoration-thickness: from-font; text-underline-offset: from-font; }
#Low { text-underline-position: under; text-underline-offset: 3px; }",
    );
    let baseline = |id| lines(&d, id)[0].baseline;
    let off = rules(&d, "Off");
    assert_eq!(off.len(), 1, "{off:?}");
    assert_eq!((off[0].0.y - baseline("Off"), off[0].0.height), (5.0, 4.0));
    assert!(near(off[0].1, Srgba::rgb(255, 0, 0)));
    assert!(
        (off[0].0.width - lines(&d, "Off")[0].width).abs() <= 1.0,
        "across the text"
    );
    // Noto Sans: post underline position -100 and thickness 50, hhea
    // descent -293, of 1000 units to the em. `under` puts the line's top
    // at the descent (14px × 0.293 = 4.1px), clear of every descender;
    // a length offsets it from there. `from-font` takes the post metrics.
    // The line and the baseline each snap to the pixel grid.
    let auto = rules(&d, "Auto")[0].0.y - baseline("Auto");
    let under = rules(&d, "Under")[0].0.y - baseline("Under");
    assert!(
        auto > 0.0 && (under - 4.102).abs() <= 1.0,
        "auto {auto}, under {under}"
    );
    let low = rules(&d, "Low")[0].0;
    assert!(
        (low.y - baseline("Low") - (40.0 * 0.293 + 3.0)).abs() <= 1.0,
        "{low:?}"
    );
    let font = rules(&d, "Font")[0].0;
    assert_eq!(
        (font.y - baseline("Font"), font.height),
        (4.0, 2.0),
        "40px × 0.1 and × 0.05"
    );
    let em = 14.0;
    let over = rules(&d, "Over")[0].0;
    assert!(
        over.y < baseline("Over") - 0.714 * em,
        "above the cap height: {over:?}"
    );
    let thru = rules(&d, "Thru")[0].0;
    let t = baseline("Thru") - thru.y;
    assert!(t > 0.0 && t < 0.536 * em + 1.0, "under the x-height: {t}");
    let dbl = rules(&d, "Dbl");
    assert_eq!(dbl.len(), 2, "{dbl:?}");
    assert_eq!((dbl[1].0.y - dbl[0].0.y, dbl[0].0.height), (4.0, 2.0));
    let kept = rules(&d, "Kept");
    assert!(
        kept.len() == 1 && kept[0].0.height == 2.0,
        "propagated: {kept:?}"
    );
}

fn near(a: [f32; 4], b: Srgba) -> bool {
    crate::support::near(a, b)
}

/// The outline effect of the text drawn in `id`.
fn outline(d: &Doc, id: &str) -> Option<og_draw::UiTextOutline> {
    let baseline = lines(d, id)[0].baseline;
    d.ui.frame().items.iter().find_map(|it| match &it.cmd {
        DrawCmd::StyledText { text, effects } => match &**text {
            DrawCmd::Text { baseline: b, .. } if *b == baseline => effects.outline,
            _ => None,
        },
        _ => None,
    })
}

/// The Compat Standard's `-webkit-text-fill-color` paints the glyphs over
/// `color`; `-webkit-text-stroke` strokes their outline at its width,
/// centred on the edge over the fill (as an SVG stroke), in `color`
/// unless it names its own. `paint-order: stroke` (SVG 2 §13.6) paints
/// the stroke under the fill, so only its outer half shows.
#[test]
fn text_stroke_and_fill_paint_as_the_compat_standard_does() {
    let d = doc(
        r#"<p id="Both">fill</p><p id="Own">s</p><p id="Under">s</p><p id="Plain">s</p>"#,
        "#Both { color: #ff0000; -webkit-text-fill-color: #0000ff; -webkit-text-stroke: 2px #00ff00; }
#Own { color: #ff0000; -webkit-text-stroke-width: 3px; }
#Under { -webkit-text-stroke: 4px #00ff00; paint-order: stroke fill; }",
    );
    let color = |id| match d
        .ui
        .frame()
        .items
        .iter()
        .find_map(|it| match it.cmd.unstyled() {
            DrawCmd::Text {
                color, baseline, ..
            } if *baseline == lines(&d, id)[0].baseline => Some(*color),
            _ => None,
        }) {
        Some(c) => c,
        None => panic!("no text in #{id}"),
    };
    assert!(
        near(color("Both"), Srgba::rgb(0, 0, 255)),
        "the fill over color"
    );
    let both = outline(&d, "Both").expect("a stroke");
    assert_eq!((both.width, both.align), (2.0, og_draw::UiAlign::Center));
    assert!(near(both.color, Srgba::rgb(0, 255, 0)));
    let own = outline(&d, "Own").expect("a stroke");
    assert!(
        near(own.color, Srgba::rgb(255, 0, 0)),
        "currentcolor: {own:?}"
    );
    assert!(near(color("Own"), Srgba::rgb(255, 0, 0)));
    let under = outline(&d, "Under").expect("a stroke");
    assert_eq!((under.width, under.align), (2.0, og_draw::UiAlign::Outside));
    assert_eq!(outline(&d, "Plain"), None);
}

/// The glyph ids drawn in `id`'s first line (`None`: the character's own).
fn ids(d: &Doc, id: &str) -> Vec<Option<u16>> {
    lines(d, id)[0].glyphs.iter().map(|g| g.glyph).collect()
}

/// CSS Fonts 4 §6 turns each variant keyword into its OpenType feature
/// (§6.4–6.9), `font-kerning: none` turns `kern` off (§6.1) and
/// `font-feature-settings` applies last (§7.2, over a variant that set
/// the same feature). The expected glyphs are Noto Sans's own GSUB
/// substitutions and its `hmtx` advances (fontTools reads them): small
/// capitals a → 954, b → 967 (and A → 954 under `c2sc`); oldstyle 1–3
/// → 1092–1094; the slashed zero 1121; the fraction slash 371;
/// superior 1 → 121, inferior → 1123; the `fi` ligature 890. Each line is
/// as wide as what it draws.
#[test]
fn font_variants_and_feature_settings_draw_the_fonts_glyphs() {
    let d = doc(
        r#"<p id="Plain">AVA fi 123</p><p id="Kern">AVA</p><p id="Caps">ab</p><p id="All">Ab</p>
<p id="Old">0123</p><p id="Zero">0</p><p id="Frac">1/2</p><p id="Sup">1</p><p id="Sub">1</p>
<p id="NoLig">fi</p><p id="Set">fi 123</p><p id="Over">123</p><p id="Short">ab 1</p>
<div id="Parent"><p id="Child">ab</p></div>"#,
        r#"#Kern { font-kerning: none; } #Caps { font-variant-caps: small-caps; }
#All { font-variant-caps: all-small-caps; } #Old { font-variant-numeric: oldstyle-nums; }
#Zero { font-variant-numeric: slashed-zero; } #Frac { font-variant-numeric: diagonal-fractions; }
#Sup { font-variant-position: super; } #Sub { font-variant-position: sub; }
#NoLig { font-variant-ligatures: no-common-ligatures; }
#Set { font-feature-settings: "onum", "liga" 0; }
#Over { font-variant-numeric: oldstyle-nums; font-feature-settings: "onum" 0; }
#Short { font-variant: small-caps oldstyle-nums; } #Parent { font-variant-caps: small-caps; }"#,
    );
    let plain = lines(&d, "Plain").remove(0);
    let adv: Vec<f32> = plain.glyphs.iter().map(|g| g.advance).collect();
    assert!(adv[0] < 0.639 - 0.01, "A kerned against V: {adv:?}");
    assert_eq!(plain.glyphs[4].glyph, Some(890), "the fi ligature");
    let kern: Vec<f32> = lines(&d, "Kern")[0]
        .glyphs
        .iter()
        .map(|g| g.advance)
        .collect();
    assert!(
        kern.iter()
            .zip([0.639, 0.6, 0.639])
            .all(|(a, b)| (a - b).abs() < 1e-4),
        "hmtx advances, unkerned: {kern:?}"
    );
    assert_eq!(ids(&d, "Caps"), [Some(954), Some(967)]);
    assert_eq!(ids(&d, "All"), [Some(954), Some(967)]);
    assert_eq!(ids(&d, "Old")[1..], [Some(1092), Some(1093), Some(1094)]);
    assert_eq!(ids(&d, "Zero"), [Some(1121)]);
    let frac = ids(&d, "Frac");
    assert!(
        frac[1] == Some(371) && frac[0].is_some() && frac[2].is_some(),
        "{frac:?}"
    );
    assert_eq!(
        (ids(&d, "Sup"), ids(&d, "Sub")),
        (vec![Some(121)], vec![Some(1123)])
    );
    assert_eq!(ids(&d, "NoLig"), [None, None], "f and i, no ligature");
    let set = ids(&d, "Set");
    assert_eq!(set[..2], [None, None]);
    assert_eq!(set[3..], [Some(1092), Some(1093), Some(1094)]);
    assert_eq!(ids(&d, "Over"), [None, None, None], "the settings win");
    assert_eq!(ids(&d, "Short"), [Some(954), Some(967), None, Some(1092)]);
    assert_eq!(ids(&d, "Child"), [Some(954), Some(967)], "inherited");
    for id in ["Plain", "Old", "Set", "Short"] {
        let l = &lines(&d, id)[0];
        let drawn: f32 = l.glyphs.iter().map(|g| g.advance).sum::<f32>() * l.size;
        assert!(
            (l.width - drawn).abs() < 0.01,
            "#{id} measured {} drew {drawn}",
            l.width
        );
    }
}

/// CSS Fonts 4 §6.10: `font-variant-east-asian: full-width` turns on
/// `fwid`, Noto Sans SC's full-width forms of A, 1 and ! (glyphs 7935,
/// 7919, 7903, each one em wide); `proportional-width` (`pwid`) leaves
/// Latin that is already proportional as it is.
#[test]
fn east_asian_variants_take_the_fonts_full_width_forms() {
    let d = doc_in(
        "tests/fixtures/ui-text/assets/fonts/NotoSansSC-Regular-subset.ttf",
        r#"<p id="Full">A1!</p><p id="Prop">A!</p>"#,
        "#Full { font-variant-east-asian: full-width; } #Prop { font-variant-east-asian: proportional-width; }",
    );
    let full = lines(&d, "Full").remove(0);
    assert_eq!(ids(&d, "Full"), [Some(7935), Some(7919), Some(7903)]);
    assert!(
        full.glyphs.iter().all(|g| g.advance == 1.0),
        "{:?}",
        full.glyphs
    );
    assert_eq!(ids(&d, "Prop"), [None, None]);
}

/// A line's text as it reads: no-break spaces as spaces, no zero-width
/// break opportunities.
fn reads(l: &Line) -> String {
    l.text.replace('\u{a0}', " ").replace('\u{200b}', "")
}

/// CSS Text 3 §4: `normal` collapses spaces and line breaks; `pre` keeps
/// both and does not wrap; `pre-line` keeps only line breaks;
/// `pre-wrap` keeps spaces and wraps after them (they hang at the line's
/// end); `break-spaces` keeps each space's room on its line; the
/// `preserve-spaces` longhand turns line breaks into kept spaces. A tab
/// advances to the next multiple of `tab-size` columns (§4.2).
/// `text-wrap: nowrap` keeps a line wider than its box whole (§5.1).
#[test]
fn white_space_and_tab_size_process_the_source_as_css_text_does() {
    let src = "a   b\n  c";
    let d = doc(
        &format!(
            r#"<p id="Normal">{src}</p><p id="Pre">{src}</p><p id="Line">  a   b
 c  </p><p id="Wrap">alpha   beta gamma</p><p id="Break">alpha   beta gamma</p>
<p id="Spaces">a  b
c</p><p id="Tab4">a	b</p><p id="Tab8">a	b</p>
<p id="NoWrap">alpha beta gamma</p><p id="Wraps">alpha beta gamma</p>"#
        ),
        "#Pre, #Tab4, #Tab8 { white-space: pre; } #Tab4 { tab-size: 4; } #Line { white-space: pre-line; }
#Wrap { white-space: pre-wrap; width: 70px; } #Break { white-space: break-spaces; width: 70px; }
#Spaces { white-space-collapse: preserve-spaces; }
#NoWrap, #Wraps { width: 70px; } #NoWrap { text-wrap: nowrap; }",
    );
    let read = |id| lines(&d, id).iter().map(reads).collect::<Vec<_>>();
    assert_eq!(read("Normal"), ["a b c"]);
    assert_eq!(read("Pre"), ["a   b", "  c"]);
    assert_eq!(read("Line"), ["a b", "c"]);
    assert_eq!(read("Spaces"), ["a  b c"]);
    assert_eq!(read("Tab4"), ["a   b"], "to column 4");
    assert_eq!(read("Tab8"), ["a       b"], "to column 8");
    // pre-wrap breaks after the run of spaces, which hangs: each line
    // fits; break-spaces keeps all three spaces on the first line.
    let wrap = lines(&d, "Wrap");
    assert_eq!(
        wrap.iter()
            .map(|l| reads(l).trim_end().to_string())
            .collect::<Vec<_>>(),
        ["alpha", "beta", "gamma"]
    );
    let alpha = d.ui.fonts().body.metrics.measure("alpha", 14.0);
    let space = d.ui.fonts().body.metrics.measure(" ", 14.0);
    let brk = lines(&d, "Break");
    assert_eq!(reads(&brk[0]), "alpha   ");
    assert!(
        (brk[0].width - (alpha + 3.0 * space)).abs() < 0.5,
        "{brk:?}"
    );
    assert!(wrap[0].width > alpha, "its spaces are kept: {wrap:?}");
    let nowrap = lines(&d, "NoWrap");
    assert!(nowrap.len() == 1 && nowrap[0].width > 70.0, "{nowrap:?}");
    assert_eq!(lines(&d, "Wraps").len(), 2);
}

/// CSS Text 3 §5.5: under `overflow-wrap: anywhere` and `break-word` a
/// word too wide for its line breaks between its letters, after moving to
/// a line of its own (so "ab" keeps its line, unlike `break-all`);
/// `anywhere` counts those breaks in its min-content width (the widest
/// letter, m), `break-word` does not (the whole word); `word-break:
/// break-word` is `anywhere`. `normal` overflows.
#[test]
fn overflow_wrap_breaks_a_word_only_when_it_cannot_fit() {
    let d = doc(
        r#"<p id="Normal">ab abcdefghijklmnop</p><p id="Any">ab abcdefghijklmnop</p><p id="Word">ab abcdefghijklmnop</p>
<p id="WB">ab abcdefghijklmnop</p><p id="MinAny">abcdefghijklm</p><p id="MinWord">abcdefghijklm</p>"#,
        "#Normal, #Any, #Word, #WB { width: 60px; } #Any, #MinAny { overflow-wrap: anywhere; }
#Word, #MinWord { overflow-wrap: break-word; } #WB { word-break: break-word; }
#MinAny, #MinWord { width: min-content; }",
    );
    let read = |id| lines(&d, id).iter().map(reads).collect::<Vec<_>>();
    assert_eq!(read("Normal"), ["ab", "abcdefghijklmnop"]);
    for id in ["Any", "Word", "WB"] {
        let l = lines(&d, id);
        assert_eq!(reads(&l[0]), "ab", "#{id}: {l:?}");
        assert_eq!(
            l.iter().skip(1).map(reads).collect::<String>(),
            "abcdefghijklmnop"
        );
        assert!(
            l.len() > 2 && l.iter().all(|l| l.width <= 60.0),
            "#{id}: {l:?}"
        );
    }
    let m = &d.ui.fonts().body.metrics;
    assert_eq!(d.get("MinAny").width, m.measure("m", 14.0).round());
    assert!(
        d.get("MinWord").width >= m.measure("abcdefghijklm", 14.0).floor(),
        "{}",
        d.get("MinWord").width
    );
}

/// The right edge of a line as drawn: its left and its glyphs' advances,
/// the last one's tracking-free.
fn drawn_right(l: &Line) -> f32 {
    let n = l.glyphs.len();
    let adv: f32 = l.glyphs[..n - 1].iter().map(|g| g.advance).sum();
    l.x + (adv + l.glyphs[n - 1].advance - l.tracking) * l.size
}

/// CSS Text 3 §6.3: `text-align-last` aligns the last line (right,
/// centre), `auto` under `justify` leaves it at the start and
/// `justify-all` justifies it too; §7.4: a justified line ends at the
/// box's edge, its free room at the spaces (`auto`, `inter-word`) or
/// between every two characters (`inter-character`), and `none` leaves
/// it at the start.
#[test]
fn justification_and_the_last_line_align_as_css_text_does() {
    let words = "alpha beta gamma delta epsilon zeta eta theta";
    let d = doc(
        &["Auto", "Last", "Center", "All", "Char", "None"]
            .map(|id| format!(r#"<p id="{id}">{words}</p>"#))
            .concat(),
        "p { width: 200px; text-align: justify; } #Last { text-align-last: right; }
#Center { text-align: start; text-align-last: center; } #All { text-align: justify-all; }
#Char { text-justify: inter-character; } #None { text-justify: none; }",
    );
    let l = |id| lines(&d, id);
    let auto = l("Auto");
    assert_eq!(auto.len(), 2, "{auto:?}");
    assert!((drawn_right(&auto[0]) - 200.0).abs() < 0.05, "{auto:?}");
    assert!(
        auto[0]
            .glyphs
            .iter()
            .filter(|g| g.advance > 0.5 && g.glyph.is_none())
            .count()
            >= 3
    );
    assert_eq!(auto[1].x, 0.0, "auto: the last line at the start");
    assert!(drawn_right(&auto[1]) < 199.0);
    assert!((drawn_right(&l("Last")[1]) - 200.0).abs() < 0.5, "right");
    let c = &l("Center")[1];
    assert!(
        (c.x - (200.0 - c.width) / 2.0).abs() < 0.5,
        "centred: {c:?}"
    );
    assert!(
        (drawn_right(&l("All")[1]) - 200.0).abs() < 0.05,
        "justify-all"
    );
    // inter-character: one extra after every character, the last's past
    // the end, so the last glyph ends at the edge.
    let (ch, plain) = (&l("Char")[0], &l("None")[0]);
    let extra: Vec<f32> = ch
        .glyphs
        .iter()
        .zip(&plain.glyphs)
        .map(|(a, b)| a.advance - b.advance)
        .collect();
    assert!(
        extra
            .iter()
            .all(|e| (e - extra[0]).abs() < 1e-4 && *e > 0.0),
        "{extra:?}"
    );
    let pen: f32 = ch.glyphs[..ch.glyphs.len() - 1]
        .iter()
        .map(|g| g.advance)
        .sum::<f32>();
    let last = plain.glyphs.last().unwrap().advance;
    assert!(((pen + last) * ch.size - 200.0).abs() < 0.05, "{ch:?}");
    let none = l("None");
    assert!(
        none[0].x == 0.0 && drawn_right(&none[0]) < 199.0,
        "{none:?}"
    );
}
