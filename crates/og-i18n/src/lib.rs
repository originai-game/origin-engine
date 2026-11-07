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
    DateStyle, TimeStyle, format_compact, format_currency, format_date, format_decimal,
    format_duration, format_locales, format_time, is_pseudo_locale,
};
pub use locale::{Direction, direction_of_locale, negotiate};
pub use message::{Arg, Message, MessageError};
pub use plural::{PluralCategory, format_ordinal, plural_category};
pub use table::{Catalog, MissingKey, STRINGS_TYPE, StringTable, base_table, check_tables};
pub use text::{
    WordKind, WordSegment, graphemes, is_bidi_control, is_rtl_char, needs_shaping,
    paragraph_direction, word_per_character, word_segments,
};

#[cfg(test)]
mod i18n_contracts {
    //! og-i18n through its public boundary.
    //!
    //! Ways it could fail, and the test that catches each:
    //! - a plural picks the wrong case for a language (Arabic's six, Russian's
    //!   few/many, Japanese's none), `=N` loses to a keyword, `#` ignores the
    //!   offset or the language's grouping → `plurals_pick_each_languages_case`
    //! - selects, nesting, numbers, quoting or a missing argument format wrongly
    //!   → `messages_format_selects_numbers_quotes_and_missing_arguments`
    //! - a malformed message or table is accepted, or refused without saying
    //!   where → `malformed_messages_and_tables_are_refused_with_where`
    //! - negotiation picks the wrong table (ja-JP, zh-TW against zh-CN and
    //!   zh-Hant, an unknown locale) → `locales_negotiate_to_the_best_table`
    //! - a key missing in a locale is not read from its fallback chain, or the
    //!   check misses a missing key, changed arguments or an extra key →
    //!   `the_catalog_falls_back_and_the_check_names_every_gap`
    //! - a right-to-left paragraph or locale is taken as left to right →
    //!   `directions_follow_the_first_strong_character_and_the_locale`

    use crate::{
        Arg, Catalog, Direction, Message, StringTable, check_tables, direction_of_locale,
        paragraph_direction, plural_category,
    };
    use std::collections::BTreeMap;

    fn args(pairs: &[(&str, Arg)]) -> BTreeMap<String, Arg> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn n(v: f64) -> Arg {
        Arg::Num(v)
    }

    #[test]
    fn plurals_pick_each_languages_case() {
        let ar = Message::parse(
        "{n, plural, zero {لا شيء} one {واحد} two {اثنان} few {# قليل} many {# كثير} other {# آخر}}",
    )
    .unwrap();
        let pick = |v: f64| ar.format("ar", &args(&[("n", n(v))]));
        assert_eq!(pick(0.0), "لا شيء");
        assert_eq!(pick(1.0), "واحد");
        assert_eq!(pick(2.0), "اثنان");
        assert_eq!(pick(5.0), "5 قليل");
        assert_eq!(pick(11.0), "11 كثير");
        assert_eq!(pick(100.0), "100 آخر");
        assert_eq!(pick(103.0), "103 قليل");
        let ru = |v: f64| plural_category("ru", v).keyword();
        assert_eq!(
            [
                ru(1.0),
                ru(2.0),
                ru(5.0),
                ru(11.0),
                ru(21.0),
                ru(22.0),
                ru(1.5)
            ],
            ["one", "few", "many", "many", "one", "few", "other"]
        );
        assert_eq!(plural_category("ja", 1.0).keyword(), "other");
        assert_eq!(plural_category("fr", 0.0).keyword(), "one");
        assert_eq!(plural_category("en", 1.0).keyword(), "one");
        assert_eq!(plural_category("en", 1.5).keyword(), "other");
        let en = Message::parse("{count, plural, offset:1 =0 {nobody} =1 {only {name}} one {{name} and # other} other {{name} and # others}}").unwrap();
        let f = |c: f64| {
            en.format(
                "en",
                &args(&[("count", n(c)), ("name", Arg::Str("Ada".into()))]),
            )
        };
        assert_eq!(f(0.0), "nobody");
        assert_eq!(f(1.0), "only Ada");
        assert_eq!(f(2.0), "Ada and 1 other");
        assert_eq!(f(1235.0), "Ada and 1,234 others");
        let de = Message::parse("{n, plural, one {# Münze} other {# Münzen}}").unwrap();
        assert_eq!(
            de.format("de", &args(&[("n", n(12500.0))])),
            "12.500 Münzen"
        );
    }

