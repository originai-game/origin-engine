// The launcher's contract (packages/launcher/og.ts): what it accepts as
// signed, what an install leaves in the cache, which version a command runs.
import { afterAll, describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { createHash, generateKeyPairSync, randomBytes, sign } from "node:crypto";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { config, install, installed } from "../og.ts";

const fixtures = join(import.meta.dir, "fixtures");
// Retries in this file wait milliseconds, not seconds.
process.env.ORIGIN_ENGINE_RETRY_MS = "1";
const made: string[] = [];
const scratch = (prefix: string) => {
  const d = mkdtempSync(join(tmpdir(), prefix));
  made.push(d);
  return d;
};
afterAll(() => {
  for (const d of made) rmSync(d, { recursive: true, force: true });
});
const read = (f: string) => readFileSync(join(fixtures, f));
const fixtureKey = read("test.pub").toString().split("\n")[1]!;

/** A channel in a directory, signed with a fresh key, holding fake engines whose og prints its arguments and engine. */
function channel() {
  const dir = scratch("og-channel-");
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const id = randomBytes(8);
  const pubkey = Buffer.concat([Buffer.from("Ed"), id, publicKey.export({ format: "der", type: "spki" }).subarray(12)]).toString("base64");
  const target = config().target;
  const signFile = (file: string, comment: string) => {
    const sig = sign(null, createHash("blake2b512").update(readFileSync(file)).digest(), privateKey);
    const global = sign(null, Buffer.concat([sig, Buffer.from(comment)]), privateKey);
    writeFileSync(`${file}.minisig`, `untrusted comment: test\n${Buffer.concat([Buffer.from("ED"), id, sig]).toString("base64")}\ntrusted comment: ${comment}\n${global.toString("base64")}\n`);
  };
  const publish = (v: string, opts: { holds?: string; corrupt?: boolean; base?: string } = {}) => {
    const name = `origin-engine-${v}-${target}`;
    const stage = scratch("og-stage-");
    mkdirSync(join(stage, name, "bin"), { recursive: true });
    mkdirSync(join(stage, name, "engine"), { recursive: true });
    writeFileSync(join(stage, name, "origin-engine.json"), JSON.stringify({ build: { version: opts.holds ?? v, base: opts.base } }));
    writeFileSync(join(stage, name, "bin/og"), `#!/bin/sh\necho "engine ${v} $OG_ENGINE_PACKAGE $*"\n`);
    chmodSync(join(stage, name, "bin/og"), 0o755);
    mkdirSync(join(dir, v), { recursive: true });
    const archive = join(dir, v, `${name}.tar.gz`);
    spawnSync("tar", ["-czf", archive, "-C", stage, name]);
    rmSync(stage, { recursive: true, force: true });
    const sum = createHash("sha256").update(readFileSync(archive)).digest("hex");
    writeFileSync(join(dir, v, "SHA256SUMS"), `${opts.corrupt ? "0".repeat(64) : sum}  ${name}.tar.gz\n`);
    signFile(join(dir, v, "SHA256SUMS"), `origin-engine ${v}`);
  };
  const latest = (v: string) => {
    mkdirSync(join(dir, "latest"), { recursive: true });
    writeFileSync(join(dir, "latest/manifest.json"), JSON.stringify({ version: v }));
  };
  const cache = scratch("og-cache-");
  return { dir, pubkey, publish, latest, cache, c: { cache, downloads: dir, pubkey, target } };
}

const unix = process.platform !== "win32";

/**
 * The channel served over HTTP (docs/release.md §1 The launcher's events),
 * with hooks that fail a request, cut an archive halfway, and record the
 * Range each archive request asked for.
 */
function served(ch: ReturnType<typeof channel>) {
  const ranges: (string | null)[] = [];
  const plan = { fail: 0, cut: 0, status: 503 };
  const server = Bun.serve({
    port: 0,
    fetch(req) {
      const path = decodeURIComponent(new URL(req.url).pathname).slice(1);
      const file = join(ch.dir, path);
      if (!existsSync(file) || !statSync(file).isFile()) return new Response("no", { status: 404 });
      const bytes = readFileSync(file);
      if (!path.endsWith(".tar.gz")) return new Response(bytes);
      ranges.push(req.headers.get("range"));
      if (plan.fail > 0) {
        plan.fail--;
        return new Response("busy", { status: plan.status });
      }
      const from = Number(/^bytes=(\d+)-$/.exec(req.headers.get("range") ?? "")?.[1] ?? 0);
      const body = bytes.subarray(from);
      const headers: Record<string, string> = { "content-length": String(body.length) };
      if (from > 0) headers["content-range"] = `bytes ${from}-${bytes.length - 1}/${bytes.length}`;
      if (plan.cut > 0) {
        plan.cut--;
        // Half the body, then the connection breaks.
        const half = body.subarray(0, Math.floor(body.length / 2));
        const stream = new ReadableStream({
          start(c) {
            c.enqueue(half);
            setTimeout(() => c.error(new Error("cut")), 20);
          },
        });
        return new Response(stream, { status: from > 0 ? 206 : 200, headers });
      }
      return new Response(body, { status: from > 0 ? 206 : 200, headers });
    },
  });
  return { url: `http://127.0.0.1:${server.port}`, ranges, plan, stop: () => server.stop(true) };
}

describe.if(unix)("install events", () => {
  const og = join(import.meta.dir, "../og.ts");
  const run = async (ch: ReturnType<typeof channel>, downloads: string, args: string[], cwd: string, extra: Record<string, string> = {}) => {
    const p = Bun.spawn(["bun", og, ...args], {
      cwd,
      stdout: "pipe",
      stderr: "pipe",
      env: { ...process.env, ORIGIN_ENGINE_CACHE: ch.cache, ORIGIN_ENGINE_DOWNLOADS: downloads, ORIGIN_ENGINE_PUBKEY: ch.pubkey, OG_ENGINE_VERSION: "", ORIGIN_ENGINE_RETRY_MS: "10", ...extra },
    });
    const [stdout, stderr, code] = await Promise.all([new Response(p.stdout).text(), new Response(p.stderr).text(), p.exited]);
    const events = stdout.split("\n").filter((l) => l.startsWith("{")).map((l) => JSON.parse(l) as Record<string, any>);
    return { code, stdout, stderr, events };
  };
  const kinds = (events: Record<string, any>[]) => events.map((e) => e.event).filter((k, i, all) => k !== "download" || all[i - 1] !== "download");

  test("og engine install --json prints only the events: resolve, download, verify, unpack, installed; then installed from the cache", async () => {
    const ch = channel();
    ch.publish("3.1.0");
    const http = served(ch);
    try {
      const r = await run(ch, http.url, ["engine", "install", "3.1.0", "--json"], tmpdir());
      expect(r.code).toBe(0);
      expect(kinds(r.events)).toEqual(["resolve", "download", "verify", "unpack", "installed"]);
      const dist = join(ch.cache, "engines/3.1.0", ch.c.target);
      expect(r.events[0]).toEqual({ event: "resolve", version: "3.1.0", target: ch.c.target });
      const size = statSync(join(ch.dir, "3.1.0", `origin-engine-3.1.0-${ch.c.target}.tar.gz`)).size;
      expect(r.events.filter((e) => e.event === "download").at(-1)).toEqual({ event: "download", bytes: size, total: size });
      expect(r.events.at(-1)).toEqual({ event: "installed", version: "3.1.0", path: dist, cached: false });
      const again = await run(ch, http.url, ["engine", "install", "3.1.0", "--json"], tmpdir());
      expect(again.events).toEqual([{ event: "resolve", version: "3.1.0", target: ch.c.target }, { event: "installed", version: "3.1.0", path: dist, cached: true }]);
      // Nothing is left in tmp/ but the directory.
      expect(readdirSync(join(ch.cache, "tmp"))).toEqual([]);
    } finally {
      http.stop();
    }
  });

  test("a failed request is retried with a retry event, and a cut download resumes with a Range request", async () => {
    const ch = channel();
    ch.publish("3.2.0");
    const http = served(ch);
    try {
      http.plan.fail = 1;
      http.plan.cut = 1;
      const r = await run(ch, http.url, ["engine", "install", "3.2.0", "--json"], tmpdir());
      expect(r.code).toBe(0);
      const retries = r.events.filter((e) => e.event === "retry");
      expect(retries.map((e) => [e.attempt, e.max])).toEqual([[2, 5], [3, 5]]);
      expect(retries[0]!.reason).toContain("503");
      expect(retries[0]!.delayMs).toBe(10);
      expect(retries[1]!.delayMs).toBe(20);
      // 503, then the cut one from the start, then the rest from where it was cut.
      expect(http.ranges[0]).toBeNull();
      expect(http.ranges[1]).toBeNull();
      expect(http.ranges[2]).toMatch(/^bytes=[1-9]\d*-$/);
      expect(r.events.at(-1)!.event).toBe("installed");
      expect(JSON.parse(readFileSync(join(ch.cache, "engines/3.2.0", ch.c.target, "origin-engine.json"), "utf8")).build.version).toBe("3.2.0");
    } finally {
      http.stop();
    }
  });

  test("errors carry a stable code and whether to retry, exit 1, and leave no engine behind", async () => {
    const ch = channel();
    ch.publish("3.3.0", { corrupt: true });
    ch.publish("3.3.1");
    const http = served(ch);
    try {
      const error = async (args: string[], extra: Record<string, string> = {}) => {
        const r = await run(ch, http.url, args, tmpdir(), extra);
        expect(r.code).toBe(1);
        expect(r.stderr).toContain("og: ");
        return r.events.at(-1)!;
      };
      expect(await error(["engine", "install", "9.9.9", "--json"])).toMatchObject({ event: "error", code: "not_found", retryable: false });
      expect(await error(["engine", "install", "3.3.0", "--json"], { ORIGIN_ENGINE_RETRIES: "2" })).toMatchObject({ event: "error", code: "checksum", retryable: true });
      expect(await error(["engine", "install", "3.3.1", "--json"], { ORIGIN_ENGINE_PUBKEY: fixtureKey })).toMatchObject({ event: "error", code: "signature", retryable: false });
      http.plan.fail = 3;
      http.plan.status = 502;
      expect(await error(["engine", "install", "3.3.1", "--json"], { ORIGIN_ENGINE_RETRIES: "3" })).toMatchObject({ event: "error", code: "network", retryable: true });
      expect(await error(["engine", "install", "not-a-version", "--json"])).toMatchObject({ event: "error", code: "usage", retryable: false });
      expect(installed(ch.c)).toEqual([]);
    } finally {
      http.stop();
    }
  });

  test("in a project, install takes its pin; another command's install goes to stderr, or to stdout with --json", async () => {
    const ch = channel();
    ch.publish("3.4.0");
    ch.publish("3.4.1");
    const game = scratch("og-game-");
    writeFileSync(join(game, "project.ogt"), 'type project\nversion 1\n---\nengine "3.4.0"\n');
    const pinned = await run(ch, ch.dir, ["engine", "install", "--json"], game);
    expect(pinned.events.at(-1)).toMatchObject({ event: "installed", version: "3.4.0", cached: false });
    writeFileSync(join(game, "project.ogt"), 'engine "3.4.1"\n');
    const quiet = await run(ch, ch.dir, ["test", "."], game);
    expect(quiet.stdout.trim()).toStartWith("engine 3.4.1 ");
    expect(quiet.stderr.split("\n").filter((l) => l.startsWith("{")).map((l) => JSON.parse(l).event)).toEqual(["resolve", "download", "verify", "unpack", "installed"]);
    rmSync(join(ch.cache, "engines/3.4.1"), { recursive: true });
    const json = await run(ch, ch.dir, ["test", ".", "--json"], game);
    // The launcher's events, then the engine's own output (here the fake og's line, not JSON).
    const lines = json.stdout.trim().split("\n");
    expect(lines.slice(0, -1).map((l) => JSON.parse(l).event)).toEqual(["resolve", "download", "verify", "unpack", "installed"]);
    expect(lines.at(-1)).toStartWith("engine 3.4.1 ");
    expect(json.stderr).not.toContain("{");
  });
});
