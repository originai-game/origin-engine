// Every language in the settings screen's list is named in its own script
// (한국어, 日本語, 简体中文, العربية) whatever the locale, so every locale font
// that can draw a name carries its characters: the Japanese and Korean
// subsets cook every CJK name's characters (Noto Sans CJK's faces have them
// all), and the Arabic subset its script. tools/make-locale-fonts.py reads
// the list from src/main.ts; before it did, the Korean subset lacked 국 and
// the Language tab drew 한국어 with a hole (the settings-saved playtest's
// missing-glyph warning). Read from the sources, so it holds with or without
// a renderer: the missing-glyph warning is only raised by a drawn frame.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { playtest } from "@origingame/engine/playtest";

// The bundled playtest runs from the kit's og-out/test/<name>/.
const KIT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

/** The settings screen's language names (`const LANGUAGES = [...]`, `label: "…"`). */
function names(): string[] {
  const block = /^const LANGUAGES = \[(.*?)^\];/ms.exec(readFileSync(join(KIT, "src", "main.ts"), "utf8"));
  return block ? [...block[1]!.matchAll(/label: "([^"]*)"/g)].map((m) => m[1]!) : [];
}

/** A font import's cooked characters: its `chars` and its charset words. */
function cooked(font: string): { chars: Set<string>; charset: string[] } {
  const text = readFileSync(join(KIT, "assets", "fonts", `${font}.ogt`), "utf8");
  return {
    chars: new Set(/^\s*chars "(.*)"$/m.exec(text)?.[1] ?? ""),
    charset: (/^\s*charset (.*)$/m.exec(text)?.[1] ?? "").split(/\s+/),
  };
}

const cjk = (c: string): boolean => /[\p{Script=Han}\p{Script=Hangul}\p{Script=Hiragana}\p{Script=Katakana}]/u.test(c);

export default playtest({
  name: "every locale font carries the language list's names",
  commands: "0.1:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    const list = names();
    t.ok(list.length >= 5 && list.includes("한국어"), `the language list is read from src/main.ts (${list.join(", ")})`);
    const wanted = [...new Set(list.join(""))].filter(cjk);
    for (const font of ["NotoSansJP-subset.otf", "NotoSansKR-subset.otf"]) {
      const { chars } = cooked(font);
      const missing = wanted.filter((c) => !chars.has(c));
      t.equal(missing.join(""), "", `${font} cooks every CJK language name's characters`);
    }
    const arabic = cooked("NotoSansArabic-subset.ttf");
    const letters = [...new Set(list.join(""))].filter((c) => /\p{Script=Arabic}/u.test(c));
    t.ok(letters.length > 0 && arabic.charset.includes("arabic"), "NotoSansArabic-subset.ttf cooks the Arabic block, العربية with it");
  },
});