    #[test]
    fn messages_format_selects_numbers_quotes_and_missing_arguments() {
        let m = Message::parse(
            "{who, select, hero {{name} wins} other {they win}}: {score, number} pts",
        )
        .unwrap();
        assert_eq!(
            m.format(
                "en",
                &args(&[
                    ("who", Arg::Str("hero".into())),
                    ("name", Arg::Str("Kai".into())),
                    ("score", n(1234.5))
                ])
            ),
            "Kai wins: 1,234.5 pts"
        );
        assert_eq!(
            m.format(
                "en",
                &args(&[("who", Arg::Str("x".into())), ("score", n(3.0))])
            ),
            "they win: 3 pts"
        );
        assert_eq!(m.arguments(), vec!["name", "score", "who"]);
        let q = Message::parse("It''s '{literal}' and {x}").unwrap();
        assert_eq!(q.format("en", &args(&[])), "It's {literal} and {x}");
        let hash = Message::parse("# is plain outside a plural").unwrap();
        assert_eq!(hash.format("en", &args(&[])), "# is plain outside a plural");
        let fr = Message::parse("{n, number}").unwrap();
        assert_eq!(
            fr.format("fr", &args(&[("n", n(1234567.25))])),
            "1\u{202f}234\u{202f}567,25"
        );
    }

    #[test]
    fn messages_write_compact_numbers_currencies_dates_and_times() {
        let m = Message::parse(
        "{n, number, compact} · {p, number, currency/eur} · {d, date, short} {d, time, short} · {d, date}",
    )
    .unwrap();
        // 2026-11-02 15:05:09 UTC.
        let a = args(&[
            ("n", n(1_234_567.0)),
            ("p", n(1234.5)),
            ("d", n(1_793_631_909.0)),
        ]);
        assert_eq!(
            m.format("de", &a),
            "1,2\u{a0}Mio. · 1.234,50\u{a0}€ · 02.11.26 15:05 · 02.11.2026"
        );
        assert_eq!(
            m.format("en-GB", &a),
            "1.2M · €1,234.50 · 11/2/26 3:05\u{202f}PM · Nov 2, 2026"
        );
        // A table's full tag reaches the formatter: zh-TW writes Traditional.
        let c = Message::parse("{n, number, compact}").unwrap();
        assert_eq!(
            c.format("zh-TW", &args(&[("n", n(123_456_789.0))])),
            "1.2億"
        );
        assert_eq!(c.format("zh", &args(&[("n", n(123_456_789.0))])), "1.2亿");
        assert_eq!(m.arguments(), vec!["d", "n", "p"]);
        // A plain number keeps its old form.
        let plain = Message::parse("{n, number, integer}").unwrap();
        assert_eq!(plain.format("de", &args(&[("n", n(1234.0))])), "1.234");
    }

