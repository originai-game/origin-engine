// Web video decoder & interactive-film presentation contracts:
// - AV1 Annex A.3 codec level selection across landscape, 9:16 portrait, and
//   2x packed-alpha streams (av1Codec).
// - WasmPlayer seek generation (`gen`) drops stale worker frames after a
//   backward seek without discarding normal forward playback frames.
// - VideoStreams.close frees decoders and clears player errors; VideoStreams.error
//   propagates decoder failures.

import { describe, expect, test } from "bun:test";
import { av1Codec, VideoStreams, WasmPlayer } from "../src/web/video.ts";

class FakeVideoFrame {
  readonly timestamp: number;
  readonly codedWidth: number;
  readonly codedHeight: number;
  closed = false;
  constructor(_data: Uint8Array, init: { codedWidth: number; codedHeight: number; timestamp: number }) {
    this.codedWidth = init.codedWidth;
    this.codedHeight = init.codedHeight;
    this.timestamp = init.timestamp;
  }
  close(): void {
    this.closed = true;
  }
}

describe("web video contracts", () => {
  test("av1Codec selects Annex A.3 levels for landscape, 9:16 portrait, and packed-alpha", () => {
    expect(av1Codec(1920, 1080)).toBe("av01.0.08M.08");
    expect(av1Codec(1080, 1920)).toBe("av01.0.08M.08");
    // 2x packed-alpha height exceeds Level 4.0 MaxPicSize (2,359,296) -> Level 5.0
    expect(av1Codec(1920, 2160)).toBe("av01.0.12M.08");
    expect(av1Codec(1080, 3840)).toBe("av01.0.12M.08");
    expect(av1Codec(3840, 2160)).toBe("av01.0.12M.08");
    // 4K packed-alpha exceeds Level 5.0 -> Level 6.0
    expect(av1Codec(3840, 4320)).toBe("av01.0.16M.08");
  });

  test("WasmPlayer seek generation discards stale frames on backward seek but keeps forward frames", () => {
    const prevVF = (globalThis as { VideoFrame?: unknown }).VideoFrame;
    (globalThis as { VideoFrame?: unknown }).VideoFrame = FakeVideoFrame;
    try {
      const posted: Array<Record<string, unknown>> = [];
      const worker = { postMessage: (msg: unknown) => { posted.push(msg as Record<string, unknown>); } };
      const player = new WasmPlayer(7, 1, 120, worker);
      const frameBuf = () => ({ width: 4, height: 4, i420: new ArrayBuffer(24) });

      // Initial want and normal forward steps must keep generation stable.
      expect(player.want(0, false)).toBeNull();
      const g0 = player.generation;
      expect(player.want(1, false)).toBeNull();
      expect(player.generation).toBe(g0);
      player.put(1, frameBuf(), g0);
      const f1 = player.want(1, false) as unknown as FakeVideoFrame | null;
      expect(f1?.timestamp).toBe(1);

      // Advance forward to frame 20, then seek backward to frame 5.
      player.want(2, false);
      expect(player.generation).toBe(g0);
      player.want(20, false);
      const gJump = player.generation;
      expect(gJump).toBeGreaterThan(g0);

      player.want(5, false);
      const gSeek = player.generation;
      expect(gSeek).toBeGreaterThan(gJump);
      expect(posted.at(-1)).toMatchObject({ type: "want", player: 7, frame: 5, gen: gSeek });

      // Stale in-flight frame from before the seek is discarded.
      player.put(5, frameBuf(), gJump);
      expect(player.want(5, false)).toBeNull();

      // Frame matching the new generation is shown.
      player.put(5, frameBuf(), gSeek);
      const f5 = player.want(5, false) as unknown as FakeVideoFrame | null;
      expect(f5?.timestamp).toBe(5);
    } finally {
      (globalThis as { VideoFrame?: unknown }).VideoFrame = prevVF;
    }
  });

  test("VideoStreams.close clears player state and error propagates fatal failures", () => {
    const streams = new VideoStreams({ name: "none", hardware: false, maxWidth: 0, maxHeight: 0, codecs: [] }, "og-av1.wasm");
    expect(streams.error(1, 10)).toBeNull();
    streams.failure = "decoder init failed";
    expect(streams.error(1, 10)).toBe("decoder init failed");
    streams.close(10);
    streams.failure = null;
    expect(streams.error(1, 10)).toBeNull();
  });
});
