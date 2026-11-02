//! Localization for Origin Engine (`og.i18n`): string tables, one `.ogt`
//! per locale (`type strings`), messages in a subset of ICU MessageFormat,
//! CLDR cardinal plural rules, locale negotiation, text direction and
//! segmentation (UAX #29 words and grapheme clusters).
//!
//! ```text
//! type strings
//! version 1
//! ---
//! locale "ja"
//! fonts "fonts/NotoSansJP-subset.ttf"     # this locale's font chain, first first
//! string "menu.start" "スタート"
//! string "shop.coins" "{count, plural, other {コイン#枚}}"
//! string "hello" "こんにちは、{name}さん"
//! ```
//!
//! Messages: `{name}` (an argument), `{n, number}`, `{n, plural, =0 {..}
//! one {..} few {..} many {..} other {..}}` with `#` for the number,
//! `{g, select, a {..} other {..}}`, `''` for a quote and `'{'` to escape;
//! `{n, number, compact}`, `{n, number, currency/EUR}`, `{d, date, short}`
//! and `{t, time}` write from the CLDR subset in `format`.

// Generated (scripts/cldr.ts), one record per line: rustfmt leaves it as written.
#[rustfmt::skip]
mod cldr;
pub mod format;
pub mod locale;
pub mod message;
pub mod plural;
pub mod table;
pub mod text;

pub use format::{
    DateStyle, TimeStyle, format_compact, format_currency, format_date, format_duration,
    format_locales, format_time,
};
pub use locale::{Direction, direction_of_locale, negotiate};
pub use message::{Arg, Message, MessageError};
pub use plural::{PluralCategory, format_ordinal, plural_category};
pub use table::{Catalog, MissingKey, STRINGS_TYPE, StringTable, check_tables};
pub use text::{
    WordKind, WordSegment, graphemes, is_bidi_control, is_rtl_char, needs_shaping,
    paragraph_direction, word_per_character, word_segments,
};
