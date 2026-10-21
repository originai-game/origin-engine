// The foes on the navmesh, minded by og.ai (og-ai §2-§5). A thorn imp
// patrols its round, sees the courier, chases (the navmesh path), winds up
// and bites (a heart's worth of strength, the hit stop, the knock-back);
// struck twice it flees and keeps away. Another imp walks its round
// unbothered. A stone warden alerts on sight ("!", the alert, a toast),
// closes in and slams: a long wind-up, a 0.9 m lunge that is the clip's
// root motion (the warden's own Transform carried forward), and the hit.
import { playtest } from "@origingame/engine/playtest";
import { dist, game, press, script, value } from "./lib.ts";
import type { Foe } from "./lib.ts";

interface Ai { path: string; sees: boolean; node: string }

export default playtest({
  name: "imps patrol, chase, bite and flee on the navmesh; a warden alerts, lunges by root motion and slams",
  commands: script(
    "0.3:play:0:quick",
    "0.4:warp:-3:31",
    "0.5:dump_ai:imp-1:patrol.json",
    "1.2:dump_ai:imp-0:chase.json",
    "1.2:dump_nav:imp-0:chase-nav.json",
    "1.2:get:imp-0.og.NavAgent:agent.json",
    "2.4:get:Game:bitten.json",
    "2.4:get:imp-0.Foe:biter.json",
    press("KeyJ", 2.5),
    press("KeyJ", 3.1),
    "3.6:get:imp-0.Foe:struck.json",
    "3.6:get:imp-0.og.Transform:near.json",
    "3.6:get:Game:at-strike.json",
    "4.6:dump_ai:imp-0:flee.json",
    "4.6:get:imp-0.og.Transform:far.json",
    "4.6:get:Game:at-flee.json",
    // The terrace warden.
    "5.0:play:1:quick",
    "5.1:warp:6:-8",
    "6.0:get:warden-0.Foe:alerted.json",
    "6.0:dump_ai:warden-0:alert.json",
    "7.38:get:warden-0.og.Transform:lunge0.json",
    "7.38:get:warden-0.Foe:slam0.json",
    "7.38:get:warden-0.og.Skeleton:rm.json",
    "7.9:get:warden-0.og.Transform:lunge1.json",
    "7.9:screenshot:slam.png",
    "8.6:get:Game:slammed.json",
    "8.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.json<Ai>("patrol.json").path, "selector > patrol round", "imp-1 walks its round");
    const chase = run.json<Ai>("chase.json");
    t.ok(chase.sees && chase.path.endsWith("chase"), `imp-0 sees the courier and chases (${chase.path})`);
    t.equal(run.json<{ status: string }>("chase-nav.json").status, "moving", "along a navmesh path");
    const agent = value<{ velocity: number[]; speed: number }>(run, "agent.json");
    t.ok(Math.hypot(agent.velocity[0]!, agent.velocity[2]!) > 3, `at the chase speed (${Math.hypot(agent.velocity[0]!, agent.velocity[2]!).toFixed(2)} m/s)`);
    const bitten = game(run, "bitten.json");
    t.ok(bitten.hp <= 88 && bitten.hurts >= 1 && value<Foe>(run, "biter.json").bites >= 1, `it bit (strength ${bitten.hp})`);
    t.equal(value<Foe>(run, "struck.json").hp, 1, "two strikes left it one thorn");
    const flee = run.json<Ai>("flee.json");
    t.ok(flee.path.endsWith("flee"), `and it flees (${flee.path})`);
    const s0 = game(run, "at-strike.json");
    const s1 = game(run, "at-flee.json");
    const d0 = dist(value<{ position: number[] }>(run, "near.json").position, [s0.heroX, s0.heroY, s0.heroZ]);
    const d1 = dist(value<{ position: number[] }>(run, "far.json").position, [s1.heroX, s1.heroY, s1.heroZ]);
    t.ok(d1 > d0 + 2, `keeping away (${d0.toFixed(1)} m, then ${d1.toFixed(1)} m)`);
    const alerted = value<Foe>(run, "alerted.json");
    t.ok(alerted.alerts >= 1, "the warden saw the courier and alerted");
    t.equal(run.json<Ai>("alert.json").path.length > 0, true, `its machine is running (${run.json<Ai>("alert.json").path})`);
    t.ok(value<Foe>(run, "slam0.json").slam > 0, "it raised its fists to slam");
    t.equal(value<{ rootMotion: string }>(run, "rm.json").rootMotion, "planar", "with the clip's root motion on");
    const moved = dist(value<{ position: number[] }>(run, "lunge0.json").position, value<{ position: number[] }>(run, "lunge1.json").position);
    t.ok(moved > 0.5, `the lunge carried it ${moved.toFixed(2)} m`);
    const slammed = game(run, "slammed.json");
    t.ok(slammed.hp <= 66, `the slam landed (strength ${slammed.hp})`);
  },
});
