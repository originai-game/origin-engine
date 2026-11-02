// @bun
// packages/engine/tests/fixtures/platform-shop/tests/ads.og.ts
import { playtest } from "/home/user/workspace/repo/packages/engine/src/playtest.ts";
var ads_og_default = playtest({
  name: "a skipped rewarded ad pays nothing and a completed one pays 10 gems",
  commands: "0.5:ad;1.5:ad;2.5:get:Wallet:wallet.json;2.6:quit",
  platform: { outcomes: { "ad.show": ["skipped"] } },
  check(run, t) {
    const w = run.json("wallet.json").value;
    t.equal(w.skipped, 1, "the skipped ad was counted, not paid");
    t.equal(w.gems, 10, "the completed ad paid 10 gems");
  }
});
export {
  ads_og_default as default
};

//# debugId=6765330379EBB42164756E2164756E21
//# sourceMappingURL=.playtest.js.map
