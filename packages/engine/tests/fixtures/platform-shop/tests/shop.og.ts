// Purchases through the local platform's fake: the first is cancelled, the
// second completes, the third is pending and its notice settles it; each
// completed one is granted once and consumed, and 100 gems unlock
// `first.gems`.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

type Wallet = { gems: number; cancelled: number; delivered: number; unlocked: number };
type Dump = { calls: Array<{ method: string; args: Record<string, unknown>; state: string; result?: { ok: boolean } }>; notices: Array<{ kind: string; data: { state: string } }> };

test("a cancelled, a completed and a pending purchase: two delivered, each once, and the achievement unlocked", async () => {
  const run = await runPlaytest({
    commands: "0.5:buy;1:buy;1.5:buy;2.5:get:Wallet:wallet.json;2.6:dump_platform:platform.json;2.7:quit",
    platform: { products: [{ id: "gems", title: "100 gems" }], outcomes: { "purchase.buy": ["cancelled", "ok", "pending"] } },
  });
  const w = run.json<{ value: Wallet }>("wallet.json").value;
  expect(w.cancelled, "the first purchase was cancelled").toBe(1);
  expect(w.delivered, "the completed and the settled pending purchase were delivered").toBe(2);
  expect(w.gems, "100 gems each").toBe(200);
  expect(w.unlocked, "100 gems unlocked the achievement").toBe(1);
  const p = run.json<Dump>("platform.json");
  expect(p.calls.filter((c) => c.method === "purchase.consume" && c.result?.ok).length, "both were consumed").toBe(2);
  expect(p.calls.filter((c) => c.method === "achievement.unlock").length, "unlocked once").toBe(1);
  expect(p.notices.map((n) => `${n.kind}:${n.data.state}`), "the pending purchase settled by a notice").toEqual(["purchase:completed"]);
});
