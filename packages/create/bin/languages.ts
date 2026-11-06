// `og new --lang` (A8): the chosen languages as og-i18n string tables, the
// face each script needs, the pseudo-locale and a playtest per language.
//
// For each chosen tag the game gets `assets/strings/<tag>.ogt`: the starter's
// own table when it has one for the language (its `en`, an archetype's
// `zh-CN`), else the starter's translation shipped under
// templates/locales/strings/<starter>/, else an empty table that the cook
// reports key by key until someone translates it (localization §2: never
// copied English). A script outside the style's Latin gets its face from
// templates/locales/fonts/ (Japanese, Korean, Traditional Chinese, Arabic,
// Hebrew, Cyrillic and Greek), named first in that table's `fonts` line,
// cooked with the table's characters and kept with its licence; Simplified
// Chinese is drawn by the UI style's `cjk` companion, which every built-in
// pack names. Arabic and Hebrew tables say `direction rtl`.
//
// The base table (what the cook checks every other against and what a player
// whose language is not offered gets) is the first chosen language with a
// complete table. An archetype keeps its English table as the base: its
// playtests read English words.

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogtString, parseStrings, PSEUDO_LOCALE, pseudoTable } from "./pseudo-locale.ts";
import type { StringTableText } from "./pseudo-locale.ts";

/** A face a script needs, from templates/locales/fonts/. */
export interface LocaleFace {
  readonly file: string;
  readonly licence: string;
  /** Charsets the cook bakes besides the table's characters. */
  readonly charset: readonly string[];
  /** Characters always cooked (an alphabet small enough to bake whole). */
  readonly alphabet: string;
  /** Characters of a table this face draws: the rest come from the style's face. */
  readonly draws: (c: string) => boolean;
}

const nonAscii = (c: string) => c.codePointAt(0)! > 0x7e;
const range = (a: number, b: number) => Array.from({ length: b - a + 1 }, (_, k) => String.fromCodePoint(a + k)).join("");

const FACES = {
  ja: { file: "NotoSansJP-Regular-subset.ttf", licence: "OFL-NotoSansJP.txt", charset: ["cjk-punct", "kana"], alphabet: "", draws: nonAscii },
  ko: { file: "NotoSansKR-Regular-subset.ttf", licence: "OFL-NotoSansKR.txt", charset: ["cjk-punct"], alphabet: "", draws: nonAscii },
  hant: { file: "NotoSansTC-Regular-subset.ttf", licence: "OFL-NotoSansTC.txt", charset: ["cjk-punct"], alphabet: "", draws: nonAscii },
  arabic: { file: "NotoSansArabic-Regular.ttf", licence: "OFL-NotoSansArabic.txt", charset: ["ascii", "arabic"], alphabet: "", draws: nonAscii },
  hebrew: { file: "NotoSansHebrew-Regular.ttf", licence: "OFL-NotoSansHebrew.txt", charset: ["ascii"], alphabet: range(0x5d0, 0x5ea) + "\u05be\u05f3\u05f4", draws: nonAscii },
  cyrillic: { file: "NotoSans-CyrillicGreek-subset.ttf", licence: "OFL-NotoSans.txt", charset: [], alphabet: range(0x410, 0x44f) + "Ёё", draws: (c) => /[\u0370-\u03ff\u0400-\u052f\u1f00-\u1fff]/u.test(c) },
} as const satisfies Record<string, LocaleFace>;

/** The writing system a language tag is set in, as far as the faces go. */
export type LocaleScript = keyof typeof FACES | "hans" | "latin" | "other";

