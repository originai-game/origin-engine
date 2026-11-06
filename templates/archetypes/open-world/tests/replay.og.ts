// Determinism. Alone or two on one device: a run with walking, running, a
// jump, a tree felled, the bridge built and a landmark claimed (the
// character controller, the animation graph and IK, the particles) runs
// twice in fresh processes and ends on the same world hash. Online: two
// processes play one run in lockstep through a relay, each driving its own
// seat by a different scripted bot, and both worlds hash the same at the
// room's tick 600 with no desync, each explorer having walked.
import { playtest } from "@origingame/engine/playtest";
import { game, ONLINE, press, tap } from "./lib.ts";

interface Facts { value: { x1?: number; z1?: number } }

const room = (bot: number) => `0.3:run;0.5:bot:${bot};9.5:get:Game:game.json;9.5:get:hero.og.Transform:hero0.json;9.5:get:hero-1.og.Transform:hero1.json;10:quit`;

export default ONLINE
  ? playtest({
      name: "two processes in one room play the same run to tick 600, each on its own seat",
      commands: room(1),
      room: { players: 2, commands: [room(1), room(2)], name: "isle" },
      check(run, t) {
        t.equal(run.warnings.join("; "), "", "the script ran without warnings");
        t.equal(run.peers.length, 1, "one peer beside the run");
        const peer = run.peers[0]!;
        t.ok(run.tick === 600 && peer.tick === 600, "both quit on tick 600");
        t.ok(run.lockstep?.running === true && peer.lockstep?.running === true, "lockstep ran on both");
        t.equal([run.lockstep?.peer, peer.lockstep?.peer].sort(), [0, 1], "the peers are seats 0 and 1");
        t.ok((run.lockstep?.desync ?? null) === null && (peer.lockstep?.desync ?? null) === null, "no desync");
        t.ok(run.netHash.length > 0 && peer.netHash === run.netHash, `both worlds hash ${run.netHash}`);
        const at = (r: typeof run, f: string) => r.json<{ value: { position: [number, number, number] } }>(f).value.position;
        t.equal(at(peer, "hero0.json"), at(run, "hero0.json"), "seat 0's explorer stands at the same place in both");
        t.equal(at(peer, "hero1.json"), at(run, "hero1.json"), "and seat 1's");
        const p0 = at(run, "hero0.json");
        const p1 = at(run, "hero1.json");
        t.ok(Math.hypot(p0[0] - p1[0], p0[2] - p1[2]) > 1, "each bot walked its own explorer its own way");
        t.equal(peer.json<Facts>("game.json").value, run.json<Facts>("game.json").value, "the peer read the same run at 9.5 s");
      },
    })
  : playtest({
      name: "a run with movement, a jump, a tree felled, the bridge built and a claim replays to the same hash",
      runs: 2,
      commands: [
        tap("Play", 0.7),
        "1.0:key_down:KeyW",
        "1.2:key_down:ShiftLeft",
        press("Space", 1.6),
        "2.2:key_up:ShiftLeft",
        "2.4:key_up:KeyW",
        "2.5:warp:tree",
        "2.7:key_down:KeyE",
        "4.0:key_up:KeyE",
        "4.1:give:6:4",
        "4.2:warp:bridge",
        "4.4:key_down:KeyE",
        "7.2:key_up:KeyE",
        "7.3:claim:2",
        "8.0:get:Game:end.json",
        "8.1:quit",
      ].join(";"),
      check(run, t) {
        t.equal(run.warnings.join("; "), "", "the script ran without warnings");
        t.equal(run.hashes.length, 2, "two runs");
        t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
        const end = game(run, "end.json");
        t.ok(end.jumps === 1 && end.felled === 1 && end.bridge && end.claimed === 1, `the run jumped, felled a tree, built the bridge and claimed a landmark (${end.jumps}, ${end.felled}, ${end.bridge}, ${end.claimed})`);
      },
    });
