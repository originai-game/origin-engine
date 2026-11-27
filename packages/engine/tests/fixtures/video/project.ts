// The fixture og-video.test.ts cooks and plays: assets/clips/count.mp4 is
// ffmpeg's testsrc2 (a moving pattern with its frame count burned in, every
// frame different) and a 440 Hz tone, 4 s at 30 fps, on a sprite at a 30 Hz
// tick, so tick 90 shows the source's frame 3 s in.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "video-fixture",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 30,
  "seed": 1
});
