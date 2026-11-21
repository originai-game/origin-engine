//! A font's baseline set (CSS Inline 3 §3.2): where each baseline lies
//! above its alphabetic one, em, read from its OpenType `BASE` table and
//! synthesized where the table does not give it (Appendix A.2).

use crate::shape::FontFile;
use skrifa::raw::TableProvider;
use skrifa::raw::types::Tag;

/// A baseline of CSS's `<baseline-metric>` (CSS Inline 3 §4.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Baseline {
    #[default]
    Alphabetic,
    /// The text-under edge: the content area's bottom.
    TextBottom,
    /// The ideographic-under baseline.
    Ideographic,
    /// Halfway between the alphabetic and x-height baselines.
    Middle,
    /// Halfway between the ideographic-over and -under baselines.
    Central,
    Mathematical,
    Hanging,
    /// The text-over edge: the content area's top.
    TextTop,
}

impl Baseline {
    /// A `<baseline-metric>` keyword.
    pub fn parse(v: &str) -> Option<Self> {
        Some(match v {
            "alphabetic" => Self::Alphabetic,
            "text-bottom" => Self::TextBottom,
            "ideographic" => Self::Ideographic,
            "middle" => Self::Middle,
            "central" => Self::Central,
            "mathematical" => Self::Mathematical,
            "hanging" => Self::Hanging,
            "text-top" => Self::TextTop,
            _ => return None,
        })
    }
}

/// A font's baselines, em above its alphabetic baseline (negative below).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Baselines {
    pub text_top: f32,
    pub text_bottom: f32,
    pub ideographic: f32,
    pub central: f32,
    pub middle: f32,
    pub mathematical: f32,
    pub hanging: f32,
}

impl Baselines {
    /// Baseline `b`, em above the alphabetic one.
    pub fn at(&self, b: Baseline) -> f32 {
        match b {
            Baseline::Alphabetic => 0.0,
            Baseline::TextBottom => self.text_bottom,
            Baseline::Ideographic => self.ideographic,
            Baseline::Middle => self.middle,
            Baseline::Central => self.central,
            Baseline::Mathematical => self.mathematical,
            Baseline::Hanging => self.hanging,
            Baseline::TextTop => self.text_top,
        }
    }

    /// The baselines of a font with `ascent`, `descent` and `x_height`
    /// (em, the descent positive) and no `BASE` table, as CSS Inline 3
    /// Appendix A.2 synthesizes them: the ideographic-over and -under
    /// baselines from the ascent and descent, the mathematical baseline at
    /// the x-middle, the hanging one at 0.6em.
    pub fn synthesize(ascent: f32, descent: f32, x_height: f32) -> Self {
        Self {
            text_top: ascent,
            text_bottom: -descent,
            ideographic: -descent,
            central: (ascent - descent) / 2.0,
            middle: x_height / 2.0,
            mathematical: x_height / 2.0,
            hanging: 0.6,
        }
    }

    /// Each baseline times `size` (em to CSS px).
    pub fn scaled(&self, size: f32) -> Self {
        Self {
            text_top: self.text_top * size,
            text_bottom: self.text_bottom * size,
            ideographic: self.ideographic * size,
            central: self.central * size,
            middle: self.middle * size,
            mathematical: self.mathematical * size,
            hanging: self.hanging * size,
        }
    }

    /// The baselines of a box with no font of its own (CSS Inline 3
    /// Appendix A.3): `top` and `bottom` its margin edges, em above its
    /// alphabetic baseline; central, middle and mathematical halfway.
    pub fn of_box(top: f32, bottom: f32) -> Self {
        let mid = (top + bottom) / 2.0;
        Self {
            text_top: top,
            text_bottom: bottom,
            ideographic: bottom,
            central: mid,
            middle: mid,
            mathematical: mid,
            hanging: top,
        }
    }
}

/// `file`'s baselines. Its `BASE` table's horizontal ones for the default
/// script (else its first) come first; the rest are synthesized as CSS
/// Inline 3 Appendix A.2 suggests: ideographic-over and -under 1em apart
/// where it names one, else the ascent and descent
/// ([`Baselines::synthesize`]); the mathematical baseline at the minus
/// sign's centre where it has one.
pub fn baselines(file: &FontFile) -> Baselines {
    let synthesized = Baselines::synthesize(file.ascent(), file.descent(), file.x_height());
    match skrifa::FontRef::new(file.bytes()) {
        Ok(face) => of_face(&face, synthesized),
        Err(_) => synthesized,
    }
}

/// `face`'s baselines at variation `location`, as [`baselines`] finds them,
/// synthesized from its metrics there where its `BASE` table is silent.
pub fn face_baselines(
    face: &skrifa::FontRef,
    location: skrifa::instance::LocationRef,
) -> Baselines {
    use skrifa::MetadataProvider;
    let m = face.metrics(skrifa::instance::Size::unscaled(), location);
    let em = f32::from(m.units_per_em.max(1));
    let x_height = m.x_height.unwrap_or(m.ascent * 0.56);
    of_face(
        face,
        Baselines::synthesize(m.ascent / em, -m.descent / em, x_height / em),
    )
}

