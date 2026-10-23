// Two Play pages in one room on the web host (docs/design.md §2.8, §2.10):
// each page is the real engine Worker and shell (tests/support/room-page.ts,
// one process each: the Worker owns the global scope) on the native host,
// playing the arena archetype, and its rooms are the Player's room.* (a
// stand-in speaking og-net's protocol to an `og relay` this test starts).
// Neither game calls roomJoin: the page's Play link (`?room=<id>`, the
// shell's setRooms(rooms, link)) puts it in the room.
//
// The ways this could fail, each asserted below: the Play link is not read
// (the pages play alone: roomInfo state `none`); the engine's join does not
// leave a page driven by a harness's `advance`/`frame` calls (only the page's
// own frame loop pumped the room); lockstep does not start, seats the pages
// wrongly, or parts the worlds (netHash differs at the same tick).
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { mkdtempSync, rmSync } from "node:fs";
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
};

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..", "..");

type Page = {
  tick: number;
  joined: boolean;
  net: {
    hash: string;
    status: { running: boolean; peer: number; startTick: number | null; desync: number | null; seats: { peer: number; seat: number }[] };
    room: { state: string; room: string; link: string; host: string };
  };
};

test("two Worker-hosted pages joined by a Play link play one duel in lockstep through a relay", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-web-rooms-"));
  const relay = Bun.spawn([ogBin(), "relay", "--port", "0"], { cwd: repo, stdout: "pipe", stderr: "pipe" });
  try {
    const arena = join(repo, "templates/archetypes/arena");
    const cooked = Bun.spawnSync([ogBin(), "cook", arena], { cwd: repo });
    expect(cooked.exitCode).toBe(0);
    const game = join(dir, "arena.js");
    expect((await bundleGame(join(repo, "templates/archetypes/arena/src/main.ts"), game, "native")).ok).toBe(true);
    const reader = relay.stdout.getReader();
    let text = "";
    while (!/:(\d+)\//.test(text)) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og relay exited: ${text}`);
      text += new TextDecoder().decode(value);
    }
    const port = /:(\d+)\//.exec(text)![1]!;
    const pages = await Promise.all(
      ["1", "2"].map(async (bot) => {
        const p = Bun.spawn(["bun", join(here, "support/room-page.ts"), game, arena, port, "duel-web", bot, "300"], { cwd: repo, stdout: "pipe", stderr: "pipe" });
        const [code, out, err] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
        if (code !== 0) throw new Error(`page ${bot} exited ${code}: ${err}${out}`);
        return JSON.parse(out.trim().split("\n").pop()!) as Page;
      }),
    );
    for (const p of pages) {
      expect(p.joined).toBe(true);
      expect(p.net.room).toMatchObject({ state: "in", room: "duel-web", link: "duel-web", host: "player" });
      expect(p.tick).toBe(300);
      expect(p.net.status.running).toBe(true);
      expect(p.net.status.desync).toBeNull();
      expect(p.net.status.seats.map((s) => [s.peer, s.seat])).toEqual([
        [0, 0],
        [1, 1],
      ]);
    }
    expect(pages.map((p) => p.net.status.peer).sort()).toEqual([0, 1]);
    expect(pages[1]!.net.hash).toBe(pages[0]!.net.hash);
  } finally {
    relay.kill();
    rmSync(dir, { recursive: true, force: true });
  }
}, 300_000);
