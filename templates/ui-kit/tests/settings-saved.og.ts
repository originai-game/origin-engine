// The settings across a relaunch (store: "keep", og-platform-sdk §2 and §4):
// the first process, on the settings tree, sets the master volume to 0 and
// the render scale to 75 %, picks 简体中文 on the Language page and
// presses Apply, which writes og.settings; the second reads og.settings on
// its first tick and comes back muted and in Chinese, the title's words
// from the zh-CN table and the tree's values where Apply left them
// (examples/WEB-FINDINGS.md #10: the kit saved its settings and never read
// them, so a reload came back in English).
import { demo, playtest, tap, ui } from "./lib.ts";

type Call = { method: string; args: Record<string, unknown>; tick: number };

export default playtest({
  name: "the settings saved on Apply come back on a relaunch: muted, in Chinese",
  store: "keep",
  commands: [
    "0.3:show:settingsTree",
    "0.6:pat:set:pending:master=0",
    "0.6:pat:set:pending:scale=0.75",
    "1.0:dump_ui:graphics.json",
    tap(1.2, "Tabs/Row/language"),
    tap(1.6, "Choice-language"),
    tap(2.0, "Choice-language.list/Panel/Content/List/Option#3"),
    tap(2.4, "Foot/Apply"),
    "2.8:dump_entity:demo:saved.json",
    "2.8:dump_platform:platform.json",
    "2.9:quit",
  ].join(";"),
  then: [
    {
      commands: ["0.5:dump_entity:demo:loaded.json", "0.6:show:actionTitle", "1.3:dump_ui:menu.json", "1.3:dump_platform:platform.json", "1.4:show:settingsTree", "2.0:dump_ui:tree.json", "2.1:quit"].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    const saved = demo(run, "saved.json");
    t.equal(saved.saves, 1, "Apply ran the save once");
    t.ok(saved.muted && saved.language === 1, "muted, in 简体中文");
    const stored = run.json<{ calls: Call[] }>("platform.json").calls.filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    const value = stored[0]?.args["value"] as { language?: string; muted?: boolean; display?: { renderScale?: number } } | undefined;
    t.ok(stored.length === 1 && value?.language === "zh-CN" && value.muted === true, "og.settings written once, with the language and the mute");
    const page = new Set(ui(run, "graphics.json").map((e) => e.text));
    // The entries' words as written: the tree draws them in capitals (a label's locale-aware `uppercase`), and the
    // probe keeps a label's source text.
    t.ok(["Quality preset", "Render scale", "Frame cap", "Vertical sync", "Fullscreen"].every((w) => page.has(w)), "the Graphics page holds the display's entries");
    t.equal(value?.display?.renderScale, 0.75, "and the render scale is saved in the display's settings");

    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    const read = next.json<{ calls: Call[] }>("platform.json").calls.filter((c) => c.method === "store.get" && c.args["key"] === "og.settings");
    t.ok(read.length === 1 && read[0]!.tick === 0, "the relaunch asked for og.settings on its first tick");
    const loaded = demo(next, "loaded.json");
    t.equal(loaded.loads, 1, "and took the answer");
    t.equal(loaded.language, 1, "the language came back: 简体中文");
    t.equal(loaded.muted, true, "the mute came back");
    const words = new Set(ui(next, "menu.json").map((e) => e.text));
    t.ok(words.has("新游戏") && words.has("设置"), "the title reads from the zh-CN table");
    t.ok(!words.has("New game") && !words.has("Settings"), "with none of the English left");
    t.ok(ui(next, "tree.json").some((e) => e.text === "75%"), "the tree shows the render scale Apply left");
  },
});
