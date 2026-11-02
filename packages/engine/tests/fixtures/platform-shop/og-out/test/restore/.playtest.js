// @bun
// packages/engine/tests/fixtures/platform-shop/tests/restore.og.ts
import { playtest } from "/home/user/workspace/repo/packages/engine/src/playtest.ts";
var restore_og_default = playtest({
  name: "a purchase cut short by a quit is delivered once on the next launch, from the host's entitlements",
  commands: "0.5:buy;0.505:quit",
  store: "keep",
  then: [{ commands: "0.5:get:Wallet:wallet.json;0.6:quit" }, { commands: "0.5:get:Wallet:wallet.json;0.6:quit" }],
  check(run, t) {
    const [relaunch, again] = run.then;
    t.equal(relaunch.json("wallet.json").value.delivered, 1, "the relaunch delivered the unconsumed purchase");
    t.equal(relaunch.json("wallet.json").value.gems, 100, "100 gems");
    t.equal(again.json("wallet.json").value.delivered, 0, "the next launch found it consumed");
  }
});
export {
  restore_og_default as default
};

//# debugId=7646F2D8534CE02A64756E2164756E21
//# sourceMappingURL=.playtest.js.map