const ARABIC = new Set(["ar", "fa", "ur", "ps", "sd", "ug", "ckb"]);
const HEBREW = new Set(["he", "iw", "yi"]);
const CYRILLIC = new Set(["ru", "uk", "be", "bg", "sr", "mk", "kk", "ky", "mn", "tg", "el"]);
/** Languages written in Latin letters: the style's face draws them (diacritics checked on the frame). */
const LATIN = new Set("en es fr de it pt nl sv da no nb nn fi is pl cs sk sl hr bs ro hu tr az vi id ms tl fil sw et lv lt sq mt ga cy eu ca gl af ha yo ig zu xh uz tk la eo".split(" "));
/** The Noto face a script outside the shipped ones needs (named in the note). */
const OTHER: Readonly<Record<string, string>> = { th: "Thai", hi: "Devanagari", mr: "Devanagari", ne: "Devanagari", bn: "Bengali", ta: "Tamil", te: "Telugu", kn: "Kannada", ml: "Malayalam", gu: "Gujarati", pa: "Gurmukhi", km: "Khmer", lo: "Lao", my: "Myanmar", si: "Sinhala", ka: "Georgian", hy: "Armenian", am: "Ethiopic" };

/** Endonyms for a settings screen's language list: each language named in itself, in every table. */
const ENDONYMS: Readonly<Record<string, string>> = {
  en: "English", "zh-CN": "简体中文", "zh-TW": "繁體中文", "zh-HK": "繁體中文（香港）", ja: "日本語", ko: "한국어", ar: "العربية", he: "עברית", fa: "فارسی",
  es: "Español", fr: "Français", de: "Deutsch", it: "Italiano", pt: "Português", "pt-BR": "Português (Brasil)", ru: "Русский", uk: "Українська",
  pl: "Polski", nl: "Nederlands", tr: "Türkçe", vi: "Tiếng Việt", id: "Bahasa Indonesia", th: "ไทย", hi: "हिन्दी", sv: "Svenska", el: "Ελληνικά",
};

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

/** The same language, Chinese by script: what og-i18n's negotiation would take. */
function sameLanguage(a: string, b: string): boolean {
  if (languageOf(a) !== languageOf(b)) return false;
  return languageOf(a) !== "zh" || hant(a) === hant(b);
}

/** templates/locales in the repository, ./locales in the published package (pack-template.ts copies it). */
function localesRoot(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const root of [join(here, "../../../templates/locales"), join(here, "../locales")]) {
    if (existsSync(join(root, "fonts/LICENSES.md"))) return root;
  }
  throw new Error("the locale faces and translations are missing; reinstall the engine dist");
}

/** How a starter loads its tables: one line in start, a LANGUAGES list, or nothing (creation.ts loads them). */
type Loader = { kind: "line"; files: string[] } | { kind: "list"; ui: string; main: string } | { kind: "none" };

function loaderOf(dir: string, entry: string): Loader {
  const main = join(dir, entry);
  const code = readFileSync(main, "utf8");
  // Every module beside the entry that loads the base table (a starter with two modes, as game-2d's courtyard, has a start each).
  const line = /^\s*must\(og\.i18n\.load\(world, "strings\/en\.ogt"\)\);$/m;
  if (line.test(code)) {
    const others = readdirSync(dirname(main)).filter((f) => f.endsWith(".ts")).map((f) => join(dirname(main), f)).filter((f) => f !== main && line.test(readFileSync(f, "utf8"))).sort();
    return { kind: "line", files: [main, ...others] };
  }
  // The LANGUAGES list lives in the module that draws the settings: ui.ts, or another beside the entry (hud.ts).
  const list = /^export const LANGUAGES = \[[\s\S]*?\] as const;$/m;
  const src = dirname(main);
  const candidates = ["ui.ts", ...readdirSync(src).filter((f) => f.endsWith(".ts") && f !== "ui.ts").sort()];
  const ui = candidates.map((f) => join(src, f)).find((f) => existsSync(f) && list.test(readFileSync(f, "utf8")));
  return ui ? { kind: "list", ui, main } : { kind: "none" };
}

