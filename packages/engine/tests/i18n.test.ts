// og.i18n on the native host over the UI kit's cooked string tables
// (templates/ui-kit/assets/strings: en, zh-CN, ja, ko, ar): the host's
// locale picks the table (ja-JP takes ja), t formats placeholders and
// plurals in each language's rules, a key a locale lacks comes from the
// base, a key no table has answers itself with one warning, and the active
// locale sets the UI's font chain and direction (Arabic: right to left).
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "templates", "ui-kit");

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
