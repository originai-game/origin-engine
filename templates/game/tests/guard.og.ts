// The guard (assets/ai/guard.ogt, a state machine on og.ai; the navmesh
// cooked from assets/nav/yard.ogt, walked by og.nav). Facing east at its
// first post, it does not see the trickster behind the wall (in range, in
// its cone) nor behind it (in range, in the open); in front of it, it does,
// raises the alarm once and chases; when the trickster slips behind the wall
// it searches where it last saw it and returns to its patrol 3 s later.
import { playtest } from "@origingame/engine/playtest";

interface AiDump {
  behaviour: string;
  kind: string;
  state: string;
  sees: boolean;
}
interface EntityDump {
  components: Record<string, Record<string, unknown>>;
}

const at = (run: { json<T>(f: string): T }, f: string) => run.json<EntityDump>(f).components["og.Transform"]!["position"] as number[];

export default playtest({
  name: "the guard patrols, sees inside its cone and not through the wall, chases, searches and returns",
  commands: [
    "0.3:lure:0:-3",
    "0.4:dump_ai:guard:walled.json",
    "0.4:dump_entity:guard:g0.json",
    "0.5:lure:-6:-7",
    "0.7:dump_ai:guard:behind.json",
    "1.0:lure:-4:-1",
    "1.1:dump_ai:guard:front.json",
    "2.2:dump_entity:guard:chasing.json",
    "2.2:dump_ai:guard:chase.json",
    "2.3:lure:-1:-4",
    "2.6:dump_ai:guard:lost.json",
    "6.0:dump_ai:guard:back.json",
    "6.0:dump_entity:guard:end.json",
    "6.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const walled = run.json<AiDump>("walled.json");
    t.equal(walled.behaviour, "guard", "the guard runs assets/ai/guard.ogt");
    t.equal(walled.kind, "machine", "a state machine");
    t.equal(walled.state, "patrol", "it patrols");
    t.equal(walled.sees, false, "the wall hides the trickster 6 m ahead, inside its cone and its 9 m sight");
    const g0 = at(run, "g0.json");
    t.ok(Math.abs(g0[1]!) < 0.2, `it stands on the navmesh's surface (y ${g0[1]!.toFixed(3)})`);
    const behind = run.json<AiDump>("behind.json");
    t.equal(behind.sees, false, "4 m behind it, in the open, the trickster is outside its 120° cone");
    t.equal(behind.state, "patrol", "so it keeps patrolling");

    const front = run.json<AiDump>("front.json");
    t.equal(front.sees, true, "in front of it the trickster is seen");
    t.equal(front.state, "alert", "and the guard is alert");
    const chase = run.json<AiDump>("chase.json");
    t.equal(chase.state, "alert", "still chasing a second later");
    const c = at(run, "chasing.json");
    const d = Math.hypot(c[0]! + 4, c[2]! + 1);
    t.ok(d < 2.0, `it closed in on the trickster (${d.toFixed(2)} m)`);

    const lost = run.json<AiDump>("lost.json");
    t.equal(lost.sees, false, "behind the wall the trickster is hidden");
    t.equal(lost.state, "search", "so the guard searches");
    t.equal(run.json<AiDump>("back.json").state, "patrol", "three seconds later it is back on patrol");
    t.equal(run.json<EntityDump>("end.json").components["Alarm"]!["raised"], 1, "the alarm was raised once, on entering alert");
  },
});