interface Planned {
  readonly tag: string;
  /** The table's strings: the starter's, a shipped translation's, or none. */
  readonly strings: readonly (readonly [string, string])[];
  readonly from: "starter" | "translation" | "empty";
  /** The starter's file it replaces, if any. */
  readonly file?: string;
}

export interface AppliedLanguages {
  readonly files: string[];
  readonly notes: string[];
  /** Asset ids of every table, for a starter that loads none itself (creation.ts loads them). */
  readonly load: string[] | null;
}

/** Writes the tables, faces, pseudo-locale and playtests for `langs` into a scaffold. */
export function applyLanguages(dir: string, langs: readonly string[], ctx: { entry: string; starter: string; archetype: boolean }): AppliedLanguages {
  const root = localesRoot();
  const stringsDir = join(dir, "assets", "strings");
  const files: string[] = [];
  const notes: string[] = [];
  const own = existsSync(stringsDir)
    ? readdirSync(stringsDir).filter((f) => f.endsWith(".ogt")).sort().map((f) => ({ file: f, table: parseStrings(readFileSync(join(stringsDir, f), "utf8")) }))
    : [];
  const ownBase = own.find((t) => t.table.base) ?? own.find((t) => t.table.locale === "en") ?? own[0];
  const shipped = join(root, "strings", ctx.starter);
  const translations = existsSync(shipped) ? readdirSync(shipped).filter((f) => f.endsWith(".ogt")).sort().map((f) => parseStrings(readFileSync(join(shipped, f), "utf8"))) : [];

  // Each chosen tag's table, deduplicated by the tag it lands on.
  const plan: Planned[] = [];
  for (const asked of langs) {
    const tag = canonicalTag(asked) === "zh" ? "zh-CN" : canonicalTag(asked);
    const mine = own.find((t) => canonicalTag(t.table.locale) === tag) ?? own.find((t) => sameLanguage(t.table.locale, tag));
    const shippedOne = translations.find((t) => canonicalTag(t.locale) === tag) ?? translations.find((t) => sameLanguage(t.locale, tag));
    const next: Planned = mine
      ? { tag: canonicalTag(mine.table.locale), strings: mine.table.strings, from: "starter", file: mine.file }
      : shippedOne
        ? { tag: canonicalTag(shippedOne.locale), strings: shippedOne.strings, from: "translation" }
        : { tag, strings: [], from: "empty" };
    if (!plan.some((p) => p.tag === next.tag)) plan.push(next);
  }
  const loader = loaderOf(dir, ctx.entry);
  const listed = loader.kind === "list";
  // A listed starter's lang.* keys are the endonyms written below, never a translation's own.
  const baseKeys = new Set(ownBase?.table.strings.map(([k]) => k).filter((k) => !listed || !k.startsWith("lang.")) ?? []);
  const complete = (p: Planned) => [...baseKeys].every((k) => p.strings.some(([key]) => key === k));
  // The base: an archetype's English (its playtests read it), else the first complete chosen table.
  let base = ctx.archetype && ownBase ? plan.find((p) => p.file === ownBase.file) : plan.find(complete);
  if (!base && ownBase && baseKeys.size > 0) {
    base = { tag: canonicalTag(ownBase.table.locale), strings: ownBase.table.strings, from: "starter", file: ownBase.file };
    plan.unshift(base);
    notes.push(`${base.tag} stays as the base table (${ctx.archetype ? "the archetype's playtests read its words" : "no chosen language has every key yet"}); players whose language is not offered see it`);
  } else if (base && plan[0] !== base) {
    plan.splice(plan.indexOf(base), 1);
    plan.unshift(base);
  }
  base ??= plan[0]!;
  // The starter's reference frames were recorded in its own locale: its frame playtests pin it
  // (pinFrames, below), so its table stays even when it was not chosen (not the base, last).
  const framed = frameTests(dir);
  const refTag = ownBase ? canonicalTag(ownBase.table.locale) : null;
  if (framed.length > 0 && ownBase && refTag && !plan.some((p) => p.tag === refTag)) {
    plan.push({ tag: refTag, strings: ownBase.table.strings, from: "starter", file: ownBase.file });
    notes.push(`${refTag} stays as a table: ${framed.join(", ")} compare frames recorded in it and pin it (setup locale:${refTag}); record those references again in another locale and set their setup, then drop it`);
  }
  // A starter's own table its playtests name (a pause test that switches the settings to 简体中文) stays too, last.
  const testsDir = join(dir, "tests");
  const testFiles = existsSync(testsDir) ? readdirSync(testsDir).filter((f) => f.endsWith(".ts")).sort() : [];
  for (const t of own) {
    const tag = canonicalTag(t.table.locale);
    if (plan.some((p) => p.tag === tag)) continue;
    const named = testFiles.filter((f) => new RegExp(`(?<![A-Za-z0-9-])${tag.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}(?![A-Za-z0-9-])`).test(readFileSync(join(testsDir, f), "utf8")));
    if (named.length === 0) continue;
    plan.push({ tag, strings: t.table.strings, from: "starter", file: t.file });
    notes.push(`${tag} stays as a table: ${named.map((f) => `tests/${f}`).join(", ")} switch to it; change those playtests to a chosen language, then drop it`);
  }

  // Faces: one per script, with its licence and its row in LICENSES.md.
  const faces = new Map<string, LocaleFace>();
  for (const p of plan) {
    const script = localeScript(p.tag);
    if (script in FACES) faces.set(p.tag, FACES[script as keyof typeof FACES]);
    else if (script === "other") notes.push(`${p.tag}: no shipped face; bring Noto Sans ${OTHER[languageOf(p.tag)] ?? "for its script"} (notofonts, SIL OFL) into assets/fonts/, name it in the table's fonts line and record its licence (og-i18n §3)`);
    else if (script === "latin" && /^(vi|pl|cs|sk|ro|hu|tr|lt|lv|hr|sl|az)$/.test(languageOf(p.tag))) notes.push(`${p.tag}: check the UI style's face covers its diacritics (the log names each character a font lacks)`);
  }

  // The tables, written in plan order; the starter's tables nobody chose go.
  mkdirSync(stringsDir, { recursive: true });
  const lists = plan.map((p) => ({ code: p.tag, key: p.tag === "zh-CN" ? "lang.zh" : `lang.${p.tag.toLowerCase()}` }));
  const endonyms = listed ? lists.map((l) => [l.key, ENDONYMS[l.code] ?? ENDONYMS[languageOf(l.code)] ?? l.code] as const) : [];
  const written: { tag: string; file: string; text: StringTableText }[] = [];
  for (const p of plan) {
    const face = faces.get(p.tag);
    const strings = p.strings.filter(([k]) => !k.startsWith("lang.") || !listed);
    const body: string[] = [];
    if (p.file) {
      const text = readFileSync(join(stringsDir, p.file), "utf8");
      body.push(...text.split("\n").filter((l) => !/^(base|fonts|direction|locale|string)\b/.test(l) && l !== ""));
    } else {
      body.push("type strings", "version 1", "---");
      body.push(p.from === "translation" ? `# ${ENDONYMS[p.tag] ?? p.tag}: the starter's words translated (og-i18n §1).` : `# ${ENDONYMS[p.tag] ?? p.tag}: not translated yet. og cook warns for each key the base`, ...(p.from === "empty" ? ["# table has and this one lacks: add them here, translated, never copied (localization §2)."] : []));
    }
    body.push(`locale ${ogtString(p.tag)}`);
    if (p === base) body.push("base true");
    if (isRtl(p.tag)) body.push("direction rtl");
    if (face) body.push(`fonts ${ogtString(`fonts/${face.file}`)}`);
    for (const [k, m] of [...strings, ...endonyms]) body.push(`string ${ogtString(k)} ${ogtString(m)}`);
    const file = `${p.tag}.ogt`;
    const text = `${body.join("\n")}\n`;
    writeFileSync(join(stringsDir, file), text);
    if (p.file && p.file !== file) rmSync(join(stringsDir, p.file));
    files.push(`assets/strings/${file}`);
    written.push({ tag: p.tag, file, text: parseStrings(text) });
    if (p.from === "empty" && baseKeys.size > 0) notes.push(`assets/strings/${file} has none of the base table's ${baseKeys.size} keys: translate them (og cook warns for each until then)`);
  }
  for (const t of own) if (!plan.some((p) => p.file === t.file || `${p.tag}.ogt` === t.file)) rmSync(join(stringsDir, t.file));

  // Faces into assets/fonts/, each with the table's characters cooked and the rest drawn at run time.
  const fontsDir = join(dir, "assets", "fonts");
  const licences: string[] = [];
  for (const [tag, face] of faces) {
    mkdirSync(fontsDir, { recursive: true });
    if (!existsSync(join(fontsDir, face.file))) {
      cpSync(join(root, "fonts", face.file), join(fontsDir, face.file));
      cpSync(join(root, "fonts", face.licence), join(fontsDir, face.licence));
      files.push(`assets/fonts/${face.file}`, `assets/fonts/${face.licence}`);
      const row = readFileSync(join(root, "fonts", "LICENSES.md"), "utf8").split("\n").find((l) => l.startsWith(`| \`${face.file}\``));
      if (row) licences.push(row);
    }
    const table = written.find((w) => w.tag === tag)!.text;
    const chars = [...new Set([...face.alphabet, ...table.strings.flatMap(([, m]) => [...m]).filter(face.draws)])].sort().join("");
    writeFileSync(join(fontsDir, `${face.file}.ogt`), `type import
version 1
---
# The ${tag} table's face (og new --lang, og-i18n §3): the atlas cooks the
# characters the tables hold (\`chars\`; add new ones when strings change, or
# cut a subset with subset-font.py); any other character the face has is
# drawn from its outlines at run time, with a warning the first time.
font {
${face.charset.length ? `  charset ${face.charset.join(" ")}\n` : ""}  chars ${ogtString(chars)}
  em_px 48
  px_range 16
  runtime true
}
`);
    files.push(`assets/fonts/${face.file}.ogt`);
  }
  if (licences.length) {
    const file = join(fontsDir, "LICENSES.md");
    const head = "# Fonts: licences\n\n| File | Languages | Characters | Licence | Origin |\n|---|---|---|---|---|\n";
    writeFileSync(file, `${existsSync(file) ? readFileSync(file, "utf8").replace(/\n*$/, "\n\n") + head : head}${licences.join("\n")}\n`);
    if (!files.includes("assets/fonts/LICENSES.md")) files.push("assets/fonts/LICENSES.md");
  }

  // The pseudo-locale, from the base table (tools/pseudo-locale.ts's rule), and the tool that remakes it.
  const baseText = readFileSync(join(stringsDir, `${base.tag}.ogt`), "utf8");
  writeFileSync(join(stringsDir, `${PSEUDO_LOCALE}.ogt`), pseudoTable(baseText));
  mkdirSync(join(dir, "tools"), { recursive: true });
  cpSync(join(dirname(fileURLToPath(import.meta.url)), "pseudo-locale.ts"), join(dir, "tools", "pseudo-locale.ts"));
  files.push(`assets/strings/${PSEUDO_LOCALE}.ogt`, "tools/pseudo-locale.ts");

  // The starter loads every table (the pseudo-locale last, never in a list players pick from).
  const tags = [...plan.map((p) => p.tag), PSEUDO_LOCALE];
  let load: string[] | null = null;
  if (loader.kind === "line") for (const file of loader.files) {
    const code = readFileSync(file, "utf8");
    writeFileSync(file, code.replace(/^(\s*)must\(og\.i18n\.load\(world, "strings\/en\.ogt"\)\);$/m, (_, sp: string) => `${sp}// The chosen languages' tables (project.ogt's create block), then the pseudo-locale.\n${sp}for (const tag of ${JSON.stringify(tags)}) must(og.i18n.load(world, \`strings/\${tag}.ogt\`));`));
  } else if (loader.kind === "list") {
    const ui = readFileSync(loader.ui, "utf8");
    writeFileSync(loader.ui, ui.replace(/^export const LANGUAGES = \[[\s\S]*?\] as const;$/m, `export const LANGUAGES = [\n${lists.map((l) => `  { code: ${JSON.stringify(l.code)}, key: ${JSON.stringify(l.key)} },`).join("\n")}\n] as const;`));
    const main = readFileSync(loader.main, "utf8");
    writeFileSync(loader.main, main.replace(/^(\s*)(for \(const l of LANGUAGES\) must\(og\.i18n\.load\(world, `strings\/\$\{l\.code\}\.ogt`\)\);)$/m, `$1$2\n$1must(og.i18n.load(world, "strings/${PSEUDO_LOCALE}.ogt"));`));
  } else load = tags.map((t) => `strings/${t}.ogt`);

  // A playtest per language and one for the pseudo-locale.
  if (ownBase && ownBase.table.locale !== base.tag && !ctx.archetype) retarget(dir, ownBase.table, written.find((w) => w.tag === base!.tag)!.text, framed);
  if (refTag) pinFrames(dir, framed, refTag);
  files.push(...playtests(dir, written.map((w) => w.tag), base.tag));
  return { files, notes, load };
}

