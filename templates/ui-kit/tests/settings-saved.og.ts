// The settings across a relaunch (store: "keep", og-platform-sdk §2 and §4):
// the first process mutes everything, picks 简体中文 on the Language tab and
// closes with Done, which writes og.settings; the second reads og.settings
// on its first tick and comes back muted and in Chinese, the main menu's
// words from the zh-CN table (examples/WEB-FINDINGS.md #10: the kit saved
// its settings and never read them, so a reload came back in English).
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, tap, ui } from "./lib.ts";

type Call = { method: string; args: Record<string, unknown>; tick: number };

export default playtest({
  name: "the settings saved on Done come back on a relaunch: muted, in Chinese",
  store: "keep",
  commands: [
    "0.3:show:settings",
    tap(0.7, "Mute"),
    tap(1.0, "Section/Tab#3"),
    tap(1.4, "Languages/Option#1"),
    tap(1.8, "Done"),
    "2.4:dump_entity:demo:saved.json",
    "2.4:dump_platform:platform.json",
    "2.5:quit",
  ].join(";"),
  then: [
    {
      commands: ["0.5:dump_entity:demo:loaded.json", "0.6:show:mainMenu", "1.3:dump_ui:menu.json", "1.3:dump_platform:platform.json", "1.4:quit"].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    const saved = demo(run, "saved.json");
    t.equal(saved.screen, SCREEN.gallery, "Done closed the settings");
    t.equal(saved.saves, 1, "and ran the save hook once");
    t.ok(saved.muted && saved.language === 1, "muted, in 简体中文");
    const stored = run.json<{ calls: Call[] }>("platform.json").calls.filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    const value = stored[0]?.args["value"] as { language?: string; muted?: boolean } | undefined;
    t.ok(stored.length === 1 && value?.language === "zh-CN" && value.muted === true, "og.settings written once, with the language and the mute");

    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    const read = next.json<{ calls: Call[] }>("platform.json").calls.filter((c) => c.method === "store.get" && c.args["key"] === "og.settings");
    t.ok(read.length === 1 && read[0]!.tick === 0, "the relaunch asked for og.settings on its first tick");
    const loaded = demo(next, "loaded.json");
    t.equal(loaded.loads, 1, "and took the answer");
    t.equal(loaded.language, 1, "the language came back: 简体中文");
    t.equal(loaded.muted, true, "the mute came back");
    const words = new Set(ui(next, "menu.json").map((e) => e.text));
    t.ok(words.has("开始") && words.has("设置"), "the main menu reads from the zh-CN table");
    t.ok(!words.has("Play") && !words.has("Settings"), "with none of the English left");
  },
});
