// The characters' animation and the simulated cloth and chain. The courier's
// clips are the library's retargeted at cook time (Mixamo names, an A-pose,
// other proportions); its blend tree runs idle, walk and run by the speed
// and switches to the air state in a jump. The imps and wardens play the
// same library retargeted at run time onto their own skeletons (Unreal
// names detected; the warden's names from a rig file), each 21 joints, on
// blend trees of their own. The cape (og.Cloth pinned at the shoulders)
// hangs when the courier stands and trails behind it when it runs; a
// banner stirs in the wind; the crypt's lamp hangs at the end of its chain
// (og.Rope), 2.6 m under the roof.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, press, script, value } from "./lib.ts";

interface Graph { asset: number; p0: number; p1: number; state: number }
interface Skeleton { asset: number; jointCount: number }

export default playtest({
  name: "retargeted clips on three skeletons through blend trees; the cape hangs and trails, a banner stirs, the lamp hangs on its chain",
  commands: script(
    "0.3:play:0:quick",
    "0.35:set:Tuning.sight=0",
    "0.4:warp:-6:40",
    "1.8:get:Game:standing.json",
    "1.8:get:hero-body.og.AnimationGraph:idle.json",
    hold("KeyW", 2.0, 1.4),
    hold("ShiftLeft", 2.0, 1.4),
    "3.2:get:Game:running.json",
    "3.2:get:hero-body.og.AnimationGraph:run.json",
    "3.25:screenshot:cape.png",
    press("Space", 3.6),
    "3.9:get:hero-body.og.AnimationGraph:air.json",
    "4.2:get:hero-body.og.Skeleton:courier.json",
    "4.2:get:imp-0.og.Skeleton:imp.json",
    "4.2:get:warden-0.og.Skeleton:warden.json",
    "4.2:get:imp-1.og.AnimationGraph:imp-graph.json",
    "4.2:get:warden-1.og.AnimationGraph:warden-graph.json",
    "4.3:get:Game:sway0.json",
    "5.3:get:Game:sway1.json",
    "5.4:warp:crypt:-4:0",
    "6.4:get:Game:crypt.json",
    "6.5:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const idle = value<Graph>(run, "idle.json");
    t.ok(idle.asset !== 0 && idle.p0 < 0.1 && idle.state === 0, "standing: the courier's graph idles on the ground state");
    const runG = value<Graph>(run, "run.json");
    t.near(runG.p0, 6.2, 0.4, `running: its speed parameter is the run (${runG.p0.toFixed(2)})`);
    t.equal(value<Graph>(run, "air.json").state, 1, "a jump cross-faded to the air state");
    const courier = value<Skeleton>(run, "courier.json");
    const imp = value<Skeleton>(run, "imp.json");
    const warden = value<Skeleton>(run, "warden.json");
    t.ok([courier, imp, warden].every((s) => s.jointCount === 21), "three skeletons of 21 joints each are posed");
    t.ok(new Set([courier.asset, imp.asset, warden.asset]).size === 3, "each its own rig");
    const ig = value<Graph>(run, "imp-graph.json");
    const wg = value<Graph>(run, "warden-graph.json");
    t.ok(ig.asset !== 0 && wg.asset !== 0 && ig.asset !== wg.asset, "the imps and wardens run blend trees of the retargeted clips");
    t.ok(ig.p0 > 0.5 || wg.p0 > 0.5, `walking their rounds (imp ${ig.p0.toFixed(2)}, warden ${wg.p0.toFixed(2)} m/s)`);
    const standing = game(run, "standing.json");
    t.ok(standing.capeHang > 0.6 && Math.abs(standing.capeBehind) < 0.6, `standing, the cape hangs (${standing.capeHang.toFixed(2)} m down, ${standing.capeBehind.toFixed(2)} m behind)`);
    const running = game(run, "running.json");
    t.ok(running.capeBehind > standing.capeBehind + 0.3, `running, it trails behind (${running.capeBehind.toFixed(2)} m)`);
    const s0 = game(run, "sway0.json").bannerSway;
    const s1 = game(run, "sway1.json").bannerSway;
    t.ok(s0 > 0.05 && Math.abs(s1 - s0) > 0.005, `the banner stirs in the gusts (${s0.toFixed(3)}, then ${s1.toFixed(3)} m)`);
    t.near(game(run, "crypt.json").lampDrop, 2.6, 0.2, `the crypt's lamp hangs ${game(run, "crypt.json").lampDrop.toFixed(2)} m under the roof on its chain`);
  },
});