/**
 * The playtests that compare frames with references/ (they import the starter's
 * tests/frames.ts): their references show the starter's own locale.
 */
function frameTests(dir: string): string[] {
  const tests = join(dir, "tests");
  if (!existsSync(tests)) return [];
  return readdirSync(tests)
    .filter((f) => f.endsWith(".og.ts") && /from "\.\/frames\.ts"/.test(readFileSync(join(tests, f), "utf8")))
    .sort()
    .map((f) => `tests/${f}`);
}

/**
 * Each frame playtest boots in the locale its references were recorded in
 * (`setup: "locale:<tag>"`, before the first tick), whatever the base table
 * and the run's locale: the per-language playtests cover the others.
 */
function pinFrames(dir: string, framed: readonly string[], tag: string): void {
  const step = `locale:${tag}`;
  for (const f of framed) {
    const path = join(dir, f);
    const code = readFileSync(path, "utf8");
    if (code.includes(step)) continue;
    const setup = /^(\s*)setup: "([^"]*)",$/m;
    const next = setup.test(code)
      ? code.replace(setup, (_, sp: string, s: string) => `${sp}setup: "${step};${s}",`)
      : code.replace(/^(\s*)(name: .*,)$/m, (_, sp: string, n: string) => `${sp}${n}\n${sp}// Its references were recorded in ${tag}: it boots there whatever the base table (og new --lang).\n${sp}setup: "${step}",`);
    if (next === code) throw new Error(`${f}: no \`name:\` line to pin its locale after`);
    writeFileSync(path, next);
  }
}

