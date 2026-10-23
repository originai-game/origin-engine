// The intro cutscene (assets/timelines/intro.ogt) played from the command
// channel: `timeline:play:intro` makes the entity `timeline-intro`, and the
// timeline drives the camera along the "flyby" rail, walks the trickster
// with its clips, shows the narration from narration.srt, plays two sounds,
// runs `spawn_wave:2` at 5.5 s and fades in and out. At 4 s and 8 s the
// caption on screen is the .srt's cue for that time and the frames are
// compared with references/<tier>/intro-4.png and intro-8.png
// (tests/frames.ts; `OG_FX_CAPTURE=1 og test .` records them). After the end
// the camera is the game's again.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

interface EntityDump {
  components: Record<string, Record<string, number | boolean | string | number[]>>;
}

interface UiDump {
  elements: Array<{ id: string; text: string; x: number; y: number; width: number; height: number }>;
}

const FILES = ["intro-4.png", "intro-8.png"] as const;

export default playtest({
  name: "the intro plays its rail camera, captions, clips, event and fades from timeline:play:intro",
  size: "1280x720",
  commands: [
    "0:timeline:play:intro",
    "4:screenshot:intro-4.png",
    "4:dump_ui:ui-4.json",
    "4:dump_entity:timeline-intro:timeline-4.json",
    "4:dump_entity:trickster:trickster-4.json",
    "8:screenshot:intro-8.png",
    "8:dump_ui:ui-8.json",
    "8:dump_world:world-8.json",
    "10.5:dump_entity:timeline-intro:timeline-end.json",
    "10.5:dump_ui:ui-end.json",
    "10.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const caption = (file: string) =>
      run.json<UiDump>(file).elements.find((e) => e.id.startsWith("timeline-caption-") && e.id.endsWith("-text"));
    const at4 = caption("ui-4.json");
    t.equal(at4?.text, "The trickster walks out to meet it.", "at 4 s the caption is the .srt's second cue");
    t.ok(at4 !== undefined && at4.y > 600, `the caption sits at the bottom (y ${at4?.y})`);
    t.equal(caption("ui-8.json")?.text, "Here they come.", "at 8 s the caption is the third cue");
    const tl = run.json<EntityDump>("timeline-4.json").components["og.Timeline"]!;
    t.equal(tl["playing"], true, "the timeline plays at 4 s");
    t.near(Number(tl["time"]), 4, 0.02, "its time is the run's (it started at tick 0)");
    // The transform track walks the trickster from (2 0 0) at 2 s to (0.5 0 1.5) at 5 s.
    const p = run.json<EntityDump>("trickster-4.json").components["og.Transform"]!["position"] as number[];
    t.ok(p[0]! < 2 && p[0]! > 0.5 && p[2]! > 0 && p[2]! < 1.5, `the trickster is on its way at 4 s (${p.join(", ")})`);
    const world = run.json<{ entities: number }>("world-8.json");
    t.ok(world.entities >= 10, `the event key at 5.5 s ran spawn_wave:2 (${world.entities} entities)`);
    const end = run.json<EntityDump>("timeline-end.json").components["og.Timeline"]!;
    t.equal(end["playing"], false, "the timeline ended");
    t.equal(Number(end["time"]), 10, "at its duration");
    t.equal(caption("ui-end.json"), undefined, "no caption after the end");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
