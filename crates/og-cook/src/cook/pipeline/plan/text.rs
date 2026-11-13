use super::*;

pub(super) fn plan_font<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { options, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    match font_settings(s, &options.font) {
        Ok(f) => {
            // "font 2": the Unicode spaces a font lacks are cooked as empty
            // glyphs; "font 3": the field is of each glyph's outline
            // union; "font 4": negative-bearing fill and post-sign error
            // correction; "font 5": every face carries OpenType data
            // for paragraph layout; "font 6": default substitutions
            // and referenced contextual lookups; "font 7": layout-only
            // tables without runtime outlines; "font 8": the vertical
            // forms (`vert`, `vrt2`) of its characters; "font 9": a glyph keyed
            // by code point and by id is one image in the atlas.
            // Older fonts re-cook.
            let st = stamp(&[b"font 9", f.describe().as_bytes(), &hash]);
            outs.push((s.rel.clone(), format::Font::NAME, st, Work::Font(s, f)));
        }
        Err(e) => early_errors.push((s.rel.clone(), "font", e)),
    }
}
