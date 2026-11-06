// A race driven by scripted players (`bot:<n>`, src/seats.ts): in a room
// (online-lockstep) two processes race the same race, each driving its own
// seat by a different script, through drifts, boosts and contacts with
// every engine, tyre and cue sounding, and both worlds hash the same at
// tick 600 with no og.Desync (the second peer in 简体中文, its own setting); alone or on one device the bot drives this
// device's seats in the race.
import { playtest } from "@origingame/engine/playtest";
import { ONLINE } from "./lib.ts";

const at = (run: { json<T>(f: string): T }, f: string) => run.json<{ value: { position: [number, number, number] } }>(f).value.position;

const script = (n: number) => `0.3:race:go;0.35:get:car-0.og.Transform:from-${n}.json;0.4:bot:${n};9.9:get:car-0.og.Transform:to-${n}.json;10:quit`;

export default playtest({
  name: "scripted players race the same race in a room, or drive this device's seats",
  commands: script(1),
  ...(ONLINE ? { room: { players: 2, commands: [script(1), `0.2:language:zh-CN;${script(2)}`], name: "race" } } : {}),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const [ax, , az] = at(run, "from-1.json");
    const [bx, , bz] = at(run, "to-1.json");
    const moved = Math.sqrt((bx - ax) ** 2 + (bz - az) ** 2);
    t.ok(moved > 1, `the bot drove seat 0's car off the grid (${moved.toFixed(1)} m)`);
    if (!ONLINE) return;
    const peer = run.peers[0]!;
    t.equal(peer.tick, 600, "the peer quit on tick 600");
    t.equal(run.lockstep?.desync ?? null, null, "no desync on the run");
    t.equal(peer.lockstep?.desync ?? null, null, "no desync on the peer");
    t.equal(peer.netHash, run.netHash, `both worlds hash ${run.netHash} at tick 600`);
  },
});