    #[test]
    fn malformed_messages_and_tables_are_refused_with_where() {
        let e = Message::parse("Hello {name").unwrap_err();
        assert!(e.reason.contains("needs '}'"), "{e}");
        let e = Message::parse("{n, plural, one {x}}").unwrap_err();
        assert!(e.reason.contains("needs an 'other' case"), "{e}");
        let e = Message::parse("{n, plural, lots {x} other {y}}").unwrap_err();
        assert!(e.reason.contains("'lots' is not a plural case"), "{e}");
        let e = Message::parse("{n, spellout}").unwrap_err();
        assert!(e.reason.contains("not an argument type"), "{e}");
        let e = Message::parse("{p, number, currency}").unwrap_err();
        assert!(e.reason.contains("needs its code"), "{e}");
        let e = Message::parse("{p, number, currency/EU}").unwrap_err();
        assert!(e.reason.contains("'EU' is not a currency code"), "{e}");
        let e = Message::parse("{d, date, tiny}").unwrap_err();
        assert!(e.reason.contains("'tiny' is not a date style"), "{e}");
        let e = Message::parse("{d, time, full}").unwrap_err();
        assert!(e.reason.contains("'full' is not a time style"), "{e}");
        let e = Message::parse("a } b").unwrap_err();
        assert_eq!(e.at, 2);
        let t = StringTable::parse(
            "type strings\nversion 1\n---\nlocale \"en\"\nstring \"a\" \"{x\"\n",
        )
        .unwrap_err();
        assert!(t.starts_with("line 5: 'a':"), "{t}");
        let t = StringTable::parse(
            "type strings\nversion 1\n---\nlocale \"en\"\nstring \"a\" \"x\"\nstring \"a\" \"y\"\n",
        )
        .unwrap_err();
        assert!(t.contains("'a' is defined twice"), "{t}");
        let t =
            StringTable::parse("type strings\nversion 1\n---\nstring \"a\" \"x\"\n").unwrap_err();
        assert!(t.contains("needs `locale"), "{t}");
        let t = StringTable::parse("type strings\nversion 1\n---\nlocale \"en\"\ncolour red\n")
            .unwrap_err();
        assert!(t.contains("'colour' is not a string table entry"), "{t}");
    }

    fn table(locale: &str, extra: &str, entries: &[(&str, &str)]) -> StringTable {
        let mut s = format!("type strings\nversion 1\n---\nlocale \"{locale}\"\n{extra}");
        for (k, v) in entries {
            s.push_str(&format!("string \"{k}\" \"{v}\"\n"));
        }
        StringTable::parse(&s).unwrap()
    }

    #[test]
    fn locales_negotiate_to_the_best_table() {
        let mut c = Catalog::default();
        for l in ["en", "ja", "zh-CN", "zh-Hant", "ar"] {
            c.insert(table(l, "", &[("k", l)]));
        }
        let active = |c: &mut Catalog, req: &[&str]| {
            c.request(req);
            c.active().unwrap().locale.clone()
        };
        assert_eq!(active(&mut c, &["ja-JP"]), "ja");
        assert_eq!(active(&mut c, &["zh-TW"]), "zh-Hant");
        assert_eq!(active(&mut c, &["zh_sg"]), "zh-CN");
        assert_eq!(active(&mut c, &["fr-FR", "ar-EG"]), "ar");
        assert_eq!(
            active(&mut c, &["sw"]),
            "en",
            "an unknown locale gets the base table"
        );
        assert_eq!(c.requested(), Some("sw"));
        c.request(&["ar-XB"]);
        assert_eq!(c.direction(), Direction::Rtl);
        c.request(&["en-XA"]);
        assert_eq!(c.direction(), Direction::Ltr);
    }

    #[test]
    fn the_catalog_falls_back_and_the_check_names_every_gap() {
        let en = table(
            "en",
            "",
            &[
                ("menu.start", "Start"),
                ("shop.coins", "{n, plural, one {# coin} other {# coins}}"),
                ("hello", "Hello, {name}"),
            ],
        );
        let ja = table(
            "ja",
            "fonts \"fonts/jp.ttf\" \"fonts/ui.ttf\"\n",
            &[
                ("menu.start", "スタート"),
                ("hello", "こんにちは、{who}"),
                ("ja.only", "x"),
            ],
        );
        assert_eq!(ja.fonts, vec!["fonts/jp.ttf", "fonts/ui.ttf"]);
        let gaps = check_tables(&[en.clone(), ja.clone()]);
        let named: Vec<(String, String, &str)> = gaps
            .iter()
            .map(|g| (g.locale.clone(), g.key.clone(), g.problem))
            .collect();
        assert_eq!(
            named,
            vec![
                ("ja".into(), "hello".into(), "arguments"),
                ("ja".into(), "shop.coins".into(), "missing"),
                ("ja".into(), "ja.only".into(), "extra"),
            ]
        );
        let mut c = Catalog::default();
        c.insert(en);
        c.insert(ja);
        c.request(&["ja"]);
        assert_eq!(c.t("menu.start", &args(&[])).unwrap(), "スタート");
        assert_eq!(
            c.t("shop.coins", &args(&[("n", n(3.0))])).unwrap(),
            "3 coins",
            "read from the base table"
        );
        assert!(!c.has("shop.coins"));
        assert_eq!(c.t("nope", &args(&[])), None);
    }

