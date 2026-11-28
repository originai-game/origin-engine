import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Crate Break: centred on the crate.
export default file.vfx({
  "duration": 1.6,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/crate-planks")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/crate-splinters")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/crate-puffs")
      }
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.15
      }
    }
  ]
});