/** The starter's playtests expect the old base's words: they now expect the new base's, but for the frame playtests, which stay in the references' locale. */
function retarget(dir: string, from: StringTableText, to: StringTableText, framed: readonly string[]): void {
  const tests = join(dir, "tests");
  if (!existsSync(tests)) return;
  const pairs = from.strings.flatMap(([k, m]) => {
    const now = to.strings.find(([key]) => key === k)?.[1];
    return now !== undefined && now !== m && !/[{}]/.test(m) ? [[m, now] as const] : [];
  });
  for (const f of readdirSync(tests).filter((f) => f.endsWith(".og.ts") && !framed.includes(`tests/${f}`))) {
    const path = join(tests, f);
    let code = readFileSync(path, "utf8");
    for (const [m, now] of pairs) {
      const esc = m.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      code = code.replace(new RegExp(`(expect:ui:text:=?)${esc}(?=[";])`, "g"), (_, p: string) => p + now);
    }
    writeFileSync(path, code);
  }
}

const HOST = `// The host the language playtests (tests/locales.ts) touch under bun, declared
// because the project type-checks with \`types: []\` (og new --lang).
declare module "node:fs" {
  export function readFileSync(path: string, encoding: string): string;
}
declare module "node:path" {
  export function dirname(path: string): string;
  export function resolve(...parts: string[]): string;
}
declare module "node:url" {
  export function fileURLToPath(url: string | URL): string;
}
declare class URL {
  constructor(url: string, base?: string | URL);
}
interface ImportMeta {
  readonly url: string;
}
`;

