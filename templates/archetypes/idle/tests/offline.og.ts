// The save and the time away: Escape saves through og.platform's store;
// the game closes and opens two hours later (`relaunch:7200`, the host's
// time away); the title offers Continue; the welcome back says how long
// and how much (half the rate for the time away) and Collect closes it
// with the sparks added. Another device then writes a save with more
// progress: this one's next write is refused as stale, read again, and
// the other's progress kept and written back.
import { playtest } from "@origingame/engine/playtest";
import { game, type PlatformDump, press, tap, text, TITLE } from "./lib.ts";

export default playtest({
  name: "the save, two hours away credited at half the rate, a stale write merged",
  commands: [
    "0.3:play",
    "0.9:own:0:10",
    "0.9:own:1:2",
    "1.0:sparks:500",
    press("Escape", 1.2),
    "1.5:get:Game:saved.json",
    "1.6:relaunch:7200",
    "1.8:get:Game:closed.json",
    "2.2:dump_ui:title.json",
    tap("play", 2.3),
    "3.2:get:Game:welcome.json",
    "3.2:dump_ui:welcome-ui.json",
    "3.25:screenshot:welcome.png",
    tap("Collect", 3.3),
    "3.6:get:Game:collected.json",
    "3.7:stale_save",
    press("Escape", 3.8),
    "4.3:get:Game:merged.json",
    "4.3:dump_platform:platform.json",
    "4.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const saved = game(run, "saved.json");
    t.equal(saved.saves, 1, "Escape saved");
    t.equal(saved.rate, 10 * 0.5 * 2 + 2 * 4, "ten embers (doubled) and two storms: 18 a second");
    const closed = game(run, "closed.json");
    t.equal(closed.phase, TITLE, "the relaunch opens on the title");
    t.ok(text(run, "title.json", "play")?.startsWith("Continue") ?? false, "which offers Continue");
    const w = game(run, "welcome.json");
    t.equal(w.owned0, 10, "the save came back");
    t.ok(Math.abs(w.offline - 18 * 0.5 * 7200) < 1e-6, `two hours away at half of 18 a second: 64,800 (${w.offline})`);
    t.equal(w.welcome, true, "the welcome back is up");
    t.equal(text(run, "welcome-ui.json", "Body"), "While you were away (2 h 0 min) the orb made 64.8K sparks.", "and says how long and how much");
    const c = game(run, "collected.json");
    t.equal(c.welcome, false, "Collect closed it");
    t.ok(c.sparks >= saved.sparks + 64800, `the sparks were added (${c.sparks.toFixed(0)})`);
    const m = game(run, "merged.json");
    t.equal(m.saveConflicts, 1, "the write after the other device's was refused as stale");
    t.ok(m.dust === 3 && m.owned0 === 20 && m.lifetime === 1e9, "the save with more progress (the other device's) was kept");
    const p = run.json<PlatformDump>("platform.json");
    const writes = p.calls.filter((x) => x.method === "store.set" && x.args["key"] === "orb");
    const last = writes[writes.length - 1];
    t.equal((last?.args["value"] as { lifetime?: number } | undefined)?.lifetime, 1e9, "and written back");
    t.equal(last?.result?.ok, true, "at the revision read again");
  },
});
