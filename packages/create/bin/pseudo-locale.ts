#!/usr/bin/env bun
// The pseudo-locale of a game's string tables (docs/ui.md §15.3):
// `assets/strings/qps-ploc.ogt`, every message of the base table with its
// Latin letters accented, 40 % longer and in brackets, so a run in it shows
// which words overflow their box and which never came from a table (they
// stay plain). `og new --lang` writes it and copies this file into the game
// as `tools/pseudo-locale.ts`; run it again after the base table changes
// (tests/pseudo.og.ts fails until you do):
//
//   bun tools/pseudo-locale.ts [project]
//
// The tag is `qps-ploc`, a private-use language, so no player's locale ever
// negotiates to it: only `og run --locale qps-ploc` or a `locale:qps-ploc`
// step selects it. Message syntax survives: arguments, plural and select
// branches, `#`, quoted text and rich-text tags stay as written. This file
// imports nothing but node's own modules.

import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

/** The pseudo-locale's tag. */
export const PSEUDO_LOCALE = "qps-ploc";

/** One `type strings` table, as far as create and the playtests read it. */
export interface StringTableText {
  readonly locale: string;
  readonly base: boolean;
  /** Key and message source, in file order. */
  readonly strings: readonly (readonly [string, string])[];
}

const unescape = (s: string) => s.replace(/\\(.)/g, (_, c: string) => (c === "n" ? "\n" : c));
/** An .ogt string literal. */
export const ogtString = (s: string) => `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n")}"`;

/** Reads a table's locale, base flag and strings (comments and other lines skipped). */
export function parseStrings(text: string): StringTableText {
  let locale = "";
  let base = false;
  const strings: [string, string][] = [];
  for (const line of text.split("\n")) {
    const l = /^locale[ \t]+"([^"]*)"/.exec(line);
    if (l) locale = l[1]!;
    const b = /^base\b[ \t]*"?([a-z]*)"?/.exec(line);
    if (b) base = b[1] !== "false";
    const s = /^string[ \t]+"((?:[^"\\]|\\.)*)"[ \t]+"((?:[^"\\]|\\.)*)"[ \t]*$/.exec(line);
    if (s) strings.push([unescape(s[1]!), unescape(s[2]!)]);
  }
  return { locale, base, strings };
}

const ACCENTS: Readonly<Record<string, string>> = {
  a: "á", e: "é", i: "í", o: "ö", u: "ü", c: "ç", n: "ñ", y: "ý", A: "Á", E: "É", I: "Í", O: "Ö", U: "Ü", C: "Ç", N: "Ñ", Y: "Ý",
};
/** og.ui.richText's tags: markup, not words. */
const TAG = /^\[\/?(?:b|i|u|s|outline|color|size|font|icon|img|action|link|wave|shake|pulse)(?:=[^\][]*)?\]/;

/** The message with its words accented; `letters` collects the words' characters (not syntax or markup). */
function accent(src: string, letters: string[], inBranch: boolean): string {
  let out = "";
  let i = 0;
  while (i < src.length) {
    const c = src[i]!;
    if (c === "'") {
      if (src[i + 1] === "'") {
        out += "''";
        i += 2;
        continue;
      }
      if (/[{}#|]/.test(src[i + 1] ?? "")) {
        const end = src.indexOf("'", i + 1);
        const stop = end < 0 ? src.length : end + 1;
        out += src.slice(i, stop);
        i = stop;
        continue;
      }
    }
    if (c === "[") {
      const tag = TAG.exec(src.slice(i));
      if (tag) {
        out += tag[0];
        i += tag[0].length;
        continue;
      }
    }
    if (c === "{") {
      // An argument: copied as written, its plural or select branches accented.
      let depth = 0;
      let j = i;
      for (; j < src.length; j++) {
        if (src[j] === "{") depth++;
        else if (src[j] === "}" && --depth === 0) break;
      }
      const arg = src.slice(i + 1, j);
      const head = /^\s*[^,{}]+,\s*(plural|select|selectordinal)\s*,/.exec(arg);
      if (!head) out += `{${arg}}`;
      else {
        let body = "";
        let k = head[0].length;
        while (k < arg.length) {
          const open = arg.indexOf("{", k);
          if (open < 0) {
            body += arg.slice(k);
            break;
          }
          body += arg.slice(k, open);
          let d = 0;
          let m = open;
          for (; m < arg.length; m++) {
            if (arg[m] === "{") d++;
            else if (arg[m] === "}" && --d === 0) break;
          }
          body += `{${accent(arg.slice(open + 1, m), letters, true)}}`;
          k = m + 1;
        }
        out += `{${head[0]}${body}}`;
      }
      i = j + 1;
      continue;
    }
    if (inBranch && c === "#") {
      out += c;
      i++;
      continue;
    }
    letters.push(c);
    out += ACCENTS[c] ?? c;
    i++;
  }
  return out;
}

/**
 * A message in the pseudo-locale: `"Start"` -> `"[Stárt st]"`. A message
 * without letters (`"{score}"`, `"3:00"`) is data and stays as it is.
 */
export function pseudoMessage(src: string): string {
  const shown: string[] = [];
  const accented = accent(src, shown, false);
  const letters = shown.filter((c) => /\p{L}/u.test(c));
  if (letters.length === 0) return src;
  const extra = Math.max(1, Math.round(shown.length * 0.4));
  let pad = "";
  for (let k = 0; k < extra; k++) {
    const c = letters[k % letters.length]!;
    pad += (ACCENTS[c] ?? c).toLowerCase();
  }
  return `[${accented} ${pad}]`;
}

/** The pseudo-locale table of a base table's text. */
export function pseudoTable(base: string): string {
  const t = parseStrings(base);
  return [
    "type strings",
    "version 1",
    "---",
    `# The pseudo-locale: ${t.locale || "the base"} table's messages accented, 40 %`,
    "# longer and in brackets. Generated: run `bun tools/pseudo-locale.ts` after",
    "# the base table changes, never edit it by hand.",
    `locale ${ogtString(PSEUDO_LOCALE)}`,
    ...t.strings.map(([k, m]) => `string ${ogtString(k)} ${ogtString(pseudoMessage(m))}`),
  ].join("\n") + "\n";
}

/** The base table of a project's `assets/strings/`: the `base true` one, else `en`, else the first (og-i18n's rule). */
export function baseTableFile(project: string): string | null {
  const dir = join(project, "assets", "strings");
  if (!existsSync(dir)) return null;
  const files = readdirSync(dir).filter((f) => f.endsWith(".ogt") && f !== `${PSEUDO_LOCALE}.ogt`).sort();
  const tables = files.map((f) => ({ f, t: parseStrings(readFileSync(join(dir, f), "utf8")) }));
  const pick = tables.find((x) => x.t.base) ?? tables.find((x) => x.t.locale === "en") ?? tables[0];
  return pick ? join(dir, pick.f) : null;
}

if (import.meta.main) {
  const project = resolve(process.argv[2] ?? ".");
  const base = baseTableFile(project);
  if (!base) {
    console.error(`pseudo-locale: no string table under ${join(project, "assets", "strings")}`);
    process.exit(1);
  }
  const out = join(project, "assets", "strings", `${PSEUDO_LOCALE}.ogt`);
  writeFileSync(out, pseudoTable(readFileSync(base, "utf8")));
  console.log(`pseudo-locale: ${base} -> ${out}`);
}