const HELPERS = `// The language playtests' checks (og new --lang): each locale-<tag>.og.ts
// switches to its locale and reads what the screen shows against the
// tables in assets/strings/; pseudo.og.ts does the same for the
// pseudo-locale. They read the tables as they are now, so they hold after
// strings change. A box (a character no font has) is a warning in the run.
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { PlaytestChecks, PlaytestRun } from "@origingame/engine/playtest";

/** A table's strings by key. */
export function table(tag: string): Map<string, string> {
  // The bundled playtest runs from og-out/test/<name>/.
  const project = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
  const text = readFileSync(resolve(project, "assets", "strings", tag + ".ogt"), "utf8");
  const out = new Map<string, string>();
  for (const line of text.split("\\n")) {
    const m = /^string[ \\t]+"((?:[^"\\\\]|\\\\.)*)"[ \\t]+"((?:[^"\\\\]|\\\\.)*)"/.exec(line);
    if (m) out.set(m[1]!.replace(/\\\\(.)/g, "$1"), m[2]!.replace(/\\\\(.)/g, "$1"));
  }
  return out;
}

/** A message drawn exactly as written: no argument, markup or quote. */
const plain = (m: string) => !/[{}'[\\]#]/.test(m);

/** What a dump_ui step shows. */
export function shown(run: PlaytestRun, file: string): Set<string> {
  return new Set(run.json<{ elements: { text: string }[] }>(file).elements.map((e) => e.text).filter((t) => t !== ""));
}

/** The commands that switch to a locale, then dump and capture the screen. */
export const commands = (tag: string) => \`0.3:locale:\${tag};0.9:dump_ui:locale-\${tag}.json;0.9:screenshot:locale-\${tag}.png;1.0:quit\`;

/**
 * In \`tag\`: no word of the base table is left where this table has its
 * own, and when the table translates anything, some of it is on screen.
 */
export function checkLocale(run: PlaytestRun, t: PlaytestChecks, tag: string, baseTag: string): void {
  t.ok(run.quit, "the script reached quit");
  t.equal(run.warnings.join("; "), "", "no warnings (a missing key or glyph is one)");
  const own = table(tag);
  const base = table(baseTag);
  const screen = shown(run, \`locale-\${tag}.json\`);
  const left = [...own].filter(([k, m]) => base.get(k) !== undefined && base.get(k) !== m && plain(base.get(k)!) && screen.has(base.get(k)!)).map(([k]) => k);
  t.equal(left.join(", "), "", \`no \${baseTag} word is left in \${tag}\`);
  const mine = [...own].filter(([k, m]) => plain(m) && (tag === baseTag || base.get(k) !== m)).map(([, m]) => m);
  if (mine.length > 0) t.ok(mine.some((m) => screen.has(m)), \`a word of the \${tag} table is on screen\`);
  t.frame(\`locale-\${tag}.png\`, () => t.ok(run.png(\`locale-\${tag}.png\`).width > 0, \`locale-\${tag}.png was drawn\`));
}

const ACCENTS: Record<string, string> = { a: "á", e: "é", i: "í", o: "ö", u: "ü", c: "ç", n: "ñ", y: "ý", A: "Á", E: "É", I: "Í", O: "Ö", U: "Ü", C: "Ç", N: "Ñ", Y: "Ý" };

/** The pseudo-locale is the base table's, as tools/pseudo-locale.ts makes it now, and it is what the screen shows. */
export function checkPseudo(run: PlaytestRun, t: PlaytestChecks, pseudoTag: string, baseTag: string): void {
  t.ok(run.quit, "the script reached quit");
  const pseudo = table(pseudoTag);
  const base = table(baseTag);
  const stale = [...base].filter(([k, m]) => {
    const p = pseudo.get(k);
    if (p === undefined) return true;
    return plain(m) && /\\p{L}/u.test(m) && !p.startsWith("[" + [...m].map((c) => ACCENTS[c] ?? c).join("") + " ");
  }).map(([k]) => k);
  t.equal(stale.join(", "), "", "the pseudo-locale matches the base table (else run bun tools/pseudo-locale.ts)");
  const screen = shown(run, \`locale-\${pseudoTag}.json\`);
  const literal = [...base].filter(([, m]) => plain(m) && /\\p{L}/u.test(m) && screen.has(m)).map(([k]) => k);
  t.equal(literal.join(", "), "", "every word on screen that is in the base table came through og.i18n");
  const words = [...pseudo.values()].filter((m) => m.startsWith("[") && plain(m.slice(1, -1)));
  if (words.length > 0) t.ok(words.some((m) => screen.has(m)), "pseudo-locale words are on screen");
  t.frame(\`locale-\${pseudoTag}.png\`, () => t.ok(run.png(\`locale-\${pseudoTag}.png\`).width > 0, \`locale-\${pseudoTag}.png was drawn\`));
}
`;

