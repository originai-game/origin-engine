// A purchase whose delivery a quit cut short: the store keeps it unconsumed,
// the relaunch asks for its entitlements and delivers it once, and the next
// relaunch finds nothing left to deliver. The game trusts the host's
// entitlements, never a save.
import { playtest } from "@origingame/engine/playtest";

type Wallet = { value: { gems: number; delivered: number } };

export default playtest({
  name: "a purchase cut short by a quit is delivered once on the next launch, from the host's entitlements",
  // The purchase is answered at 0.5 s and applied the tick after; the quit comes before the Shop reads it.
  commands: "0.5:buy;0.505:quit",
  store: "keep",
  then: [{ commands: "0.5:get:Wallet:wallet.json;0.6:quit" }, { commands: "0.5:get:Wallet:wallet.json;0.6:quit" }],
  check(run, t) {
    const [relaunch, again] = run.then;
    t.equal(relaunch!.json<Wallet>("wallet.json").value.delivered, 1, "the relaunch delivered the unconsumed purchase");
    t.equal(relaunch!.json<Wallet>("wallet.json").value.gems, 100, "100 gems");
    t.equal(again!.json<Wallet>("wallet.json").value.delivered, 0, "the next launch found it consumed");
  },
});
