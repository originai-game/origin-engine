import { og } from "@origingame/engine";
import { Shell, shellStates, startShell } from "./shell-state.ts";
import { mountShell } from "./shell.tsx";

export default og.game({
  states: shellStates,
  systems: [Shell],
  start(world) {
    // Initialize the game's world here, before startShell captures New game's reset point.
    // Menu art: setShellArt(world, { keyArt, logo, liveScene }) with og.load.texture handles.
    startShell(world);
  },
  ui: mountShell,
});
