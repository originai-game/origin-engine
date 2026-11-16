//! Whether a face has an OpenType variant form of a character (CSS Fonts 4
//! §6.3, §6.5: the small capitals and super- and subscripts a face lacks
//! are synthesised).

use crate::shape::{Features, FontFile, GlyphKey};

impl FontFile {
    /// Whether turning on feature `tag` (`smcp`, `c2sc`, `sups`, `subs`)
    /// draws `c` with another glyph than its own; false in a build that
    /// does not shape.
    pub fn has_variant(&self, tag: [u8; 4], c: char) -> bool {
        let run = c.to_string();
        let any = |_: GlyphKey| true;
        let glyph = |features| {
            self.shape_with(&run, false, features, &any)
                .and_then(|g| g.first().and_then(|g| g.glyph))
        };
        let form = glyph(Features::NONE.with_settings(&[(tag, 1)]));
        form.is_some() && form != glyph(Features::NONE)
    }
}
