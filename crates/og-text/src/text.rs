//! What the layout reads beside Parley: the characters no face need
//! draw, and the ICU dictionaries complex scripts break words with.

/// Characters no face need draw: joiners and variation selectors.
pub fn invisible(c: char) -> bool {
    matches!(c as u32, 0x200c..=0x200d | 0x2060 | 0xfe00..=0xfe0f | 0xe0100..=0xe01ef)
}

/// Installs ICU dictionary `attribute` (`cjdict`, `thaidict`, `laodict`,
/// `khmerdict` or `burmesedict`) from the bytes a font pack ships
/// (`og_i18n::segmenter`), and has Parley cut words and lines with the
/// installed dictionaries from then on. Text laid out before keeps its
/// lines until it is laid out again.
pub fn add_segmentation(attribute: &str, bytes: &[u8]) -> Result<(), String> {
    static SEGMENTERS: parley_engine::segmenters::Segmenters =
        parley_engine::segmenters::Segmenters {
            word: og_i18n::segmenter::word,
            line: og_i18n::segmenter::line,
            has_word_dictionary: og_i18n::segmenter::has_word_dictionary,
        };
    og_i18n::segmenter::install(attribute, bytes)?;
    parley_engine::segmenters::set(&SEGMENTERS);
    Ok(())
}
