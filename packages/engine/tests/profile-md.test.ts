// `og run --profile out.md` renders bun's JSON profile (src/node/profile-md.ts).
// A sample without a position counts at its function's own mapped line, never
// at the bundle; a function that recurses counts its stack once in its total.
import { expect, test } from "bun:test";
import { render, type CpuProfile } from "../src/node/profile-md.ts";

test("the markdown profile names each sample's function at its source line, recursion counted once", () => {
  const frame = (id: number, functionName: string, url: string, lineNumber: number, children: number[], positionTicks?: { line: number; ticks: number }[]) =>
    ({ id, callFrame: { functionName, url, lineNumber, columnNumber: 0 }, children, ...(positionTicks ? { positionTicks } : {}) });
  const game = "file:///games/walk/src/main.ts";
  const profile: CpuProfile = {
    nodes: [
      frame(1, "(root)", "", -1, [2]),
      // spawnWave starts on line 20 (0-based 19); two samples are on line 24, one has no position.
      frame(2, "spawnWave", game, 19, [3], [{ line: 24, ticks: 2 }]),
      frame(3, "spawnWave", game, 19, [4]),
      frame(4, "og_world_spawn", "", -1, []),
    ],
    startTime: 0,
    endTime: 5000,
    samples: [2, 2, 2, 3, 4],
    timeDeltas: [1000, 1000, 1000, 1000, 1000],
  };
  const md = render(profile);
  const row = (name: string) => md.split("\n").find((l) => l.startsWith("| ") && l.includes(`\`${name}\``) && /^\| [\d.]+% \| [\d.]+ms \| [\d.]+% \|/.test(l));
  // Self: four samples of spawnWave over two nodes; total: the five under the outer call, not 5 + 2.
  expect(row("spawnWave")).toBe("| 80.0% | 4.0ms | 100.0% | 5.0ms | `spawnWave` | `/games/walk/src/main.ts:20` |");
  expect(row("og_world_spawn")).toBe("| 20.0% | 1.0ms | 20.0% | 1.0ms | `og_world_spawn` | `[native code]` |");
  // The sample without a position is at the line the function starts on.
  expect(md).toContain("**Hot lines:**\n- `/games/walk/src/main.ts:20` (2)\n- `/games/walk/src/main.ts:24` (2)\n");
  expect(md).not.toContain("file://");
});
