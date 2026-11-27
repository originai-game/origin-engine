//! Native string-table payloads; the ICU grammar belongs to localization,
//! while reflected layouts belong to the cooked format crate.
use crate::{Direction, StringTable};
use og_bundle::format::{self, StringEntryRow, StringFontRow};
use og_bundle::runtime::{Bundle, LoadError, invalid, one, table};

pub trait BundleStrings {
    fn strings(&self, id: &str, bytes: &[u8]) -> Result<StringTable, LoadError>;
}

impl BundleStrings for Bundle {
    fn strings(&self, id: &str, bytes: &[u8]) -> Result<StringTable, LoadError> {
        let blob = self.blob::<format::Strings>(id, bytes)?;
        let meta = one::<format::StringsMeta>(&blob)?;
        let bytes = blob.raw("text")?;
        let text = |offset: u32, len: u32| {
            (offset as usize)
                .checked_add(len as usize)
                .and_then(|end| bytes.get(offset as usize..end))
                .and_then(|s| std::str::from_utf8(s).ok())
                .map(str::to_string)
                .ok_or_else(|| invalid(&blob, "string-table text is out of range or not UTF-8"))
        };
        let mut entries = std::collections::BTreeMap::new();
        for entry in table::<StringEntryRow>(&blob, "entries")? {
            let key = text(entry.key_offset, entry.key_len)?;
            if entries
                .insert(key, text(entry.message_offset, entry.message_len)?)
                .is_some()
            {
                return Err(invalid(&blob, "duplicate string-table key"));
            }
        }
        let fonts = table::<StringFontRow>(&blob, "fonts")?
            .iter()
            .map(|font| text(font.offset, font.len))
            .collect::<Result<_, _>>()?;
        let fallback = text(meta.fallback_offset, meta.fallback_len)?;
        let direction = match meta.direction {
            0 => Direction::Ltr,
            1 => Direction::Rtl,
            _ => return Err(invalid(&blob, "invalid string-table direction")),
        };
        let base = match meta.base {
            0 => false,
            1 => true,
            _ => return Err(invalid(&blob, "invalid string-table base flag")),
        };
        StringTable::new(
            text(meta.locale_offset, meta.locale_len)?,
            (!fallback.is_empty()).then_some(fallback),
            base,
            Some(direction),
            fonts,
            entries,
        )
        .map_err(|e| invalid(&blob, e))
    }
}
