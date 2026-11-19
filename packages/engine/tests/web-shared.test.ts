// Two pages in one shared world on the web host (docs/design.md §2.12),
// self-hosted with no platform: an `og world-server --auth hs256` this test
// starts, and a token endpoint (the test's own, signing with the same
// secret) behind each page's tokenEndpoint session provider. Each page is
// the real engine Worker and shell (tests/support/shared-page.ts, one
// process each) on the native host, running showcase/shared-walk; the
// Worker opens the connection itself. The endpoint also names a
// WebTransport: the first page carries it (a stand-in over the server's
// WebSocket, as bun has none), the second's never opens and falls back to
// the WebSocket. A third page has no session provider.
//
// The ways this could fail, each asserted below: the page's provider never
// reaches the Worker, or the Worker never carries the engine's `connect`
// (the pages stay `connecting`); the token does not cross or the server
// refuses it (`refused`); the snapshots do not reach the engine (nobody in
// sight); a page with no provider waits forever instead of ending
// `unavailable`; WebTransport frames are cut or joined wrongly, a message
// other than a proposal leaves as a datagram or a proposal as a frame, or
// a WebTransport that fails to open is not followed by the WebSocket.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { bundleGame } from "../src/node/build.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number };
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): {
    exited: Promise<number>;
    stdout: ReadableStream<Uint8Array>;
    stderr: ReadableStream<Uint8Array>;
    kill(): void;
  };
  serve(o: { port: number; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
};

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..", "..");
const SECRET = "a-test-secret-of-at-least-32-bytes-long";

type Status = { state: string; host: string; inSight: number; reason: string; corrections: number; player: string; transport: string };
type Wt = { opened: number; datagramsOut: number; datagramsIn: number; misrouted: number };

const b64url = (bytes: Uint8Array): string => btoa(String.fromCharCode(...bytes)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");

async function hs256(claims: Record<string, unknown>): Promise<string> {
  const key = await crypto.subtle.importKey("raw", new TextEncoder().encode(SECRET), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const part = (o: unknown) => b64url(new TextEncoder().encode(JSON.stringify(o)));
  const body = `${part({ alg: "HS256", typ: "JWT" })}.${part(claims)}`;
  const mac = await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(body));
  return `${body}.${b64url(new Uint8Array(mac))}`;
}

test("two Worker-hosted pages see each other through a self-hosted world server and token endpoint", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-web-shared-"));
  const secret = join(dir, "secret");
  writeFileSync(secret, SECRET);
  const server = Bun.spawn([ogBin(), "world-server", "--listen", "127.0.0.1:0", "--auth", "hs256", "--secret-file", secret, "--audience", "shared-walk"], { cwd: repo, stdout: "pipe", stderr: "pipe" });
  let endpoint: { port: number; stop(): void } | undefined;
  try {
    const walk = join(repo, "showcase/shared-walk");
    expect(Bun.spawnSync([ogBin(), "cook", walk], { cwd: repo }).exitCode).toBe(0);
    const game = join(dir, "walk.js");
    expect((await bundleGame(join(walk, "src/main.ts"), game, "native")).ok).toBe(true);
    const reader = server.stdout.getReader();
    let text = "";
    while (!/ws:\/\/[\d.]+:(\d+)/.test(text)) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og world-server exited: ${text}`);
      text += new TextDecoder().decode(value);
    }
    const url = `ws://127.0.0.1:${/ws:\/\/[\d.]+:(\d+)/.exec(text)![1]}/`;
    let n = 0;
    endpoint = Bun.serve({
      port: 0,
      async fetch(req) {
        const ask = (await req.json()) as { world: string };
        const now = Math.floor(Date.now() / 1000);
        const sub = `p${++n}`;
        return Response.json({
          url,
          token: await hs256({ aud: "shared-walk", sub, name: sub, world: ask.world, iat: now, exp: now + 300 }),
          webtransport: { url: "https://127.0.0.1:1/", certHashes: ["ab".repeat(32)] },
        });
      },
    });
    const at = `http://127.0.0.1:${endpoint.port}/token`;
    const pages = await Promise.all(
      ([[at, `wt=${url}`], [at, "wt-fail"], ["none", ""]] as const).map(async ([provider, transport]) => {
        const p = Bun.spawn(["bun", join(here, "support/shared-page.ts"), game, walk, provider, provider === "none" ? "30" : "480", transport], { cwd: repo, stdout: "pipe", stderr: "pipe" });
        const [code, out, err] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
        if (code !== 0) throw new Error(`page exited ${code}: ${err}${out}`);
        return JSON.parse(out.trim().split("\n").pop()!) as { tick: number; maxInSight: number; shared: Status; wt: Wt };
      }),
    );
    const [a, b, none] = pages as [(typeof pages)[0], (typeof pages)[0], (typeof pages)[0]];
    for (const p of [a, b]) {
      expect(p.shared).toMatchObject({ state: "joined", host: "endpoint", inSight: 1, corrections: 0 });
    }
    expect(new Set([a.shared.player, b.shared.player])).toEqual(new Set(["p1", "p2"]));
    expect(a.shared.transport).toBe("wt");
    expect(a.wt.opened).toBe(1);
    expect(a.wt.misrouted).toBe(0);
    expect(a.wt.datagramsOut).toBeGreaterThan(50);
    expect(a.wt.datagramsIn).toBeGreaterThan(50);
    expect(b.shared.transport).toBe("ws");
    expect(b.wt.datagramsOut).toBe(0);
    expect(none.shared.state).toBe("refused");
    expect(none.shared.host).toBe("none");
    expect(none.shared.reason).toStartWith("unavailable");
  } finally {
    endpoint?.stop();
    server.kill();
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
