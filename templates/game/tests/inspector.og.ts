// The inspector (development builds, docs/design.md §3.6): `inspect:crate`
// opens its panel on the crate, a click on its scale's x field, its text
// cleared and `1.5` typed with Enter edit it through the probe's `set`, and
// the Resources tab lists the world's resources, the game's Run unfolded.
// Each state's frame is compared with references/<tier>/inspector-*.png
// (tests/frames.ts; `OG_FX_CAPTURE=1 og test .` records them). The panel's
// rectangles are fixed at 1280x720: its elements are in each dump_inspector.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

interface InspectorDump {
  open: boolean;
  selected: string | null;
  dirty: string[];
  status: string;
  elements: Array<{ path: string; text: string; x: number; y: number; width: number; height: number }>;
}

interface EntityDump {
  components: Record<string, Record<string, number | boolean | string | number[]>>;
}

const FILES = ["inspector-entity.png", "inspector-edited.png", "inspector-resources.png"] as const;

export default playtest({
  name: "the inspector opens on the crate, edits its scale through set, and lists the resources",
  size: "1280x720",
  commands: [
    "0.5:inspect:crate",
    // The crate has landed by 2 s.
    "2:screenshot:inspector-entity.png",
    "2:dump_inspector:entity.json",
    // Its scale's x field (Inspector/Detail/F:crate.og.Transform.scale/P0),
    // left of the detail pane's scrollbar gutter.
    "2.2:touch_move:1070,573",
    "2.25:touch_start",
    "2.3:touch_end",
    // Its one character cleared (Ctrl+A is the line's start on a Mac).
    "2.4:key_down:End",
    "2.42:key_up:End",
    "2.44:key_down:Backspace",
    "2.46:key_up:Backspace",
    // `type` takes two ticks a character: Enter after the three.
    "2.5:type:1.5",
    "2.8:key_down:Enter",
    "2.83:key_up:Enter",
    "3:screenshot:inspector-edited.png",
    "3:dump_inspector:edited.json",
    "3:dump_entity:crate:crate.json",
    // The Resources tab (Inspector/View/Tab#1).
    "3.2:touch_move:1169,112",
    "3.25:touch_start",
    "3.3:touch_end",
    // The game's own Run resource, unfolded (Inspector/Detail/S:Run).
    "3.4:touch_move:1084,190",
    "3.45:touch_start",
    "3.5:touch_end",
    "3.7:screenshot:inspector-resources.png",
    "3.7:dump_inspector:resources.json",
    "3.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const entity = run.json<InspectorDump>("entity.json");
    t.ok(entity.open, "inspect:crate opened the panel");
    t.equal(entity.selected, "crate", "on the crate");
    const at = (d: InspectorDump, path: string) => d.elements.find((e) => e.path === path);
    const field = at(entity, "Inspector/Detail/F:crate.og.Transform.scale/P0");
    t.ok(field !== undefined && Math.abs(field.x + field.width / 2 - 1070) < 1 && Math.abs(field.y + field.height / 2 - 573) < 1, "the scale's x field is where the script clicks");
    const edited = run.json<InspectorDump>("edited.json");
    t.equal(edited.dirty.join(","), "crate.og.Transform.scale", "the edit is marked unsaved");
    t.equal(at(edited, "Inspector/Bar/Save")?.text, "Save (1)", "Save counts it");
    const scale = run.json<EntityDump>("crate.json").components["og.Transform"]!["scale"] as number[];
    t.equal(scale.join(","), "1.5,1,1", "the crate's scale took the edit");
    const resources = run.json<InspectorDump>("resources.json");
    t.ok(resources.elements.some((e) => e.path.startsWith("Inspector/Detail/F:Run.")), "the Resources tab shows the Run resource's fields");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
