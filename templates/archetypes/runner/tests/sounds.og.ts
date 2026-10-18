// Sounds: the title is silent; a round's start, a jump, a coin and a crash
// each play their own voice (the sfx-<bit> entity holds an og.AudioPlayback
// while it plays), and the crash is heard in the mix.
import { playtest } from "@origingame/engine/playtest";
import { tap } from "./lib.ts";

interface EntityDump {
  components: Record<string, Record<string, unknown>>;
}

const VOICES = [
  ["start.json", "the round's start"],
  ["jump.json", "the jump"],
  ["coin.json", "the coin"],
  ["crash.json", "the crash"],
] as const;

export default playtest({
  name: "a round's start, a jump, a coin and a crash each play their sound",
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    "0.45:listen:title.wav:0.3",
    tap(0.5),
    "0.7:dump_entity:sfx-8:start.json",
    tap(1.2),
    "1.25:dump_entity:sfx-1:jump.json",
    "2.0:place:coin:-4.6",
    "2.15:dump_entity:sfx-2:coin.json",
    "2.6:place:crate:-4.6",
    "2.8:dump_entity:sfx-4:crash.json",
    "2.9:listen:crash.wav:0.3",
    "3.0:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.wav("title.wav").peak < 0.001, `the title is silent (peak ${run.wav("title.wav").peak})`);
    for (const [file, what] of VOICES) {
      const d = run.json<EntityDump>(file);
      t.ok("og.AudioPlayback" in d.components, `${what} plays its voice (${file}: ${Object.keys(d.components).join(", ")})`);
    }
    t.ok(run.wav("crash.wav").peak > 0.05, `the crash is heard (peak ${run.wav("crash.wav").peak})`);
  },
});
