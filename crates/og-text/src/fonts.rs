//! The collection every text is laid out from: the font files a game's
//! packs ship, the fonts it loads itself and its locale's font packs. A
//! closed fontique collection: no host font is ever discovered, so every
//! host lays text out from the same faces.
//!
//! This module only hands files to fontique and says which families its
//! generic families and its fallbacks name. fontique reads each face's
//! family, weight, style and width from the file and matches and
//! synthesises them (CSS Fonts 4 §5); Parley picks a face for each
//! character cluster, first from the text's `font-family` list, then from
//! the fallbacks of the cluster's script and the text's locale
//! ([`Collection::set_fallback`]). Nothing is declared over a file: no
//! alias family, weight range or `unicode-range`.

pub use parley::fontique::GenericFamily;
use parley::fontique::{
    Blob, FallbackKey, FamilyId, FontStyle, FontWeight, FontWidth, Language, Script, ScriptExt,
};
use parley::style::FontFamilyName;
use parley::{FontContext, FontData, LayoutContext};
use skrifa::{FontRef, MetadataProvider};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A registered font file: what a laid-out run's glyphs are rastered from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceId(pub u32);

/// The faces texts are laid out from, and the context Parley lays them
/// out with.
pub struct Collection {
    pub context: FontContext,
    /// Parley's layout scratch, kept between texts.
    layout: LayoutContext<u16>,
    /// Each file once, by the bytes' address.
    files: Vec<Arc<Vec<u8>>>,
    /// Each file's families (a collection file may hold several).
    families: Vec<Vec<FamilyId>>,
    /// The file of each registered blob.
    blobs: BTreeMap<u64, FaceId>,
    /// Moves when a file is added or a generic family or a fallback list
    /// is set: a text laid out before may now take other faces.
    revision: u64,
}

impl Default for Collection {
    fn default() -> Self {
        Self {
            context: FontContext {
                collection: parley::fontique::Collection::new(
                    parley::fontique::CollectionOptions {
                        shared: false,
                        system_fonts: false,
                    },
                ),
                source_cache: Default::default(),
            },
            layout: LayoutContext::new(),
            files: Vec::new(),
            families: Vec::new(),
            blobs: BTreeMap::new(),
            revision: 0,
        }
    }
}

/// The scripts a segment's characters without one take when no character
/// near them has one either.
const COMMON: [Script; 3] = [
    Script::from_bytes(*b"Zyyy"),
    Script::from_bytes(*b"Zinh"),
    Script::from_bytes(*b"Zzzz"),
];