/// `face`'s `BASE` baselines over `out`, its synthesized ones.
fn of_face(face: &skrifa::FontRef, mut out: Baselines) -> Baselines {
    let face = face.clone();
    let upem = face
        .head()
        .map_or(1000.0, |h| f32::from(h.units_per_em()).max(1.0));
    let coords = base_coords(&face).unwrap_or_default();
    let romn = coords
        .iter()
        .find(|(t, _)| t == b"romn")
        .map_or(0.0, |c| c.1);
    let base = |tag: &[u8; 4]| {
        coords
            .iter()
            .find(|(t, _)| t == tag)
            .map(|&(_, c)| (c - romn) / upem)
    };
    let ideographic = match (base(b"ideo"), base(b"idtp")) {
        (Some(u), Some(o)) => Some((u, o)),
        (Some(u), None) => Some((u, u + 1.0)),
        (None, Some(o)) => Some((o - 1.0, o)),
        (None, None) => None,
    };
    if let Some((under, over)) = ideographic {
        out.ideographic = under;
        out.central = (under + over) / 2.0;
    }
    if let Some(m) = base(b"math").or_else(|| ink_centre(&face, '\u{2212}', upem)) {
        out.mathematical = m;
    }
    if let Some(h) = base(b"hang") {
        out.hanging = h;
    }
    out
}

/// The horizontal axis' baseline coordinates, font units, of the default
/// script (`DFLT`), else of the first script it describes.
fn base_coords(f: &skrifa::FontRef) -> Option<Vec<([u8; 4], f32)>> {
    let axis = f.base().ok()?.horiz_axis()?.ok()?;
    let tags: Vec<Tag> = axis
        .base_tag_list()?
        .ok()?
        .baseline_tags()
        .iter()
        .map(|t| t.get())
        .collect();
    let scripts = axis.base_script_list().ok()?;
    let records = scripts.base_script_records();
    let record = records
        .iter()
        .find(|r| r.base_script_tag() == Tag::new(b"DFLT"))
        .or(records.first())?;
    let values = record
        .base_script(scripts.offset_data())
        .ok()?
        .base_values()?
        .ok()?;
    Some(
        tags.iter()
            .zip(values.base_coords().iter())
            .filter_map(|(t, c)| Some((t.to_be_bytes(), f32::from(c.ok()?.coordinate()))))
            .collect(),
    )
}

/// The centre of `c`'s ink, em above the baseline.
fn ink_centre(f: &skrifa::FontRef, c: char, upem: f32) -> Option<f32> {
    use skrifa::MetadataProvider;
    let gid = f.charmap().map(c)?;
    let glyf = f.glyf().ok()?;
    let loca = f.loca(None).ok()?;
    let glyph = loca.get_glyf(gid, &glyf).ok()??;
    Some((f32::from(glyph.y_min()) + f32::from(glyph.y_max())) / 2.0 / upem)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font(path: &str) -> std::sync::Arc<FontFile> {
        let path = format!(
            "{}/../../templates/fonts/{path}",
            env!("CARGO_MANIFEST_DIR")
        );
        FontFile::new(std::sync::Arc::new(std::fs::read(path).unwrap())).unwrap()
    }

    /// Per fontTools: Noto Sans has no BASE table and no minus sign, so
    /// every baseline is synthesized from its metrics (typo ascent 1069,
    /// descent 293, x-height 536 per 1000). Noto Sans JP's BASE gives
    /// `ideo` -120 and no `idtp` (over 1em above: central 380) and no
    /// `math` (its minus sign's ink, 335 to 403: 369); its ascent and
    /// descent are hhea's 1160 and 288. A wrong build reads JP's
    /// ideographic baseline from its descent, or centres it on the
    /// content area.
    #[test]
    fn baselines_come_from_base_and_synthesize_the_rest() {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
        let b = baselines(&font("latin/NotoSans-Latin.ttf"));
        assert!(
            near(b.text_top, 1.069) && near(b.text_bottom, -0.293),
            "{b:?}"
        );
        assert!(
            near(b.ideographic, -0.293) && near(b.central, 0.388),
            "{b:?}"
        );
        assert!(
            near(b.middle, 0.268) && near(b.mathematical, 0.268),
            "{b:?}"
        );
        assert!(near(b.hanging, 0.6), "{b:?}");
        let j = baselines(&font("japanese/NotoSansJP-Regular.ttf"));
        assert!(
            near(j.text_top, 1.16) && near(j.text_bottom, -0.288),
            "{j:?}"
        );
        assert!(near(j.ideographic, -0.12) && near(j.central, 0.38), "{j:?}");
        assert!(
            near(j.mathematical, 0.369) && near(j.middle, 0.2715),
            "{j:?}"
        );
    }
}