/** tests/locales.ts, tests/locale-<tag>.og.ts per language and tests/pseudo.og.ts. */
function playtests(dir: string, tags: readonly string[], baseTag: string): string[] {
  const tests = join(dir, "tests");
  mkdirSync(tests, { recursive: true });
  const out = ["tests/locales.ts"];
  writeFileSync(join(tests, "locales.ts"), HELPERS);
  if (!existsSync(join(dir, "host.d.ts"))) {
    writeFileSync(join(tests, "host.d.ts"), HOST);
    out.push("tests/host.d.ts");
  }
  for (const tag of tags) {
    const file = `locale-${tag}.og.ts`;
    writeFileSync(join(tests, file), `import { playtest } from "@origingame/engine/playtest";
import { checkLocale, commands } from "./locales.ts";
export default playtest({
  name: "the game in ${tag}${isRtl(tag) ? ", right to left" : ""}: its own words, no ${baseTag} left",
  commands: commands(${JSON.stringify(tag)}),
  check: (run, t) => checkLocale(run, t, ${JSON.stringify(tag)}, ${JSON.stringify(baseTag)}),
});
`);
    out.push(`tests/${file}`);
  }
  writeFileSync(join(tests, "pseudo.og.ts"), `import { playtest } from "@origingame/engine/playtest";
import { checkPseudo, commands } from "./locales.ts";
// The pseudo-locale (og-i18n §6): every word 40 % longer, accented and in
// brackets. Read the frame for cut and overlapping words; a plain word on
// screen is a literal that never went through og.i18n.
export default playtest({
  name: "the game in the pseudo-locale: every word from a table, none cut",
  commands: commands(${JSON.stringify(PSEUDO_LOCALE)}),
  check: (run, t) => checkPseudo(run, t, ${JSON.stringify(PSEUDO_LOCALE)}, ${JSON.stringify(baseTag)}),
});
`);
  out.push("tests/pseudo.og.ts");
  return out;
}