impl Collection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the faces of font file `data`, each under the family,
    /// weight, style and width the file gives it. The same `Arc` again is
    /// the same file.
    pub fn add(&mut self, data: Arc<Vec<u8>>) -> Result<FaceId, String> {
        if let Some(i) = self.files.iter().position(|f| Arc::ptr_eq(f, &data)) {
            return Ok(FaceId(i as u32));
        }
        let blob = Blob::new(data.clone());
        let registered = self.context.collection.register_fonts(blob.clone(), None);
        if registered.iter().all(|(_, fonts)| fonts.is_empty()) {
            return Err("not a font file, or one that holds no face".into());
        }
        let id = FaceId(self.files.len() as u32);
        self.files.push(data);
        self.families
            .push(registered.iter().map(|(family, _)| *family).collect());
        self.blobs.insert(blob.id(), id);
        self.revision += 1;
        Ok(id)
    }

    /// The family of file `id`'s first face, as the file names it: what a
    /// `font-family` list calls it.
    pub fn family_name(&mut self, id: FaceId) -> Option<&str> {
        let family = *self.families.get(id.0 as usize)?.first()?;
        self.context.collection.family_name(family)
    }

    /// What CSS generic family `generic` (`sans-serif`, `serif`,
    /// `monospace`, `ui-rounded`, `emoji`) names: the named families of
    /// `css`, a `font-family` list, in its order. A family the collection
    /// lacks is skipped, as in CSS.
    pub fn set_generic(&mut self, generic: GenericFamily, css: &str) {
        let ids: Vec<FamilyId> = FontFamilyName::parse_css_list(css)
            .flatten()
            .filter_map(|name| match name {
                FontFamilyName::Named(n) => self.context.collection.family_id(&n),
                FontFamilyName::Generic(_) => None,
            })
            .collect();
        self.context
            .collection
            .set_generic_families(generic, ids.into_iter());
        self.revision += 1;
    }

    /// The faces a text in `locale` (BCP 47) takes a character from when
    /// no family of its `font-family` list draws it: `faces`' families in
    /// order, for every script (a character without a script of its own,
    /// such as ★, belongs to its neighbours'). `None` sets what a text in
    /// any other locale takes.
    ///
    /// fontique keeps a list of its own only for the locales it tells
    /// apart within a script (`zh-TW`, `zh-HK`, `ja` and `ko` for Han,
    /// `fa` and `ur` for Arabic, and so on): a text in one of those reads
    /// its locale's list, any other the `None` lists, and every text
    /// then Han's `None` list (fontique's own rule, for punctuation). A
    /// script's default locale (`zh-CN` for Han) is its `None` list, which
    /// a call for it replaces. So set `None` first, then each locale whose
    /// order differs.
    pub fn set_fallback(&mut self, locale: Option<&str>, faces: &[FaceId]) {
        let language = match locale {
            Some(l) => match Language::parse(l) {
                Ok(l) => Some(l),
                Err(_) => return,
            },
            None => None,
        };
        let mut ids: Vec<FamilyId> = Vec::new();
        for face in faces {
            for family in self.families.get(face.0 as usize).into_iter().flatten() {
                if !ids.contains(family) {
                    ids.push(*family);
                }
            }
        }
        let scripts = Script::all_samples()
            .iter()
            .map(|(script, _)| *script)
            .chain(COMMON);
        for script in scripts {
            self.context.collection.set_fallbacks(
                FallbackKey::new(script, language.as_ref()),
                ids.iter().copied(),
            );
        }
        self.revision += 1;
    }

    /// Moves whenever the faces a text may take change: a file added, a
    /// generic family or a fallback list set.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The file a laid-out run's font was registered from.
    pub fn face_of(&self, font: &FontData) -> Option<FaceId> {
        self.blobs.get(&font.data.id()).copied()
    }

    /// A registered file's bytes.
    pub fn file(&self, id: FaceId) -> &Arc<Vec<u8>> {
        &self.files[id.0 as usize]
    }

    /// Whether a face of registered file `id` maps `c` to a glyph.
    pub fn covers(&self, id: FaceId, c: char) -> bool {
        let data = self.file(id).as_slice();
        (0..)
            .map_while(|i| FontRef::from_index(data, i).ok())
            .any(|f| f.charmap().map(c).is_some())
    }

    /// The fontique context and Parley's layout context, to build with.
    pub(crate) fn contexts(&mut self) -> (&mut FontContext, &mut LayoutContext<u16>) {
        (&mut self.context, &mut self.layout)
    }

    /// The primary font of `css` at `weight` and slant (CSS Fonts 4 §5):
    /// the face fontique matches in the first of its families, a generic's
    /// as [`Collection::set_generic`] set them, that maps a space. Its
    /// file and its index there.
    pub fn primary(&mut self, css: &str, weight: f32, italic: bool) -> Option<(FaceId, u32)> {
        let style = if italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
        for name in FontFamilyName::parse_css_list(css).flatten() {
            let ids: Vec<FamilyId> = match name {
                FontFamilyName::Named(n) => {
                    self.context.collection.family_id(&n).into_iter().collect()
                }
                FontFamilyName::Generic(g) => self.context.collection.generic_families(g).collect(),
            };
            for id in ids {
                let Some(family) = self.context.collection.family(id) else {
                    continue;
                };
                let Some(font) =
                    family.match_font(FontWidth::NORMAL, style, FontWeight::new(weight), true)
                else {
                    continue;
                };
                let Some(blob) = font.load(Some(&mut self.context.source_cache)) else {
                    continue;
                };
                let Some(&face) = self.blobs.get(&blob.id()) else {
                    continue;
                };
                let spaced = FontRef::from_index(&self.files[face.0 as usize], font.index())
                    .is_ok_and(|f| f.charmap().map(' ').is_some());
                if spaced {
                    return Some((face, font.index()));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parley::StyleProperty;
    use parley::style::FontFamily;
    use std::borrow::Cow;

    fn file(path: &str) -> Arc<Vec<u8>> {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates/");
        Arc::new(std::fs::read(format!("{root}{path}")).unwrap_or_else(|e| panic!("{path}: {e}")))
    }

    /// Each cluster's file and wght coordinate, in text order; `None`
    /// where no face was found.
    fn faces(
        fonts: &mut Collection,
        css: &str,
        weight: f32,
        locale: Option<&str>,
        text: &str,
    ) -> Vec<(char, Option<FaceId>, i16)> {
        let mut lcx = LayoutContext::<()>::new();
        let mut b = lcx.ranged_builder(&mut fonts.context, text, 1.0, false);
        b.push_default(StyleProperty::FontFamily(FontFamily::Source(
            Cow::Borrowed(css),
        )));
        b.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
        b.push_default(StyleProperty::FontSize(16.0));
        if let Some(l) = locale.and_then(|l| l.parse().ok()) {
            b.push_default(StyleProperty::Locale(Some(l)));
        }
        let mut layout = b.build(text);
        layout.break_all_lines(None);
        let mut out = Vec::new();
        for line in layout.lines() {
            for run in line.runs() {
                let id = fonts.face_of(run.font());
                let wght = run.normalized_coords().first().map_or(0, |c| c.to_bits());
                for c in run.clusters() {
                    let ch = text[c.text_range()].chars().next().unwrap();
                    let drawn = c.glyphs().all(|g| g.id != 0);
                    out.push((ch, id.filter(|_| drawn), wght));
                }
            }
        }
        out.sort_by_key(|&(c, _, _)| text.find(c));
        out
    }

    /// A file's faces are registered under the file's own family, weight
    /// and style: Chakra Petch's SemiBold and Bold files are one family,
    /// and a weight takes the nearer file (CSS Fonts 4 §5.2: 400 looks up
    /// to 600, 700 is Bold's own).
    #[test]
    fn a_family_and_its_weights_come_from_the_files() {
        let mut fonts = Collection::new();
        let semi = fonts
            .add(file("styles/shared/ChakraPetch-SemiBold.ttf"))
            .unwrap();
        let bold = fonts
            .add(file("styles/shared/ChakraPetch-Bold.ttf"))
            .unwrap();
        assert_eq!(fonts.family_name(semi), Some("Chakra Petch"));
        assert_eq!(fonts.family_name(bold), Some("Chakra Petch"));
        for (weight, face) in [(400.0, semi), (600.0, semi), (700.0, bold), (900.0, bold)] {
            assert_eq!(
                faces(&mut fonts, "\"chakra petch\"", weight, None, "A"),
                vec![('A', Some(face), 0)],
                "weight {weight}"
            );
        }
        assert_eq!(
            fonts.primary("Missing, \"Chakra Petch\"", 700.0, false),
            Some((bold, 0))
        );
        assert!(fonts.add(Arc::new(b"not a font".to_vec())).is_err());
    }

    /// A text takes each character from the first family of its list that
    /// draws it, a variable face at the text's weight: Latin and `·` from
    /// Barlow, ideographs and ★ (which Barlow lacks) from Noto Sans SC,
    /// its `wght` axis moved to 700.
    #[test]
    fn a_later_family_draws_what_an_earlier_one_lacks() {
        let mut fonts = Collection::new();
        let barlow = fonts
            .add(file("styles/shared/BarlowSemiCondensed-SemiBold.ttf"))
            .unwrap();
        let sc = fonts.add(file("styles/cjk/NotoSansSC[wght].ttf")).unwrap();
        let css = "\"Barlow Semi Condensed\", \"Noto Sans SC\"";
        let got = faces(&mut fonts, css, 700.0, None, "L·关★");
        let face: Vec<_> = got.iter().map(|&(c, f, _)| (c, f)).collect();
        assert_eq!(
            face,
            vec![
                ('L', Some(barlow)),
                ('·', Some(barlow)),
                ('关', Some(sc)),
                ('★', Some(sc)),
            ]
        );
        assert_eq!(got[0].2, 0, "a static face has no location");
        assert!(got[2].2 > 0, "wght moved above its default: {got:?}");
        assert_eq!(got[2].2, got[3].2);
    }

    /// A generic family names the theme's families, in their order.
    #[test]
    fn a_generic_names_the_themes_families() {
        let mut fonts = Collection::new();
        let barlow = fonts
            .add(file("styles/shared/BarlowSemiCondensed-SemiBold.ttf"))
            .unwrap();
        let sc = fonts.add(file("styles/cjk/NotoSansSC[wght].ttf")).unwrap();
        fonts.set_generic(
            GenericFamily::SansSerif,
            "\"Barlow Semi Condensed\", \"Not Shipped\", \"Noto Sans SC\"",
        );
        let got = faces(&mut fonts, "sans-serif", 400.0, None, "L关");
        let face: Vec<_> = got.iter().map(|&(c, f, _)| (c, f)).collect();
        assert_eq!(face, vec![('L', Some(barlow)), ('关', Some(sc))]);
        assert_eq!(fonts.primary("sans-serif", 400.0, false), Some((barlow, 0)));
    }

    /// What no family of the list draws comes from the fallbacks of the
    /// text's locale, in that locale's order, whatever script the
    /// character's neighbours give it (★ after Latin). A locale fontique
    /// keeps a list of its own for (`zh-TW`, `ja`) reads the lists set for
    /// no locale until its own is set, as every other locale does.
    #[test]
    fn fallbacks_are_per_locale() {
        let mut fonts = Collection::new();
        fonts
            .add(file("styles/shared/BarlowSemiCondensed-SemiBold.ttf"))
            .unwrap();
        let sans = fonts.add(file("styles/cjk/NotoSansSC[wght].ttf")).unwrap();
        let serif = fonts.add(file("styles/cjk/NotoSerifSC[wght].ttf")).unwrap();
        let css = "\"Barlow Semi Condensed\"";
        let han = |fonts: &mut Collection, locale| {
            let got = faces(fonts, css, 400.0, locale, "L★关");
            (got[1].1, got[2].1)
        };
        assert_eq!(han(&mut fonts, None), (None, None), "no fallback is set");
        fonts.set_fallback(None, &[sans, serif]);
        for locale in [None, Some("en"), Some("zh-CN"), Some("zh-TW"), Some("ja")] {
            assert_eq!(
                han(&mut fonts, locale),
                (Some(sans), Some(sans)),
                "{locale:?}"
            );
        }
        fonts.set_fallback(Some("zh-TW"), &[serif, sans]);
        assert_eq!(
            han(&mut fonts, Some("zh-TW")),
            (Some(sans), Some(serif)),
            "★ sits in a Latin segment, whose list is the one set for no locale"
        );
        assert_eq!(han(&mut fonts, Some("zh-Hant")).1, Some(serif));
        assert_eq!(han(&mut fonts, Some("ja")).1, Some(sans));
        assert_eq!(han(&mut fonts, Some("en")).1, Some(sans));
    }
}
