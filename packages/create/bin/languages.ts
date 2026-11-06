/** The writing system a language tag is set in, as far as the shipped faces go. */
export type LocaleScript = "ja" | "ko" | "hant" | "arabic" | "hebrew" | "cyrillic" | "hans" | "latin" | "other";

const ARABIC = new Set(["ar", "fa", "ur", "ps", "sd", "ug", "ckb"]);
const HEBREW = new Set(["he", "iw", "yi"]);
const CYRILLIC = new Set(["ru", "uk", "be", "bg", "sr", "mk", "kk", "ky", "mn", "tg", "el"]);
const LATIN = new Set("en es fr de it pt nl sv da no nb nn fi is pl cs sk sl hr bs ro hu tr az vi id ms tl fil sw et lv lt sq mt ga cy eu ca gl af ha yo ig zu xh uz tk la eo".split(" "));

/** `zh_cn` -> `zh-CN`, `zh-hant` -> `zh-Hant` (og-i18n's canonical form). */
export function canonicalTag(tag: string): string {
  return tag.trim().split(/[-_]/).filter(Boolean).map((p, k) => (k === 0 ? p.toLowerCase() : p.length === 4 ? p[0]!.toUpperCase() + p.slice(1).toLowerCase() : p.toUpperCase())).join("-");
}

const languageOf = (tag: string) => canonicalTag(tag).split("-")[0]!;
const hant = (tag: string) => /-(Hant|TW|HK|MO)(-|$)/.test(canonicalTag(tag));

export function localeScript(tag: string): LocaleScript {
  const lang = languageOf(tag);
  if (lang === "zh") return hant(tag) ? "hant" : "hans";
  if (lang === "ja" || lang === "ko") return lang;
  if (ARABIC.has(lang)) return "arabic";
  if (HEBREW.has(lang)) return "hebrew";
  if (CYRILLIC.has(lang) && !/-Latn/.test(canonicalTag(tag))) return "cyrillic";
  if (LATIN.has(lang) || /-Latn/.test(canonicalTag(tag))) return "latin";
  return "other";
}

/** Right to left (og-i18n's `direction_of_locale`). */
export const isRtl = (tag: string) => ["ar", "he", "iw", "fa", "ur", "ps", "sd", "yi", "dv", "ug", "ckb"].includes(languageOf(tag));
