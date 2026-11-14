// og.i18n on the native host over the UI kit's cooked string tables
// (tests/fixtures/ui-text/assets/strings: en, zh-CN, ja, ko, ar): the host's
// locale picks the table (ja-JP takes ja), t formats placeholders and
// plurals in each language's rules, a key a locale lacks comes from the
// base, a key no table has answers itself with one warning, and the active
// locale sets the UI's font chain and direction (Arabic: right to left).
import { expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { cookProject, ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "ui-text");

// At module level, not in a hook: a hook's 5 s would not cover a cook on a
// loaded machine (b2532cd's gate).
{
  loadNativeHost();
  // The kit's tables and fonts, cooked (incrementally) by the og this package built.
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

function start(locale?: string): Engine {
  const r = createEngine(og.game({}) as GameToken, { root: kit, ...(locale ? { locale } : {}) });
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type Info = { requested: string; active: string; direction: string; locales: string[]; fonts: string[] };

test("og.i18n: the host's locale picks the table, and t formats each language's plurals", () => {
  const e = start("ja-JP");
  const w = e.world;
  for (const l of ["en", "zh-CN", "ja", "ko", "ar"]) {
    const r = og.i18n.load(w, `strings/${l}.ogt`);
    if (!r.ok) throw new Error(r.reason);
  }
  const info = og.i18n.locale(w) as unknown as Info;
  expect([info.requested, info.active, info.direction]).toEqual(["ja-JP", "ja", "ltr"]);
  expect(info.locales).toEqual(["ar", "en", "ja", "ko", "zh-CN"]);
  expect(info.fonts[0]).toBe("fonts/NotoSansJP-subset.otf");
  expect(info.fonts.length).toBe(4);
  expect(og.i18n.t(w, "menu.play")).toBe("プレイ");
  expect(og.i18n.t(w, "confirm.message", { slot: 2 })).toBe("スロット[b]2[/b]の進行状況は失われます。[color=danger]元に戻すことはできません。[/color]");
  expect(og.i18n.t(w, "hud.coins", { n: 1 })).toBe("コイン1枚");
  expect(og.i18n.has(w, "menu.play")).toBeTrue();

  const en = og.i18n.setLocale(w, "en-GB") as unknown as Info;
  expect(en.active).toBe("en");
  expect([og.i18n.t(w, "hud.coins", { n: 1 }), og.i18n.t(w, "hud.coins", { n: 1250 })]).toEqual(["1 coin", "1,250 coins"]);

  const ar = og.i18n.setLocale(w, "ar-EG") as unknown as Info;
  expect([ar.active, ar.direction, ar.fonts[0]]).toEqual(["ar", "rtl", "fonts/NotoSansArabic-subset.ttf"]);
  expect([0, 1, 2, 5, 11, 100].map((n) => og.i18n.t(w, "hud.coins", { n }))).toEqual(["لا عملات", "عملة واحدة", "عملتان", "5 عملات", "11 عملة", "100 عملة"]);

  // zh-TW has no table of its own: the base answers.
  expect((og.i18n.setLocale(w, "fr") as unknown as Info).active).toBe("en");
  expect(og.i18n.t(w, "no.such.key")).toBe("no.such.key");
  expect(og.i18n.t(w, "no.such.key")).toBe("no.such.key");
  const warned = e.takeLog().filter((l) => l.text.includes("no table has 'no.such.key'"));
  expect(warned.length).toBe(1);
});

test("og.i18n: refusals name the fix", () => {
  const e = start();
  const r = og.i18n.load(e.world, "strings/none.ogt");
  expect(r.ok).toBeFalse();
  const info = og.i18n.locale(e.world) as unknown as Info;
  expect([info.requested, info.active, info.locales.length]).toEqual(["", "", 0]);
});

test("the kit's zh-CN table is simplified Chinese: every Han character is in GB2312", () => {
  // Wrong: a traditional form (級, 選, 體, 設, 獲) in the simplified table; GB2312 holds the simplified set and none of those.
  const gbk = new TextDecoder("gbk");
  const gb2312 = new Set<string>();
  for (let row = 0xb0; row <= 0xf7; row++) for (let col = 0xa1; col <= 0xfe; col++) gb2312.add(gbk.decode(new Uint8Array([row, col])));
  expect(gb2312.has("级") && !gb2312.has("級")).toBeTrue();
  const table = readFileSync(join(kit, "assets", "strings", "zh-CN.ogt"), "utf8");
  const outside = [...table.matchAll(/^string "([^"]*)" "(.*)"$/gm)].flatMap(([, key, text]) =>
    [...text!].filter((ch) => /\p{Script=Han}/u.test(ch) && !gb2312.has(ch)).map((ch) => `${key}: ${ch}`));
  expect(outside).toEqual([]);
});

test("og.i18n: format functions follow the active locale, else the requested one, with moments the game passes in", () => {
  // No table loaded: the host's request (de-DE) decides.
  const de = start("de-DE").world;
  expect(og.i18n.formatNumber(de, 1234.5, { decimals: 2 })).toBe("1.234,50");
  expect(og.i18n.formatNumber(de, -0.001, { decimals: 2, grouping: false })).toBe("0,00");
  expect(og.i18n.formatCompact(de, 1234567)).toBe("1,2\u00a0Mio.");
  expect(og.i18n.formatCurrency(de, 1234.5, "EUR")).toBe("1.234,50\u00a0€");
  // 2026-11-02 15:05:09 UTC in milliseconds, as og.platform.now gives it; Tokyo is 540 minutes ahead.
  const t = 1793631909000;
  expect(og.i18n.formatDate(de, t)).toBe("02.11.2026");
  expect(og.i18n.formatDate(de, t, { style: "full" })).toBe("Montag, 2. November 2026");
  expect(og.i18n.formatTime(de, t, { offsetMinutes: 540 })).toBe("00:05");
  expect(og.i18n.formatDuration(de, 3723)).toBe("1:02:03");
  expect(og.i18n.formatDuration(de, 3723, "short")).toBe("2h");
  const bn = start("bn").world;
  expect(og.i18n.formatNumber(bn, 1234567.5, {})).toBe("১২,৩৪,৫৬৭.৫");
  expect(og.i18n.formatDuration(bn, 125)).toBe("২:০৫");
  // A loaded table's locale wins over the request.
  const w = start("de-DE").world;
  for (const l of ["en", "ja"]) {
    const r = og.i18n.load(w, `strings/${l}.ogt`);
    if (!r.ok) throw new Error(r.reason);
  }
  og.i18n.setLocale(w, "ja-JP");
  expect(og.i18n.formatCompact(w, 12345)).toBe("1.2万");
  expect(og.i18n.formatCompact(w, 12345, 2)).toBe("1.23万");
  expect(og.i18n.formatDate(w, t, { style: "monthYear" })).toBe("2026年11月");
  expect(og.i18n.formatCurrency(w, 1234.6, "JPY")).toBe("￥1,235");
  expect(og.i18n.formatDate(w, t, { style: "long" })).toBe("2026年11月2日");
  og.i18n.setLocale(w, "en-US");
  expect(og.i18n.formatTime(w, t, { style: "medium" })).toBe("3:05:09\u202fPM");
  og.i18n.setLocale(w, "en-XA");
  // English translations are loaded, but must not erase pseudo formatting.
  expect(og.i18n.formatNumber(w, 1234.5, {})).toBe("[1,234.5~~~]");
  expect(og.i18n.formatDuration(w, 125)).toBe("[2:05~~]");
});

test("og.i18n: a document's English base takes over from a translation loaded before it, and the UI's direction follows", () => {
  // A game that loads its Arabic table before its documents, with no locale
  // asked for: Arabic is active (the only table) until the documents' English
  // strings arrive as the base, which then is. Wrong: the UI kept the
  // Arabic table's right-to-left direction (and fonts) over English text.
  const root = mkdtempSync(join(tmpdir(), "og-i18n-base-"));
  try {
    mkdirSync(join(root, "assets/ui"), { recursive: true });
    mkdirSync(join(root, "assets/strings"), { recursive: true });
    writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Base"\nid "base"\nentry "main.ts"\n');
    writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
    writeFileSync(join(root, "assets/ui/card.svelte"), '<meta name="component" content="Card">\n<meta name="strings" content="./card.en.ogt">\n<div id="Row"><span id="A">{t("card.title")}</span><span id="B">b</span></div>\n<style>#Row { display: flex; width: stretch; }</style>\n');
    writeFileSync(join(root, "assets/ui/card.en.ogt"), 'type strings\nversion 1\n---\nlocale "en"\nstring "card.title" "Title"\n');
    writeFileSync(join(root, "assets/strings/ar.ogt"), 'type strings\nversion 1\n---\nlocale "ar"\nstring "card.title" "العنوان"\n');
    cookProject(root, root);
    const seen: string[] = [];
    const run = (locale?: string) => {
      const r = createEngine(og.game({
        start(world) {
          const ar = og.i18n.load(world, "strings/ar.ogt");
          if (!ar.ok) throw new Error(ar.reason);
          const doc = og.load.document(world, "ui/card.svelte");
          if (!doc.ok) throw new Error(doc.reason);
          const info = og.i18n.locale(world) as unknown as Info;
          seen.push(`${info.active} ${info.direction} ${og.i18n.t(world, "card.title")}`);
        },
        ui(world) {
          const mounted = og.ui.scope(world, "t").mount("card", "Card", {});
          if (!mounted.ok) throw new Error(mounted.reason);
        },
      }) as GameToken, { root, ...(locale ? { locale } : {}) });
      if (!r.ok) throw new Error(r.reason);
      r.value.advance(3);
      // The row runs from the right under a right-to-left UI.
      const els = (r.value.probe.dump({ what: "ui" }) as unknown as { elements: { path: string; x: number }[] }).elements;
      const x = (id: string) => els.find((e) => e.path.endsWith(`/${id}`))!.x;
      seen.push(x("A") < x("B") ? "ltr" : "rtl");
      r.value.destroy();
    };
    run();
    run("ar-EG");
    expect(seen).toEqual(["en ltr Title", "ltr", "ar rtl العنوان", "rtl"]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
