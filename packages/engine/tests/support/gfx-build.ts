// Cargo's "fresh" flag checks inputs, not the integrity of a cached output.
// Both browser gfx suites use this one build/receipt path.
import { createHash } from "node:crypto";
import { existsSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";

export function buildGfxWasm(run: (command: string[]) => string, targetDir: string, profile: string): string {
  const build = ["cargo", "build", "--profile", profile, "--target", "wasm32-unknown-unknown", "--target-dir", targetDir, "-p", "og-tests-web", "--lib", "--message-format=json-render-diagnostics"];
  for (let attempt = 0; attempt < 2; attempt++) {
    const artifact = run(build).split("\n")
      .filter((line) => line.startsWith("{"))
      .map((line) => JSON.parse(line) as { reason: string; target?: { name: string }; filenames?: string[]; fresh: boolean })
      .find((message) => message.reason === "compiler-artifact" && message.target?.name === "og_tests_web");
    const raw = artifact?.filenames?.find((file) => file.endsWith(".wasm"));
    if (!raw || !artifact) throw new Error("cargo built no og_tests_web.wasm");
    const bytes = readFileSync(raw);
    const valid = [0, 97, 115, 109, 1, 0, 0, 0].every((byte, i) => bytes[i] === byte);
    const hash = createHash("sha256").update(bytes).digest("hex");
    const receipt = `${raw}.sha256`;
    if (valid && (!artifact.fresh || (existsSync(receipt) && readFileSync(receipt, "utf8") === hash))) {
      if (!artifact.fresh) {
        const pending = `${receipt}.${process.pid}.tmp`;
        try {
          writeFileSync(pending, hash);
          renameSync(pending, receipt);
        } finally {
          rmSync(pending, { force: true });
        }
      }
      return raw;
    }
    if (!artifact.fresh || attempt > 0) throw new Error(`invalid or unverified wasm build product: ${raw}`);
    console.error(`discarding invalid or unverified cached wasm product: ${raw}`);
    // Invalidate only this package/profile. Cargo must relink before a new
    // hash can attest its output; never bless a fresh but unverified file.
    run(["cargo", "clean", "--profile", profile, "--target", "wasm32-unknown-unknown", "--target-dir", targetDir, "-p", "og-tests-web"]);
  }
  throw new Error("cargo did not produce a verified og_tests_web.wasm");
}
