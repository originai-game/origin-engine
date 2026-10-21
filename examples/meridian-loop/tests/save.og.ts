// The platform across a relaunch (`store: "keep"`): the first process
// drives a lap (the autopilot), which becomes the record with its poses,
// and switches the language to German in the settings (saved when they
// close). The second process comes back to a German title naming the
// record, the record's poses loaded, and the ghost driving the very first
// lap of the new race where the first process's car drove.
import { playtest } from "@origingame/engine/playtest";
import { dist, position, press, race, script, texts } from "./lib.ts";

const GO = 31;

export default playtest({
  name: "a lap record and its ghost, and the language, come back after a relaunch",
  store: "keep",
  commands: script(
    "0.5:race:go",
    "0.6:drive:auto",
    `${((GO + 600) / 60).toFixed(3)}:get:car.og.Transform:car-600.json`,
    "35.5:get:Race:lapped.json",
    "35.6:open:pause",
    "36:open:settings",
    "36.6:language:de",
    press("Escape", 37),
    "37.6:dump_platform:calls.json",
    "37.7:quit",
  ),
  then: [
    {
      commands: script(
        "1:get:Race:back.json",
        "1:dump_ui:title.json",
        "1.5:race:go",
        "1.6:drive:auto",
        `${((GO + 60 + 600) / 60).toFixed(3)}:get:ghost.og.Transform:ghost-600.json`,
        `${((GO + 60 + 600) / 60).toFixed(3)}:get:Race:racing.json`,
        "13:quit",
      ),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    const lapped = race(run, "lapped.json");
    t.equal(lapped.record, lapped.lap1, `the first lap is the record (${lapped.lap1} ticks)`);
    const calls = run.json<{ calls: { method: string; args: Record<string, unknown> }[] }>("calls.json").calls;
    t.ok(calls.some((c) => c.method === "store.set" && c.args.key === "og.settings" && (c.args.value as { language?: string }).language === "de"), "closing the settings saved the language");
    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    const back = race(next, "back.json");
    t.ok(back.loaded, "the stored record was read on the first tick");
    t.equal(back.record, lapped.record, "the record came back");
    t.equal(back.bestLen, Math.ceil(lapped.record / 2), "with its poses");
    const title = texts(next, "title.json");
    t.ok(title.includes("Zeitfahren"), "the title is in German again");
    const ms = Math.round((lapped.record * 1000) / 60);
    const shown = `Rundenrekord 0:${String(Math.floor(ms / 1000)).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
    t.ok(title.includes(shown), `the title names the record (${shown})`);
    t.ok(race(next, "racing.json").ghostActive, "the stored ghost drives the first lap of the new race");
    const e = dist(position(run, "car-600.json"), position(next, "ghost-600.json"));
    t.ok(e < 0.02, `600 ticks into the lap the ghost is where the first process's car was (${(e * 100).toFixed(1)} cm)`);
  },
});
