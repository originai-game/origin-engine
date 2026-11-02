// Purchases through the local platform's fake: the first is cancelled, the
// second completes, the third is pending and its notice settles it; each
// completed one is granted once and consumed, and 100 gems unlock
// `first.gems`.
import { playtest } from "@origingame/engine/playtest";

type Wallet = { gems: number; cancelled: number; delivered: number; unlocked: number };
type Dump = { calls: Array<{ method: string; args: Record<string, unknown>; state: string; result?: { ok: boolean } }>; notices: Array<{ kind: string; data: { state: string } }> };

export default playtest({
  name: "a cancelled, a completed and a pending purchase: two delivered, each once, and the achievement unlocked",
  commands: "0.5:buy;1:buy;1.5:buy;2.5:get:Wallet:wallet.json;2.6:dump_platform:platform.json;2.7:quit",
  platform: { products: [{ id: "gems", title: "100 gems" }], outcomes: { "purchase.buy": ["cancelled", "ok", "pending"] } },
  check(run, t) {
    const w = run.json<{ value: Wallet }>("wallet.json").value;
    t.equal(w.cancelled, 1, "the first purchase was cancelled");
    t.equal(w.delivered, 2, "the completed and the settled pending purchase were delivered");
    t.equal(w.gems, 200, "100 gems each");
    t.equal(w.unlocked, 1, "100 gems unlocked the achievement");
    const p = run.json<Dump>("platform.json");
    t.equal(p.calls.filter((c) => c.method === "purchase.consume" && c.result?.ok).length, 2, "both were consumed");
    t.equal(p.calls.filter((c) => c.method === "achievement.unlock").length, 1, "unlocked once");
    t.equal(p.notices.map((n) => `${n.kind}:${n.data.state}`).join(), "purchase:completed", "the pending purchase settled by a notice");
  },
});
