// og replay keeps the platform's journal (docs/platform-asks.md #18): a
// purchase the recording's fake cancelled and a notice it sent are fed back
// at their ticks on compare, which runs with no fake, so the replay hashes
// as recorded; without the journal the default fake buys and sends nothing,
// and the replay diverges at the tick after the answer.
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const project = join(repo, "packages/engine/tests/fixtures/platform-replay");

test("og replay feeds the platform's recorded answers and notices back at their ticks", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-platform-replay-"));
  try {
    const fake = join(dir, "fake.json");
    writeFileSync(fake, JSON.stringify({ outcomes: { "purchase.buy": ["cancelled"] }, notices: [{ tick: 30, kind: "entitlements", data: { entitlements: [] } }] }));
    const trace = join(dir, "trace.json");
    const og = (...args: string[]) => Bun.spawnSync([ogBin(), "replay", ...args], { cwd: project });
    const rec = og("record", project, "--commands", "1:wait", "--ticks", "60", "--clock", "1767225600000", "--platform-fake", fake, "--out", trace, "--json");
    expect(rec.exitCode, new TextDecoder().decode(rec.stderr)).toBe(0);
    const t = JSON.parse(readFileSync(trace, "utf8")) as { platform: { entries: Array<{ tick: number; request?: number; answer?: { ok: boolean; error?: { code: string } }; notice?: { kind: string } }> } };
    const answer = t.platform.entries.find((e) => e.answer)!;
    expect([answer.answer!.ok, answer.answer!.error?.code]).toEqual([false, "cancelled"]);
    expect(t.platform.entries.filter((e) => e.notice).map((e) => [e.tick, e.notice!.kind])).toEqual([[30, "entitlements"]]);

    const same = og("compare", trace, project, "--json");
    const report = JSON.parse(new TextDecoder().decode(same.stdout)) as { same: boolean; firstDivergentTick: number | null };
    expect([same.exitCode, report.same, report.firstDivergentTick]).toEqual([0, true, null]);

    // The same trace without its journal: the live default fake answers ok and sends no notice.
    const stripped = { ...JSON.parse(readFileSync(trace, "utf8")) };
    delete stripped.platform;
    const bare = join(dir, "bare.json");
    writeFileSync(bare, JSON.stringify(stripped));
    const diverged = og("compare", bare, project, "--json");
    const r2 = JSON.parse(new TextDecoder().decode(diverged.stdout)) as { same: boolean; firstDivergentTick: number | null };
    expect([diverged.exitCode, r2.same]).toEqual([3, false]);
    expect(r2.firstDivergentTick).toBeGreaterThan(10);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 300_000);
