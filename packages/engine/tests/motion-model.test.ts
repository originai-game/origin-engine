import { expect, test } from "bun:test";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: {
  serve(options: { port: number; hostname: string; fetch(request: Request): Response }): { port: number; stop(force: boolean): void };
  spawn(command: string[], options: { env: Record<string, string | undefined>; stdout: "ignore"; stderr: "pipe" }): {
    exited: Promise<number>; stderr: ReadableStream<Uint8Array>;
  };
};

test("development model installation verifies the pin; remote verification never trusts the cache", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-motion-model-"));
  const bytes = new TextEncoder().encode("pinned motion weights");
  const sha = createHash("sha256").update(bytes).digest("hex");
  const key = `motion/${sha}.safetensors`;
  const model = join(root, "motion/model.safetensors");
  let response: "valid" | "missing" | "corrupt" = "valid";
  const server = Bun.serve({ port: 0, hostname: "127.0.0.1", fetch(req) {
    if (response === "missing" || !new URL(req.url).pathname.endsWith(key)) {
      return new Response("<Error><Code>NoSuchKey</Code><Message>missing pin</Message></Error>", {
        status: 404, headers: { "content-type": "application/xml" },
      });
    }
    return new Response(response === "corrupt" ? new Uint8Array(bytes.length) : bytes);
  },
  });
  const env = {
    ...process.env,
    OG_MOTION_MODEL: "",
    OG_SHOWCASE_S3_ACCESS_KEY: "test",
    OG_SHOWCASE_S3_SECRET_KEY: "test",
    OG_SHOWCASE_BUCKET: "test",
    OG_SHOWCASE_S3_REGION: "us-east-1",
    OG_SHOWCASE_S3_ENDPOINT: `http://127.0.0.1:${server.port}`,
  };
  const run = async (command: string, override = "") => {
    const child = Bun.spawn(["bun", join(root, "scripts/motion-model.ts"), command], {
      env: { ...env, OG_MOTION_MODEL: override }, stdout: "ignore", stderr: "pipe",
    });
    return { code: await child.exited, error: await new Response(child.stderr).text() };
  };
  try {
    mkdirSync(join(root, "scripts"));
    copyFileSync(fileURLToPath(new URL("../scripts/motion-model.ts", import.meta.url)), join(root, "scripts/motion-model.ts"));
    writeFileSync(join(root, "motion.json"), JSON.stringify({ file: "motion/model.safetensors", key, sha256: sha, bytes: bytes.length }));
    expect(await run("ensure")).toEqual({ code: 0, error: "" });
    expect(new Uint8Array(readFileSync(model))).toEqual(bytes);
    writeFileSync(model, new Uint8Array(bytes.length));
    expect((await run("ensure")).code).toBe(0);
    expect(new Uint8Array(readFileSync(model))).toEqual(bytes);
    expect((await run("verify")).code).toBe(0);

    response = "missing";
    const missing = await run("verify");
    expect(missing.code).not.toBe(0);
    expect(missing.error).toContain(key);
    expect((await run("ensure")).code).toBe(0); // A verified local copy works offline.
    rmSync(model);
    expect((await run("ensure")).code).not.toBe(0);

    response = "corrupt";
    expect((await run("verify")).error).toContain("motion.json pins");
    const local = join(root, "override.safetensors");
    writeFileSync(local, new Uint8Array(bytes.length));
    expect((await run("ensure", local)).code).not.toBe(0);
    writeFileSync(local, bytes);
    expect((await run("ensure", local)).code).toBe(0);
    expect(new Uint8Array(readFileSync(model))).toEqual(bytes);
  } finally {
    server.stop(true);
    rmSync(root, { recursive: true, force: true });
  }
});
