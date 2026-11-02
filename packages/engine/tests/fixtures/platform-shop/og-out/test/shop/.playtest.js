// @bun
// packages/engine/tests/fixtures/platform-shop/tests/shop.og.ts
import { playtest } from "/home/user/workspace/repo/packages/engine/src/playtest.ts";
var shop_og_default = playtest({
  name: "a cancelled, a completed and a pending purchase: two delivered, each once, and the achievement unlocked",
  commands: "0.5:buy;1:buy;1.5:buy;2.5:get:Wallet:wallet.json;2.6:dump_platform:platform.json;2.7:quit",
  platform: { products: [{ id: "gems", title: "100 gems" }], outcomes: { "purchase.buy": ["cancelled", "ok", "pending"] } },
  check(run, t) {
    const w = run.json("wallet.json").value;
    t.equal(w.cancelled, 1, "the first purchase was cancelled");
    t.equal(w.delivered, 2, "the completed and the settled pending purchase were delivered");
    t.equal(w.gems, 200, "100 gems each");
    t.equal(w.unlocked, 1, "100 gems unlocked the achievement");
    const p = run.json("platform.json");
    t.equal(p.calls.filter((c) => c.method === "purchase.consume" && c.result?.ok).length, 2, "both were consumed");
    t.equal(p.calls.filter((c) => c.method === "achievement.unlock").length, 1, "unlocked once");
    t.equal(p.notices.map((n) => `${n.kind}:${n.data.state}`).join(), "purchase:completed", "the pending purchase settled by a notice");
  }
});
export {
  shop_og_default as default
};

//# debugId=EC79BA875B4D94F664756E2164756E21
//# sourceMappingURL=.playtest.js.map
