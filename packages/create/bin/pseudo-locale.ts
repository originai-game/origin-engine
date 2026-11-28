#!/usr/bin/env bun
// The pseudo-locale of a game's string tables (docs/ui.md §6.1):
// `content/strings/qps-ploc.ts`, every message of the base table with its
// Latin letters accented, 40 % longer and in brackets, so a run in it shows
// which words overflow their box and which never came from a table (they
// stay plain). Run this script from the engine distribution's create/bin/
// after the base table changes:
//
//   bun create/bin/pseudo-locale.ts [project]
//
// The tag is `qps-ploc`, a private-use language, so no player's locale ever
// negotiates to it: only `og run --locale qps-ploc` or a `locale:qps-ploc`
// step selects it. Message syntax survives: arguments, plural and select
// branches, `#`, quoted text and rich-text tags stay as written.

import { mkdirSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";
import { ogBin } from "../../engine/src/node/project.ts";

/** The pseudo-locale's tag. */
export const PSEUDO_LOCALE = "qps-ploc";

/** A native checked table returned by `og i18n tables --json`. */
export interface StringTable {
  readonly path: string;
  readonly locale: string;
  readonly base: boolean;
  readonly entries: Readonly<Record<string, string>>;
}

/** The pseudo-locale's accents, the same letters og-i18n's `accent` (`format.rs`). */
export const ACCENTS: Readonly<Record<string, string>> = {
  a: "á", e: "é", i: "í", o: "ö", u: "ü", c: "ç", n: "ñ", y: "ý", A: "Á", E: "É", I: "Í", O: "Ö", U: "Ü", C: "Ç", N: "Ñ", Y: "Ý",
};
/** Rich text tags (`[b]`, `[color=info]`): markup, not words. */
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
export function pseudoTable(base: StringTable): string {
  const entries = Object.fromEntries(Object.entries(base.entries).map(([key, text]) => [key, pseudoMessage(text)]));
  return `// Generated from ${base.locale}; rerun pseudo-locale.ts after editing the base.\nimport { file } from '@origingame/engine/author';\nexport default file.strings(${JSON.stringify({locale: PSEUDO_LOCALE, entries}, null, 2)});\n`;
}

/** The base table across project modules; never import author TS in Bun. */
export function baseTable(project: string): StringTable | null {
  const result = spawnSync(ogBin(), ["i18n", "tables", project, "--json"], {encoding: "utf8", maxBuffer: Infinity});
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr.trim() || `og i18n tables exited ${result.status}`);
  const output = result.stdout.split(/\r?\n/).filter(Boolean).map(line => JSON.parse(line) as {event?: string; tables?: StringTable[]});
  const objects = output.filter(value => value.event === undefined);
  if (objects.length !== 1 || !Array.isArray(objects[0]?.tables)) throw new Error("expected one og i18n tables object");
  const tables = objects[0].tables.filter(t => t.locale.toLowerCase() !== PSEUDO_LOCALE);
  return tables.find(t => t.base) ?? tables.find(t => t.locale === "en") ?? tables[0] ?? null;
}

if (import.meta.main) {
  const project = resolve(process.argv[2] ?? ".");
  const base = baseTable(project);
  if (!base) {
    console.error(`pseudo-locale: no string table in ${project}`);
    process.exit(1);
  }
  const dir = join(project, "content", "strings");
  mkdirSync(dir, {recursive: true});
  const out = join(dir, `${PSEUDO_LOCALE}.ts`);
  writeFileSync(out, pseudoTable(base));
  console.log(`pseudo-locale: ${base.path} -> ${out}`);
}
