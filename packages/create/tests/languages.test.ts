// The language helpers (bin/languages.ts, bin/pseudo-locale.ts) mirror
// og-i18n, which create cannot import: a bun test never runs cargo, so the
// data the two sides share is pinned here against the Rust sources, in the
// read-the-source pattern of the knowledge tests. A list changed on one
// side fails here before it mislabels a language.

import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { canonicalTag, isRtl } from "../bin/languages.ts";
import { ACCENTS, pseudoMessage } from "../bin/pseudo-locale.ts";

const localeRs = readFileSync(join(import.meta.dir, "..", "..", "..", "crates", "og-i18n", "src", "locale.rs"), "utf8");
const formatRs = readFileSync(join(import.meta.dir, "..", "..", "..", "crates", "og-i18n", "src", "format.rs"), "utf8");

test("isRtl is og-i18n's direction_of_locale", () => {
  const arm = /match language\(tag\)\.as_str\(\)\s*\{([^}]*)\}/.exec(localeRs)![1]!;
  const languages = [...arm.matchAll(/"([a-z]+)"/g)].map((m) => m[1]!);
  expect(languages.length).toBeGreaterThan(5);
  expect(languages).toEqual(["ar", "he", "iw", "fa", "ur", "ps", "sd", "yi", "dv", "ug", "ckb"]);
  for (const lang of languages) expect(isRtl(lang)).toBe(true);
  expect(isRtl("en")).toBe(false);
  expect(isRtl("en-GB")).toBe(false);
});

test("canonicalTag is og-i18n's canonical", () => {
  // The cases canonical's doc comment states.
  expect(canonicalTag("zh_cn")).toBe("zh-CN");
  expect(canonicalTag("ZH-cn")).toBe("zh-CN");
  expect(canonicalTag("zh-hant")).toBe("zh-Hant");
  expect(canonicalTag("EN")).toBe("en");
});

test("languages.ts's hant regex is og-i18n's chinese_script", () => {
  const fn = /pub\(crate\) fn chinese_script[\s\S]*?\n}/.exec(localeRs)![0]!;
  const markers = [...fn.matchAll(/"(-[A-Za-z]+)"/g)].map((m) => m[1]!.slice(1));
  expect(markers.length).toBeGreaterThan(2);
  const ts = readFileSync(join(import.meta.dir, "..", "bin", "languages.ts"), "utf8");
  const re = /\/-\(([^)]+)\)\(-\|\$\)\//.exec(ts)![1]!;
  expect(re.split("|")).toEqual(markers);
});

test("pseudo-locale's accents are og-i18n's accent()", () => {
  const fn = /fn accent\(c: char\) -> char \{[\s\S]*?\n\}/.exec(formatRs)![0]!;
  const pairs = [...fn.matchAll(/'([a-zA-Z])' => '([^'])',/g)].map((m) => [m[1]!, m[2]!] as const);
  expect(pairs.length).toBe(16);
  expect(Object.entries(ACCENTS)).toEqual(pairs);
  expect(pseudoMessage("a é n")).toBe("[á é ñ áé]");
});
