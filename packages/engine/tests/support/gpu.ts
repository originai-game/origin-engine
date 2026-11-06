import { test } from "bun:test";
import { fileURLToPath } from "node:url";
import { og } from "../../src/index.ts";
import { createEngine, loadNativeHost, native } from "../../src/node/index.ts";

declare const Bun: {
  spawnSync(cmd: string[], options: { stdout: "pipe"; stderr: "pipe" }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

// Probe in a child: loading a host and registering a game must not change
// the API/host state of the tests importing this module.
if (import.meta.main) {
  loadNativeHost();
  const made = createEngine(og.game({}));
  if (!made.ok) throw new Error(made.reason);
  try {
    process.stdout.write(native().og_node_attach_offscreen(made.value.id, 16, 16));
  } finally {
    made.value.destroy();
  }
}

function unavailable(): string | null {
  if (import.meta.main) return null;
  const probe = Bun.spawnSync(["bun", fileURLToPath(import.meta.url)], { stdout: "pipe", stderr: "pipe" });
  if (probe.exitCode !== 0) throw new Error(new TextDecoder().decode(probe.stderr));
  const result: { ok: boolean; reason?: string } = JSON.parse(new TextDecoder().decode(probe.stdout));
  if (result.ok) return null;
  if (!result.reason?.startsWith("no GPU adapter:")) throw new Error(result.reason ?? "renderer probe failed");
  return result.reason;
}

export const gpuError = unavailable();
export const gpuUnavailable = gpuError !== null;
export const gpuTest = test.skipIf(gpuUnavailable);
if (gpuError) {
  const expected = process.env.OG_RENDERER?.trim().toLowerCase() === "none";
  process.stderr.write(`WARNING ${expected ? "" : "unexpected "}missing GPU: ${gpuError}; GPU tests are registered as skipped, never passed\n`);
}
