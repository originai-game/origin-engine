// Spike §10.2: the native binary rendering the same frame headless inside today's E2B
// probe sandbox (Mesa lavapipe, no GPU). Billable; reads E2B_API_KEY from the platform .env.
import { Sandbox } from "e2b";
const env = await Bun.file("/Volumes/app/origingame/.env").text();
const apiKey = env.match(/^E2B_API_KEY=(.+)$/m)?.[1]?.trim();
if (!apiKey) throw new Error("no E2B_API_KEY in /Volumes/app/origingame/.env");
const template = process.env.TEMPLATE ?? "origin-runner-probe";
const root = "/Volumes/app/origin-engine";
const t0 = performance.now();
const sb = await Sandbox.create(template, { apiKey, timeoutMs: 600_000, metadata: { purpose: "og-spike" } });
console.log(`sandbox ${sb.sandboxId} (${template}) in ${(performance.now() - t0).toFixed(0)} ms`);
try {
  await sb.files.write([
    { path: "/tmp/spike/og-spike", data: await Bun.file(`${root}/target/x86_64-unknown-linux-gnu/release/og-spike`).arrayBuffer() },
    { path: "/tmp/spike/trickster.glb", data: await Bun.file(`${root}/crates/og-spike/web/models/trickster.glb`).arrayBuffer() },
    { path: "/tmp/spike/dropship.glb", data: await Bun.file(`${root}/crates/og-spike/web/models/dropship.glb`).arrayBuffer() },
  ]);
  const icd = (await sb.commands.run("ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -1; nproc; vulkaninfo --summary 2>/dev/null | grep -E 'deviceName|apiVersion' | head -2")).stdout.trim();
  console.log("sandbox:", icd.replace(/\n/g, " | "));
  const icdPath = icd.split("\n")[0];
  const run = await sb.commands.run("chmod +x /tmp/spike/og-spike && cd /tmp/spike && ./og-spike --out frame.png --count 25 --frames 30 trickster.glb dropship.glb", { timeoutMs: 300_000, envs: { VK_ICD_FILENAMES: icdPath, RUST_LOG: "info", WGPU_BACKEND: "vulkan" } });
  console.log(run.stdout.trim()); if (run.stderr.trim()) console.log("stderr:", run.stderr.trim().split("\n").slice(-6).join("\n"));
  const png = (await sb.files.read("/tmp/spike/frame.png", { format: "bytes" })) as Uint8Array;
  await Bun.write(`${root}/.spike-out/sandbox.png`, png);
  console.log(`sandbox frame: ${png.length} bytes -> .spike-out/sandbox.png`);
} finally {
  await sb.kill();
}
