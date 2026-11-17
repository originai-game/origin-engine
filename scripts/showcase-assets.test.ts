import { expect, test } from "bun:test";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const script = join(import.meta.dir, "showcase-assets.ts");

test("a project's pins install verified from the public bucket, fall back to the private one and never accept other bytes", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-showcase-assets-"));
  const project = join(root, "first-light");
  const files: Record<string, Uint8Array> = {
    "assets/models/dog.glb": new TextEncoder().encode("a pinned model"),
    "assets/terrain/height.png": new TextEncoder().encode("a pinned height map"),
  };
  const sha = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  const keyOf = (file: string) => `showcase/first-light/${sha(files[file]!)}${file.slice(file.lastIndexOf("."))}`;
  // What each bucket answers: the public one under /pub, the private one (S3) elsewhere.
  const answers = { pub: "valid" as "valid" | "missing" | "corrupt", s3: "valid" as "valid" | "missing" | "corrupt" };
  const server = Bun.serve({ port: 0, hostname: "127.0.0.1", fetch(request) {
    const path = new URL(request.url).pathname;
    const mode = path.startsWith("/pub/") ? answers.pub : answers.s3;
    const file = Object.keys(files).find((f) => path.endsWith(keyOf(f)));
    if (mode === "missing" || !file) return new Response("<Error><Code>NoSuchKey</Code><Message>missing pin</Message></Error>", { status: 404, headers: { "content-type": "application/xml" } });
    return new Response(mode === "corrupt" ? new Uint8Array(files[file]!.length) : files[file]!);
  } });
  const run = async (...args: string[]) => {
    const child = Bun.spawn(["bun", script, ...args], {
      env: {
        ...process.env,
        OG_SHOWCASE_PUBLIC_URL: `http://127.0.0.1:${server.port}/pub`,
        OG_SHOWCASE_S3_ACCESS_KEY: "test",
        OG_SHOWCASE_S3_SECRET_KEY: "test",
        OG_SHOWCASE_BUCKET: "test",
        OG_SHOWCASE_S3_REGION: "us-east-1",
        OG_SHOWCASE_S3_ENDPOINT: `http://127.0.0.1:${server.port}`,
      },
      stdout: "ignore", stderr: "pipe",
    });
    return { code: await child.exited, error: await new Response(child.stderr).text() };
  };
  const local = (file: string) => join(project, file);
  const place = () => {
    for (const [file, bytes] of Object.entries(files)) {
      mkdirSync(join(local(file), ".."), { recursive: true });
      writeFileSync(local(file), bytes);
    }
  };
  const clear = () => {
    for (const file of Object.keys(files)) rmSync(local(file), { force: true });
  };
  try {
    mkdirSync(project, { recursive: true });
    writeFileSync(join(project, "project.ogt"), "type project\nversion 1\n---\n");

    // Pinning names each file by its hash under the project's name.
    place();
    expect((await run("pin", project, ...Object.keys(files))).code).toBe(0);
    const pins = (JSON.parse(readFileSync(join(project, "assets.json"), "utf8")) as { files: { file: string; key: string; url: string; bytes: number; sha256: string }[] }).files;
    expect(pins.map((p) => [p.file, p.key, p.bytes, p.sha256])).toEqual(Object.keys(files).map((f) => [f, keyOf(f), files[f]!.length, sha(files[f]!)]));
    expect(pins[0]!.url).toBe(`http://127.0.0.1:${server.port}/pub/${pins[0]!.key}`);

    // Verified local files need no bucket at all.
    answers.pub = answers.s3 = "missing";
    expect(await run("ensure", project)).toEqual({ code: 0, error: "" });

    // A missing file and a changed one of the same length are both fetched.
    answers.pub = answers.s3 = "valid";
    rmSync(local("assets/models/dog.glb"));
    writeFileSync(local("assets/terrain/height.png"), new Uint8Array(files["assets/terrain/height.png"]!.length));
    expect((await run("ensure", project)).code).toBe(0);
    for (const [file, bytes] of Object.entries(files)) expect(new Uint8Array(readFileSync(local(file)))).toEqual(bytes);

    // The private bucket answers when the public one does not, or answers other bytes.
    for (const pub of ["missing", "corrupt"] as const) {
      clear();
      answers.pub = pub;
      expect((await run("ensure", project)).code).toBe(0);
      expect(new Uint8Array(readFileSync(local("assets/models/dog.glb")))).toEqual(files["assets/models/dog.glb"]!);
    }

    // Other bytes from both are refused, the file named and nothing installed; verify reads the remote even with good local files.
    clear();
    answers.s3 = "corrupt";
    const refused = await run("ensure", project);
    expect(refused.code).not.toBe(0);
    expect(refused.error).toMatch(/assets\/(models\/dog\.glb|terrain\/height\.png)/);
    expect(Object.keys(files).some((f) => existsSync(local(f)))).toBe(false);
    place();
    expect((await run("verify", project)).code).not.toBe(0);
    answers.pub = answers.s3 = "valid";
    expect((await run("verify", project)).code).toBe(0);

    // A pin never writes outside its project.
    writeFileSync(join(project, "assets.json"), JSON.stringify({ files: [{ ...pins[0], file: "../escaped.glb" }] }));
    const outside = await run("ensure", project);
    expect(outside.code).not.toBe(0);
    expect(outside.error).toContain("../escaped.glb");
    expect(existsSync(join(root, "escaped.glb"))).toBe(false);
  } finally {
    server.stop(true);
    rmSync(root, { recursive: true, force: true });
  }
}, 60_000);
