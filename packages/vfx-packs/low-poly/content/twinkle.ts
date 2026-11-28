import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Twinkle: on what sparkles.
export default file.vfx({
  "duration": 1.6,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/twinkle-stars")
      }
    }
  ]
});
