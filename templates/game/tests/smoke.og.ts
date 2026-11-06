// The template's playtest, run by `og test` through `og run --headless
// --commands`: a tap on the menu's Start button (found by its UI id, no key
// fallback) starts the trickster spinning and drops the crate onto the
// ground.
//
// Properties first: `invariants` hold after every tick of the run (the first
// tick one fails is the failure, with what it observed), beside the engine's
// default invariants (no NaN, every field inside its range, no dangling
// entity, everything inside og.WorldBounds) and the game's og.assert.always
// calls; `require` names the og.assert.sometimes and reachable sites
// (src/main.ts) the run must reach. The `expect:` steps pin what happens at
// one tick. One frame is kept, for what only a frame shows: the sky's colour.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "A tap on Start spins the trickster and drops the crate onto the ground",
  invariants: [
    // The crate never sinks into the ground nor flies off the yard.
    "entity:crate:og.Transform.position.y>=0.4",
    "entity:crate:within:-15,0,-15,15,5,15",
    // The trickster stays on its spot while it spins.
    "entity:trickster:og.Transform.position.y>-0.01",
  ],
  require: ["the player starts the run", "the trickster spins", "a held body is let fall"],
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
    "2.0:expect:entity:crate:within:-3,0.4,-1,1,0.6,2",
    "2.0:expect:event:og.CollisionStarted",
    // The look (1280x720): the sky at the top.
    "2.0:screenshot:smoke.png",
    "2.0:expect:region:560,16,64,24:color:#9da5b1:24",
    "2.5:quit",
  ].join(";"),
});
