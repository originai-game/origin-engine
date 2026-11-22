// The native loader refuses a host library built from other engine sources
// than the og that launched it (OG_COOK_FORMATS against og_node_formats),
// naming both stamps and the fix, before any loader refuses a cooked blob
// with "holds type ..., re-run og cook", which a re-cook cannot fix.
import { expect, test } from "bun:test";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

/** Loads a stand-in addon reporting `host` with OG_COOK_FORMATS `cook`; what the loader threw. */
function load(cook: string | undefined, host: string | null): string {
  const dir = mkdtempSync(join(tmpdir(), "og-host-formats-"));
  const addon = join(dir, "addon.cjs");
  const formats = host === null ? "" : `og_node_formats: () => ${JSON.stringify(host)},`;
  writeFileSync(addon, `module.exports = { ${formats} og_host_shared() { throw new Error("stand-in loaded"); } };`);
  const script = `import { loadNativeHost } from ${JSON.stringify(join(pkg, "src/node/index.ts"))};\ntry { loadNativeHost(${JSON.stringify(addon)}); } catch (e) { console.log(String(e.message)); }`;
  const env: Record<string, string | undefined> = { ...process.env, OG_NATIVE_ADDON: undefined, OG_COOK_FORMATS: cook };
  const r = Bun.spawnSync(["bun", "-e", script], { cwd: pkg, env });
  return new TextDecoder().decode(r.stdout) + new TextDecoder().decode(r.stderr);
}

test("a host library whose cooked formats differ from og's is named with the fix", () => {
  const newer = load("00000000000000aa", "00000000000000bb");
  expect(newer).toContain("built from other engine sources than this og");
  expect(newer).toContain("og cooks formats 00000000000000aa, the library reads 00000000000000bb");
  expect(newer).toContain("Fix: ");
  expect(newer).not.toContain("stand-in loaded");
  // A library from before the stamp existed is a mismatch too.
  expect(load("00000000000000aa", null)).toContain("the library reads none");
}, 60_000);

test("the same stamps, or a host started outside og, load as before", () => {
  for (const [cook, host] of [
    ["00000000000000aa", "00000000000000aa"],
    [undefined, "00000000000000bb"],
  ] as const) {
    const out = load(cook, host);
    expect(out).toContain("stand-in loaded");
    expect(out).not.toContain("other engine sources");
  }
}, 60_000);