    #[test]
    fn directions_follow_the_first_strong_character_and_the_locale() {
        assert_eq!(direction_of_locale("ar-EG"), Direction::Rtl);
        assert_eq!(direction_of_locale("he"), Direction::Rtl);
        assert_eq!(direction_of_locale("ja"), Direction::Ltr);
        assert_eq!(
            paragraph_direction("123 مرحبا world", Direction::Ltr),
            Direction::Rtl
        );
        assert_eq!(
            paragraph_direction("Score: مرحبا", Direction::Rtl),
            Direction::Ltr
        );
        assert_eq!(paragraph_direction("123 !", Direction::Rtl), Direction::Rtl);
        let ar = table("ar", "", &[("k", "v")]);
        assert_eq!(ar.direction, Direction::Rtl);
        let forced = table("ar", "direction ltr\n", &[]);
        assert_eq!(forced.direction, Direction::Ltr);
    }

    /// UAX #29 words as word moves take them: punctuation runs merged, Han
    /// and Hiragana a word per character, Thai per grapheme cluster, marks
    /// with their base; grapheme clusters keep a base and its marks.
    #[test]
    fn words_and_graphemes_follow_uax_29() {
        use crate::{WordKind, graphemes, word_segments};
        let cut = |t: &str| -> Vec<String> {
            let chars: Vec<char> = t.chars().collect();
            word_segments(&chars)
                .iter()
                .map(|s| chars[s.start..s.end].iter().collect())
                .collect()
        };
        assert_eq!(
            cut("can't stop_now 3.14...?!"),
            ["can't", " ", "stop_now", " ", "3.14", "...?!"]
        );
        assert_eq!(cut("東京へ行く。"), ["東", "京", "へ", "行", "く", "。"]);
        assert_eq!(cut("Score 原点42"), ["Score", " ", "原", "点", "42"]);
        assert_eq!(cut("안녕하세요 세계"), ["안녕하세요", " ", "세계"]);
        assert_eq!(cut("ไทยง่าย"), ["ไ", "ท", "ย", "ง\u{e48}", "า", "ย"]);
        assert_eq!(cut("e\u{301}cole x"), ["e\u{301}cole", " ", "x"]);
        let chars: Vec<char> = "a\nb".chars().collect();
        let kinds: Vec<WordKind> = word_segments(&chars).iter().map(|s| s.kind).collect();
        assert_eq!(kinds, [WordKind::Word, WordKind::Break, WordKind::Word]);
        assert_eq!(graphemes("e\u{301}a"), [(0, 2), (2, 3)]);
        // A conjunct (consonant, virama, consonant) and its sign: one cluster (GB9c).
        assert_eq!(graphemes("नमस्ते"), [(0, 1), (1, 2), (2, 6)]);
    }

    #[test]
    fn display_uppercase_uses_locale_and_context() {
        use crate::text::uppercase;
        assert_eq!(uppercase("straße ﬃ", "de-DE"), "STRASSE FFI");
        assert_eq!(uppercase("izmir ı", "tr-TR"), "İZMİR I");
        assert_eq!(uppercase("izmir ı", "en"), "IZMIR I");
        assert_eq!(uppercase("άδικος", "el-GR"), "ΑΔΙΚΟΣ");
        assert_eq!(uppercase("άδικος", "en"), "ΆΔΙΚΟΣ");
        assert_eq!(uppercase("i\u{307}\u{301}", "lt"), "I\u{301}");
        assert_eq!(uppercase("i\u{307}\u{301}", "en"), "I\u{307}\u{301}");
        assert_eq!(uppercase("中文 42", "zh-Hant"), "中文 42");
        assert_eq!(uppercase("abc", "not_a_locale"), "ABC");
    }
}
