// What Brasswick keeps in og.platform across a relaunch (store: "keep").
// The first process chooses Español in the settings (saved as og.settings
// when they close), plays the second level to its lamp (the whole world in
// the vault slot, through saveSlot), and finishes it (the progress record
// at the revision read). The relaunch reads all three on its first tick: the
// title is in Spanish, Continue names the second level, the level select
// has all three open; Continue restores the world at the lamp, the gear
// taken before it still taken.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, hold, PLAY, script, tap, TITLE, value } from "./lib.ts";

interface Call {
  method: string;
  args: { key?: string; slot?: string; value?: { unlocked?: number; best?: number[] }; meta?: { level?: number; lamp?: number } };
  result?: { ok: boolean; data?: { revision?: number } };
}

export default playtest({
  name: "the settings, the vault slot at a lamp and the progress survive a relaunch; Continue restores the world at the lamp",
  store: "keep",
  commands: script(
    tap("settings", 0.6),
    tap("Tab#3", 1.2),
    tap("Option#1", 1.6),
    tap("Done", 2.0),
    "2.4:get:Prefs:prefs.json",
    "2.5:play:2",
    "3.0:warp:l2-gear-0:0:-0.4",
    "3.3:warp:l2-lamp-0:-1.2:0",
    hold("ArrowRight", 3.5, 0.25),
    "4.2:get:Game:lit.json",
    "4.3:finish",
    "4.8:get:Game:finished.json",
    "4.9:dump_platform:calls.json",
    "5.0:quit",
  ),
  then: [
    {
      commands: [
        "0.8:dump_ui:title.json",
        "0.8:get:Game:at-title.json",
        "0.8:get:Prefs:prefs-back.json",
        "0.9:levels",
        "1.5:dump_ui:levels.json",
        "1.6:title",
        // Continue after the saved tick (about 214): a restore that moves og.Time past the script's
        // last step ends og run there, the steps after it never run (FINDINGS.md #13).
        "4.0:touch_move_to:continue",
        "4.017:touch_start",
        "4.05:touch_end",
        "4.9:get:Game:continued.json",
        "4.9:get:l2-lamp-0.og.Light2D:light.json",
        "4.9:get:l2-gear-0.og.Sprite:gear.json",
        "4.9:dump_ui:play.json",
        "5.0:quit",
      ].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    t.equal(value<{ language: number }>(run, "prefs.json").language, 1, "Español chosen on the settings' Language tab");
    const lit = game(run, "lit.json");
    t.ok(lit.checkpoint === 0 && lit.gears === 1 && lit.saves === 1 && lit.hasSlot, `the lamp saved the world (${lit.saves} save, ${lit.gears} gear)`);
    const done = game(run, "finished.json");
    t.ok(done.unlocked === 3 && done.revision > 0, `finishing the second level unlocked the third and wrote the progress (revision ${done.revision})`);
    const calls = run.json<{ calls: Call[] }>("calls.json").calls;
    const slot = calls.find((c) => c.method === "slot.save");
    t.ok(slot !== undefined && slot.args.meta?.level === 1 && slot.args.meta?.lamp === 0 && slot.result?.ok === true, "the vault slot holds the second level's first lamp");
    const settings = calls.find((c) => c.method === "store.set" && c.args.key === "og.settings");
    t.ok(settings !== undefined && (settings.args.value as { language?: string } | undefined)?.language === "es", "the settings were saved with the language when they closed");
    const progress = calls.find((c) => c.method === "store.set" && c.args.key === "progress");
    t.ok(progress !== undefined && progress.args.value?.unlocked === 3, "the progress record names three levels open");

    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    const title = game(next, "at-title.json");
    t.ok(title.phase === TITLE && title.hasSlot && title.slotLevel === 1 && title.unlocked === 3, "the relaunch read the slot's manifest and the progress on its first tick");
    t.equal(value<{ language: number }>(next, "prefs-back.json").language, 1, "and the settings: Spanish again");
    const cont = elements(next, "title.json").find((e) => e.id === "continue");
    t.ok(cont !== undefined && !cont.states.includes("disabled") && cont.text === "Continuar · nivel 2", `Continue is offered for the second level, in Spanish ("${cont?.text}")`);
    const cards = elements(next, "levels.json").filter((e) => e.kind === "cell");
    t.ok(cards.length === 3 && cards.every((c) => c.name !== "lock"), "all three levels are open in the level select");
    const c = game(next, "continued.json");
    t.ok(c.phase === PLAY && c.level === 1 && c.checkpoint === 0 && c.loads === 1, `Continue restored the second level at its lamp (phase ${c.phase}, level ${c.level}, loads ${c.loads})`);
    t.ok(Math.abs(c.heroX - 128.5) < 1 && c.gears === 1, `Wick stands by the lamp with the gear he had (${c.heroX.toFixed(2)}, ${c.gears} gear)`);
    t.ok(value<{ intensity: number }>(next, "light.json").intensity > 1, "the lamp is lit again");
    t.equal(value<{ visible: boolean }>(next, "gear.json").visible, false, "the gear taken before it stays taken");
    t.ok(elements(next, "play.json").some((e) => e.text === "De vuelta al farol"), "a toast says so, in Spanish");
  },
});
