// Records the `vulkan-cpu` gfx reference tier (Mesa lavapipe, the CI tier)
// inside the platform's E2B probe sandbox, as tools/bench/e2b-run.ts does.
// Billable: one sandbox, every scene in one run.
//
//   bun install --cwd tools/bench        # once: the pinned e2b SDK
//   CARGO_TARGET_DIR=target/agent-render cargo zigbuild --release \
//     --target x86_64-unknown-linux-gnu.2.31 -p og-tests --bin og-gfx --test gfx --test effects
//   bun crates/og-tests/scripts/capture-vulkan-cpu.ts
//
// In the sandbox: `og-gfx --all --capture` renders every scene on lavapipe;
// the gfx test binary then runs against those frames (determinism and the
// behaviour tests on the CI tier), the effects test binary runs the E1
// behaviour tests (the 600-tick particle determinism on lavapipe among
// them); a short timing run follows. The frames go
// to crates/og-tests/references/vulkan-cpu/, the logs to
// $CARGO_TARGET_DIR/vulkan-cpu/. Reads E2B_API_KEY from the platform .env.
import { createRequire } from "node:module";
import { readdirSync, statSync, mkdirSync } from "node:fs";

const root = new URL("../../..", import.meta.url).pathname.replace(/\/$/, "");
const require = createRequire(`${root}/tools/bench/package.json`);
const { Sandbox } = await import(require.resolve("e2b"));
const env = await Bun.file("/Volumes/app/origingame/.env").text();
const apiKey = env.match(/^E2B_API_KEY=(.+)$/m)?.[1]?.trim();
if (!apiKey) throw new Error("no E2B_API_KEY in /Volumes/app/origingame/.env");
const template = process.env.TEMPLATE ?? "origin-runner-probe";
const target = process.env.CARGO_TARGET_DIR ?? `${root}/target/agent-render`;
const linux = `${target}/x86_64-unknown-linux-gnu/release`;
const deps = `${linux}/deps`;
const newest = (name: string) =>
  readdirSync(deps)
    .filter((f) => new RegExp(`^${name}-[0-9a-f]+$`).test(f))
    .map((f) => `${deps}/${f}`)
    .sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)[0];
const testBin = newest("gfx");
if (!testBin) throw new Error(`no gfx test binary in ${deps}: build with --test gfx`);
const effectsBin = newest("effects");
if (!effectsBin) throw new Error(`no effects test binary in ${deps}: build with --test effects`);
const assets = `${root}/crates/og-tests/assets`;
const files = ["trickster.glb", "dropship.glb", ...readdirSync(`${assets}/fonts`).map((f) => `fonts/${f}`)];

const t0 = performance.now();
const sb = await Sandbox.create(template, { apiKey, timeoutMs: 1_200_000, metadata: { purpose: "og-gfx-vulkan-cpu" } });
console.log(`sandbox ${sb.sandboxId} (${template}) in ${(performance.now() - t0).toFixed(0)} ms`);
const logs = `${target}/vulkan-cpu`;
mkdirSync(logs, { recursive: true });
try {
  await sb.files.write([
    { path: "/tmp/og/og-gfx", data: await Bun.file(`${linux}/og-gfx`).arrayBuffer() },
    { path: "/tmp/og/gfx-test", data: await Bun.file(testBin).arrayBuffer() },
    { path: "/tmp/og/effects-test", data: await Bun.file(effectsBin).arrayBuffer() },
    ...(await Promise.all(files.map(async (f) => ({ path: `/tmp/og/assets/${f}`, data: await Bun.file(`${assets}/${f}`).arrayBuffer() })))),
  ]);
  const probe = await sb.commands.run("ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -1; nproc; (vulkaninfo --summary 2>/dev/null | grep -E 'deviceName|driverInfo' | head -2) || true");
  const [icd, ...about] = probe.stdout.trim().split("\n");
  console.log("sandbox:", icd, "|", about.join(" | "));
  const envs = { VK_ICD_FILENAMES: icd, OG_TEST_ASSETS: "/tmp/og/assets", OG_GFX_BACKEND: "vulkan", RUST_LOG: "warn" };
  const step = async (name: string, cmd: string, extra: Record<string, string> = {}) => {
    const t = performance.now();
    const r = await sb.commands.run(cmd, { timeoutMs: 900_000, envs: { ...envs, ...extra } }).catch((e: any) => e.result ?? { exitCode: -1, stdout: "", stderr: String(e) });
    const out = `$ ${cmd}\n${r.stdout}\n${r.stderr ? `stderr:\n${r.stderr}\n` : ""}exit ${r.exitCode}, ${((performance.now() - t) / 1000).toFixed(1)} s\n`;
    await Bun.write(`${logs}/${name}.txt`, out);
    console.log(`--- ${name}: exit ${r.exitCode} in ${((performance.now() - t) / 1000).toFixed(1)} s`);
    console.log(r.stdout.trim().split("\n").slice(-40).join("\n"));
    if (r.stderr?.trim()) console.log("stderr:", r.stderr.trim().split("\n").slice(-8).join("\n"));
    return r;
  };
  await step("capture", "chmod +x /tmp/og/og-gfx /tmp/og/gfx-test && cd /tmp/og && ./og-gfx --all --capture /tmp/og/refs");
  await step("tests", "cd /tmp/og && ./gfx-test --test-threads=2 --nocapture 2>&1 | grep -E '^(gfx |test |pixels|shadowed|partially|auto:|manual:|bgra|[0-9.]+% of|[0-9]+ pixels)|panicked|test result' ", { OG_TEST_REFERENCES: "/tmp/og/refs" });
  await step("effects", "chmod +x /tmp/og/effects-test && cd /tmp/og && ./effects-test --test-threads=2 --nocapture 2>&1 | grep -vE '^\\s*$' | tail -120");
  await step("timing", "cd /tmp/og && ./og-gfx gltf-reference sprites-20k particles --frames 10");
  const listing = await sb.commands.run("ls /tmp/og/refs/vulkan-cpu");
  const out = `${root}/crates/og-tests/references/vulkan-cpu`;
  mkdirSync(out, { recursive: true });
  for (const name of listing.stdout.trim().split("\n").filter((n: string) => n.endsWith(".png"))) {
    const png = (await sb.files.read(`/tmp/og/refs/vulkan-cpu/${name}`, { format: "bytes" })) as Uint8Array;
    await Bun.write(`${out}/${name}`, png);
  }
  const report = (await sb.files.read("/tmp/og/refs/report.txt")) as string;
  await Bun.write(`${logs}/report.txt`, report);
  console.log(`frames -> ${out}; logs -> ${logs}; sandbox time ${((performance.now() - t0) / 1000).toFixed(0)} s`);
} finally {
  await sb.kill();
}
