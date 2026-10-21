// The save slot across a relaunch (og-game §17, og-platform-sdk §2,
// store: "keep"). Resting at the crypt terrace's waystone restores the
// courier's strength, marks chapter II reached and saves the whole world
// (world.snapshot(), about 78 KB of text, written in parts under the
// store's 64 KiB a key: FINDINGS #13) beside what the title shows of it. Rest
// and quit returns to the title. The relaunch reads the slot on its first
// tick: the title offers Continue with the place and the hour (the primary
// action, focused); Continue reads the parts and world.restore()s them, so
// the courier stands by that waystone with the sigil and the hillwort it
// carried, the session's settings are read again, a toast welcomes it back; and the Chapters screen has chapter
// II unlocked from the stored progress.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, PLAY, press, script, tap, TITLE, value } from "./lib.ts";

export default playtest({
  name: "a waystone saves the whole world in the store; a relaunch continues it there, and chapter II is unlocked",
  store: "keep",
  commands: script(
    "0.3:play:0:quick",
    "0.35:set:Tuning.sight=0",
    "0.4:give:herbs:2",
    "0.45:give:sigils:1",
    "0.5:hp:30",
    "0.6:warp:waystone-1",
    press("KeyF", 0.9),
    "1.2:get:Game:rested.json",
    "1.3:dump_platform:calls.json",
    press("Escape", 1.5),
    tap("quit", 1.9),
    "2.6:get:Game:quit.json",
    "2.7:quit",
  ),
  then: [
    {
      commands: [
        "0.8:dump_ui:title.json",
        "0.8:get:Game:at-title.json",
        "0.9:key_down:Enter",
        "0.934:key_up:Enter",
        "2.4:get:Game:continued.json",
        "2.4:dump_ui:play.json",
        "2.5:screenshot:continued.png",
        "2.6:key_down:Escape",
        "2.634:key_up:Escape",
        "3.0:touch_move_to:quit",
        "3.017:touch_start",
        "3.05:touch_end",
        "3.9:open:chapters",
        "5.1:dump_ui:chapters.json",
        "5.2:quit",
      ].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    const rested = game(run, "rested.json");
    t.equal([rested.hp, rested.saves, rested.savePlace, rested.chaptersReached & 2], [100, 1, 1, 2], "resting restored strength, reached chapter II and saved there");
    t.ok(rested.saveBytes > 0 && rested.saveParts === Math.ceil(rested.saveBytes / 60000), `the world (${rested.saveBytes} characters) went in ${rested.saveParts} parts`);
    const calls = run.json<{ calls: Array<{ method: string; args: { key?: string; value?: unknown } }> }>("calls.json").calls;
    const keys = calls.filter((c) => c.method === "store.set").map((c) => c.args.key);
    t.ok(keys.includes("slot") && keys.includes("slot.0") && keys.includes("slot.1"), `the slot and its parts were written (${keys.join(", ")})`);
    const slot = calls.find((c) => c.method === "store.set" && c.args.key === "slot")!.args.value as { place: number; parts: number };
    t.equal([slot.place, slot.parts], [1, rested.saveParts], "the slot names its waystone and its parts");
    t.equal(game(run, "quit.json").phase, TITLE, "Rest and quit went to the title");

    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    t.ok(game(next, "at-title.json").hasSave, "the relaunch read the slot on its first tick");
    const title = elements(next, "title.json");
    const cont = title.find((e) => e.id === "continue");
    t.ok(cont !== undefined && cont.text === "Continue · The Crypt Terrace, 16:30", `Continue names the place and the hour (${cont?.text})`);
    t.ok(cont !== undefined && cont.states.includes("focused"), "and has the focus, so Enter takes it");
    const c = game(next, "continued.json");
    t.equal(c.phase, PLAY, "Continue restored the run");
    t.equal([c.sigils, c.herbs, c.loads, c.hp], [1, 2, 1, 100], "with the sigil, the hillwort and the strength it had");
    t.ok(Math.hypot(c.heroX - 1.8, c.heroZ - 6.2) < 0.5, `by the crypt terrace's waystone (${c.heroX.toFixed(2)}, ${c.heroZ.toFixed(2)})`);
    t.ok(elements(next, "play.json").some((e) => e.text === "Back on the stair, The Crypt Terrace"), "a toast welcomes the courier back");
    const chapters = elements(next, "chapters.json");
    const cards = chapters.filter((e) => e.parent === "Levels" && e.id.startsWith("Cell#"));
    const unlocked = cards.filter((c) => !chapters.some((l) => l.parent === c.id && l.id === "Lock"));
    t.equal(cards.length, 4, "the Chapters screen has four cards");
    t.ok(unlocked.length >= 2, `chapters I and II are open on it (${unlocked.length} open)`);
    void value;
  },
});
