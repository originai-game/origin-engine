// Every language in the kit's lists (the settings screen's, src/main.ts, and
// the gallery's select, src/controls.ts) is named in its own script (한국어,
// 繁體中文, Русский, العربية) whatever the locale, so every locale font that
// can draw a name carries its characters: the Japanese and Korean subsets
// cook every CJK name's characters (Noto Sans CJK's faces have them all),
// the Arabic subset its script, and the Chinese body face's import the
// Cyrillic it draws Русский with. tools/make-locale-fonts.py reads the lists
// from the sources; before it did, the Korean subset lacked 국 and
// the Language tab drew 한국어 with a hole (the settings-saved playtest's
// missing-glyph warning). Read from the sources, so it holds with or without
// a renderer: the missing-glyph warning is only raised by a drawn frame.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { playtest } from "@origingame/engine/playtest";

// The bundled playtest runs from the kit's og-out/test/<name>/.
const KIT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

/** A source's language names: its `const LANGUAGES = [...]`, each name matched by `name`. */
function names(file: string, name: RegExp): string[] {
  const block = /^const LANGUAGES = \[(.*?)\];/ms.exec(readFileSync(join(KIT, "src", file), "utf8"));
  return block ? [...block[1]!.matchAll(name)].map((m) => m[1]!) : [];
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
    const settings = names("main.ts", /label: "([^"]*)"/g);
    const gallery = names("controls.ts", /"([^"]*)"/g);
    t.ok(settings.length >= 5 && settings.includes("한국어"), `the settings screen's list is read from src/main.ts (${settings.join(", ")})`);
    t.ok(gallery.includes("繁體中文") && gallery.includes("Русский"), `the gallery's select is read from src/controls.ts (${gallery.join(", ")})`);
    const list = [...settings, ...gallery];
    const wanted = [...new Set(list.join(""))].filter(cjk);
    for (const font of ["NotoSansJP-subset.otf", "NotoSansKR-subset.otf"]) {
      const { chars } = cooked(font);
      const missing = wanted.filter((c) => !chars.has(c));
      t.equal(missing.join(""), "", `${font} cooks every CJK language name's characters`);
    }
    const arabic = cooked("NotoSansArabic-subset.ttf");
    const letters = [...new Set(list.join(""))].filter((c) => /\p{Script=Arabic}/u.test(c));
    t.ok(letters.length > 0 && arabic.charset.includes("arabic"), "NotoSansArabic-subset.ttf cooks the Arabic block, العربية with it");
    const sc = cooked("NotoSansSC-Regular-subset.ttf");
    const cyrillic = [...new Set(list.join(""))].filter((c) => /\p{Script=Cyrillic}/u.test(c));
    t.equal(cyrillic.filter((c) => !sc.chars.has(c)).join(""), "", "the Chinese body face's import cooks the Cyrillic of Русский");
  },
});
