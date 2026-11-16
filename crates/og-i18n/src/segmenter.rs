//! ICU's word-segmentation dictionaries, loaded at run time: Chinese and
//! Japanese (`cjdict`, about 2 MB) and Thai, Lao, Khmer and Burmese
//! (`thaidict`, `laodict`, `khmerdict`, `burmesedict`, about 1 MB
//! together), which UAX #14 and #29 leave to a dictionary because these
//! scripts are written without spaces. The engine links none of them: the
//! cook copies a dictionary out of ICU's compiled data ([`compiled`]) into a
//! font pack's index for each shipped locale that needs it (docs/ui.md
//! §3.4), and the runtime installs it from there ([`install`]). With none
//! installed, these scripts break by the rules alone, as Parley does
//! without its `complex-scripts` feature.
//!
//! Every word and line segmenter here ([`word`], [`line`]) uses the
//! installed dictionaries; og-text hands them to Parley
//! (third_party/parley_engine/PATCHED.md), so laid-out lines and
//! [`crate::word_segments`] cut the same words.

use icu_locale_core::LanguageIdentifier;
use icu_properties::props::Script;
use icu_provider::prelude::*;
use icu_segmenter::options::{
    LineBreakOptions, LineBreakStrictness, LineBreakWordOption, WordBreakInvariantOptions,
};
use icu_segmenter::provider::{
    DictionaryBreakData, SegmenterDictionaryAutoV2, SegmenterDictionaryExtendedV2,
};
use icu_segmenter::{LineSegmenter, LineSegmenterBorrowed, WordSegmenter, WordSegmenterBorrowed};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// ICU's dictionaries: (marker attribute, the languages that take it).
pub const DICTIONARIES: &[(&str, &[&str])] = &[
    ("cjdict", &["ja", "zh"]),
    ("thaidict", &["th"]),
    ("laodict", &["lo"]),
    ("khmerdict", &["km"]),
    ("burmesedict", &["my"]),
];

/// The dictionary a BCP 47 language tag's text is cut with, by its primary
/// subtag: `cjdict` for `ja` and `zh`, `thaidict` for `th` and so on.
pub fn dictionary_for(tag: &str) -> Option<&'static str> {
    let primary = tag
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    DICTIONARIES
        .iter()
        .find(|(_, langs)| langs.contains(&primary.as_str()))
        .map(|(attr, _)| *attr)
}

/// What is installed, and the segmenters built over it (rebuilt after each
/// [`install`]; a segmenter borrows its dictionaries for the process).
struct Installed {
    dictionaries: BTreeMap<&'static str, &'static DictionaryBreakData<'static>>,
    word: Option<WordSegmenterBorrowed<'static>>,
    lines: Vec<(LineKey, LineSegmenterBorrowed<'static>)>,
}

type LineKey = (
    Option<LineBreakStrictness>,
    Option<LineBreakWordOption>,
    Option<LanguageIdentifier>,
);

static INSTALLED: RwLock<Installed> = RwLock::new(Installed {
    dictionaries: BTreeMap::new(),
    word: None,
    lines: Vec::new(),
});

/// A dictionary's bytes as the cook writes them: a tag byte, then for 0
/// (a `Char16Trie`) its u16 trie little-endian, and for 1 (a `ZeroTrie`)
/// the alphabet's inversion list (a u32 count and as many u32s) and the
/// trie's bytes.
fn encode(data: &DictionaryBreakData<'_>) -> Vec<u8> {
    match data {
        DictionaryBreakData::Char16Trie(t) => {
            let mut out = vec![0];
            out.extend(t.data.iter().flat_map(u16::to_le_bytes));
            out
        }
        DictionaryBreakData::ZeroTrie { alphabet, trie } => {
            let list = alphabet.get_inversion_list_vec();
            let mut out = vec![1];
            out.extend(u32::try_from(list.len()).unwrap_or(0).to_le_bytes());
            out.extend(list.iter().flat_map(|v| v.to_le_bytes()));
            out.extend_from_slice(trie.as_bytes());
            out
        }
    }
}

fn decode(bytes: &'static [u8]) -> Result<DictionaryBreakData<'static>, String> {
    let bad = || "a segmentation dictionary is cut short or malformed".to_string();
    match bytes.split_first() {
        Some((0, data)) => zerovec::ZeroVec::parse_bytes(data)
            .map(|d| {
                DictionaryBreakData::Char16Trie(icu_collections::char16trie::Char16Trie::new(d))
            })
            .map_err(|_| bad()),
        Some((1, rest)) => {
            let n = rest
                .get(..4)
                .and_then(|b| b.try_into().ok())
                .map(u32::from_le_bytes)
                .ok_or_else(bad)? as usize;
            let end = n
                .checked_mul(4)
                .and_then(|l| l.checked_add(4))
                .ok_or_else(bad)?;
            let list: Vec<u32> = rest
                .get(4..end)
                .ok_or_else(bad)?
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes(*c))
                .collect();
            let alphabet =
                icu_collections::codepointinvlist::CodePointInversionList::try_from_u32_inversion_list_slice(&list)
                    .map_err(|_| bad())?;
            Ok(DictionaryBreakData::ZeroTrie {
                alphabet,
                trie: zerotrie::ZeroTrieSimpleAscii::from_store(zerovec::ZeroVec::new_borrowed(
                    &rest[end..],
                )),
            })
        }
        _ => Err(bad()),
    }
}

/// The bytes of ICU's compiled dictionary `attribute` (one of
/// [`DICTIONARIES`]), for the cook to ship. Only the cook calls it, so
/// only the cook links the compiled dictionaries.
pub fn compiled(attribute: &str) -> Option<Vec<u8>> {
    let attr = DataMarkerAttributes::try_from_str(attribute).ok()?;
    let request = DataRequest {
        id: DataIdentifierBorrowed::for_marker_attributes(attr),
        metadata: {
            let mut m = DataRequestMetadata::default();
            m.silent = true;
            m
        },
    };
    let baked = icu_segmenter::provider::Baked;
    if attribute == "cjdict" {
        let r: DataResponse<SegmenterDictionaryAutoV2> = baked.load(request).ok()?;
        Some(encode(r.payload.get()))
    } else {
        let r: DataResponse<SegmenterDictionaryExtendedV2> = baked.load(request).ok()?;
        Some(encode(r.payload.get()))
    }
}

/// Installs dictionary `attribute` (one of [`DICTIONARIES`]) from the bytes
/// the cook wrote ([`compiled`]): every segmenter made afterwards uses it.
/// The bytes stay for the process, as a game's dictionaries do.
pub fn install(attribute: &str, bytes: &[u8]) -> Result<(), String> {
    let Some(&(attr, _)) = DICTIONARIES.iter().find(|(a, _)| *a == attribute) else {
        return Err(format!("'{attribute}' is not a segmentation dictionary"));
    };
    let mut installed = INSTALLED.write().map_err(|e| e.to_string())?;
    if installed.dictionaries.contains_key(attr) {
        return Ok(());
    }
    let data = decode(Box::leak(bytes.into()))?;
    installed
        .dictionaries
        .insert(attr, Box::leak(Box::new(data)));
    installed.word = None;
    installed.lines.clear();
    Ok(())
}

/// Whether dictionary `attribute` is installed.
pub fn installed(attribute: &str) -> bool {
    INSTALLED
        .read()
        .is_ok_and(|i| i.dictionaries.contains_key(attribute))
}

/// The installed dictionaries, as ICU's data provider asks for them.
struct Provider<'a>(&'a BTreeMap<&'static str, &'static DictionaryBreakData<'static>>);

impl Provider<'_> {
    fn get(
        &self,
        req: DataRequest<'_>,
    ) -> Result<&'static DictionaryBreakData<'static>, DataError> {
        self.0
            .get(req.id.marker_attributes.as_str())
            .copied()
            .ok_or(DataErrorKind::IdentifierNotFound.into_error())
    }
}

