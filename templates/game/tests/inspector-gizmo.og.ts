// The inspector's transform gizmo (development builds, docs/design.md
// §3.6): `inspect:crate` opens the panel on the crate, the pointer resting
// on the x arrow lights it, a drag of 60 px along it moves the crate along
// x alone as one undo step, and the rotate and scale modes show their
// rings and bars. Each state's frame is compared with
// references/<tier>/inspector-gizmo-*.png (tests/frames.ts;
// `OG_FX_CAPTURE=1 og test .` records them). The handles' places are fixed
// at 1280x720: they are in each dump_inspector's `gizmo.handles`.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

interface Gizmo {
  mode: string;
  space: string;
  shown: boolean;
  hover: string | null;
  dragging: string | null;
  handles: Array<{ handle: string; x: number; y: number }>;
}

interface InspectorDump {
  dirty: string[];
  gizmo: Gizmo;
}

interface EntityDump {
  components: Record<string, Record<string, number[]>>;
}

const FILES = ["inspector-gizmo-translate.png", "inspector-gizmo-rotate.png", "inspector-gizmo-scale.png"] as const;

export default playtest({
  name: "the inspector's gizmo lights the x arrow, drags the crate along x as one undo step, and shows rotate and scale",
  size: "1280x720",
  commands: [
    "0.5:inspect:crate",
    "2:dump_entity:crate:before.json",
    // Resting on the x arrow (gizmo.handles' x).
    "2.1:touch_move:423,134",
    "2.3:screenshot:inspector-gizmo-translate.png",
    "2.3:dump_inspector:hover.json",
    // Dragged 60 px to the right, along the arrow on screen.
    "2.35:touch_start",
    "2.4:touch_move:443,134",
    "2.45:touch_move:463,134",
    "2.5:touch_move:483,134",
    "2.55:touch_end",
    "2.7:dump_entity:crate:moved.json",
    "2.7:dump_inspector:dragged.json",
    "2.8:inspect:mode:rotate",
    "3:screenshot:inspector-gizmo-rotate.png",
    "3.1:inspect:mode:scale",
    "3.3:screenshot:inspector-gizmo-scale.png",
    "3.3:dump_inspector:scale.json",
    // Undo (Inspector/Tools/Undo) takes the whole drag back.
    "3.4:touch_move:1164,74",
    "3.45:touch_start",
    "3.5:touch_end",
    "3.7:dump_entity:crate:undone.json",
    "3.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const hover = run.json<InspectorDump>("hover.json");
    t.ok(hover.gizmo.shown, "the crate shows the gizmo");
    t.equal(hover.gizmo.mode, "translate", "translate first");
    const x = hover.gizmo.handles.find((h) => h.handle === "x");
    t.ok(x !== undefined && Math.abs(x.x - 423) < 2 && Math.abs(x.y - 134) < 2, "the x arrow is where the script rests");
    t.equal(hover.gizmo.hover, "x", "the pointer lights the x arrow");
    const position = (f: string) => run.json<EntityDump>(f).components["og.Transform"]!["position"]!;
    const before = position("before.json");
    const moved = position("moved.json");
    t.ok(moved[0]! > before[0]! + 0.2, `the drag moved the crate along x (${before[0]} to ${moved[0]})`);
    t.equal(`${moved[1]},${moved[2]}`, `${before[1]},${before[2]}`, "and along x alone");
    const dragged = run.json<InspectorDump>("dragged.json");
    t.equal(dragged.dirty.join(","), "crate.og.Transform.position", "the drag is one unsaved field");
    t.equal(dragged.gizmo.dragging, null, "the drag ended with the release");
    t.equal(run.json<InspectorDump>("scale.json").gizmo.mode, "scale", "inspect:mode:scale");
    t.equal(position("undone.json").join(","), before.join(","), "one undo takes the drag back");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
