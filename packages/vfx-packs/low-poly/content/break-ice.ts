import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Ice Break: centred on the ice.
export default file.vfx({
  "duration": 1.4,
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
        "asset": ref("base/vfx/low-poly/emitters/ice-shards")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/ice-glints")
      }
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.12
      }
    }
  ]
});
