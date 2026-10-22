// The template's playtest, run by `og test` through `og run --headless
// --commands`: a tap on the menu's Start button (found by its UI id, no key
// fallback) starts the trickster spinning and drops the crate onto the
// ground. Every check is an `expect:` step evaluated in the run at its tick
// (docs/design.md §3.4): a failed one fails the run with the tick and what
// it observed, and a frame one without a renderer is reported unchecked.
// The frame, the scene described as data, the dumps and the UI tree are
// written beside them.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "A tap on Start spins the trickster and drops the crate; the run writes its frame, description, dumps and UI tree",
  commands: [
    "0.5:touch_move_to:Start",
    "0.6:touch_start",
    "0.7:touch_end",
    // The tap was one press; the run started; the button says so.
    "0.8:expect:event:og.PointerDown:1",
    "0.8:expect:state:started==true",
    "0.9:expect:ui:text:=Started",
    // The trickster spins; the crate fell from 4 m and rests on the ground.
    "2.0:expect:state:spin>0",
    "2.0:expect:entity:crate:within:-3,0.4,-1,-1.8,0.6,2",
    "2.0:expect:event:og.CollisionStarted",
    // The frame: the sky above the scene, the dark menu panel, the crate in view.
    "2.0:screenshot:smoke.png",
    "2.0:expect:region:560,16,64,24:color:#9da5b1:24",
    "2.0:expect:entity:crate:within:200,180,140,120",
    "2.0:expect:entity:trickster:within:320,150,120,140",
    "2.1:describe:describe.json",
    "2.1:dump_entity:trickster:trickster.json",
    "2.15:dump_entity:crate:crate.json",
    "2.2:dump_world:world.json",
    "2.3:dump_ui:ui.json",
    "2.5:quit",
  ].join(";"),
});