impl DataProvider<SegmenterDictionaryAutoV2> for Provider<'_> {
    fn load(&self, req: DataRequest) -> Result<DataResponse<SegmenterDictionaryAutoV2>, DataError> {
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_static_ref(self.get(req)?),
        })
    }
}

impl DataProvider<SegmenterDictionaryExtendedV2> for Provider<'_> {
    fn load(
        &self,
        req: DataRequest,
    ) -> Result<DataResponse<SegmenterDictionaryExtendedV2>, DataError> {
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_static_ref(self.get(req)?),
        })
    }
}

/// The word segmenter (UAX #29) with the installed dictionaries.
pub fn word() -> WordSegmenterBorrowed<'static> {
    let plain = || WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default());
    if let Ok(i) = INSTALLED.read() {
        if i.dictionaries.is_empty() {
            return plain();
        }
        if let Some(w) = i.word {
            return w;
        }
    }
    let Ok(mut i) = INSTALLED.write() else {
        return plain();
    };
    let mut s = plain().static_to_owned();
    // A dictionary the provider lacks is skipped; it never fails otherwise.
    let _ = s.load_dictionary_unstable(&Provider(&i.dictionaries));
    let w = Box::leak(Box::new(s)).as_borrowed();
    i.word = Some(w);
    w
}

/// The line segmenter (UAX #14) for `options` with the installed
/// dictionaries.
pub fn line(options: LineBreakOptions<'_>) -> LineSegmenterBorrowed<'static> {
    let key: LineKey = (
        options.strictness,
        options.word_option,
        options.content_locale.cloned(),
    );
    if let Ok(i) = INSTALLED.read() {
        if i.dictionaries.is_empty() {
            return LineSegmenter::new_for_non_complex_scripts(options);
        }
        if let Some((_, l)) = i.lines.iter().find(|(k, _)| *k == key) {
            return *l;
        }
    }
    let Ok(mut i) = INSTALLED.write() else {
        return LineSegmenter::new_for_non_complex_scripts(options);
    };
    let mut s = LineSegmenter::new_for_non_complex_scripts(options).static_to_owned();
    let _ = s.load_dictionary_unstable(&Provider(&i.dictionaries));
    let l = Box::leak(Box::new(s)).as_borrowed();
    i.lines.push((key, l));
    l
}

/// Whether the installed dictionaries cut `script`'s words (Parley applies
/// UAX #29's WB999 to the runs of a script none covers).
pub fn has_word_dictionary(script: Script) -> bool {
    let attr = match script {
        Script::Han | Script::Hiragana => "cjdict",
        Script::Thai => "thaidict",
        Script::Lao => "laodict",
        Script::Khmer => "khmerdict",
        Script::Myanmar => "burmesedict",
        _ => return false,
    };
    installed(attr)
}
