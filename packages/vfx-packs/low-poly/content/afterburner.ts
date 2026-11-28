import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Toy Jet Flame: at the nozzle, +y out of it.
export default file.vfx({
  "duration": 1,
  "looping": true,
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/jet-flames")
      }
    },
    {
      "offset": [
        0,
        0.5,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/jet-puffs")
      }
    },
    {
      "life": 1,
      "offset": [
        0,
        0.3,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.254,
          0.014
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.6
            ],
            [
              1,
              1.6
            ]
          ]
        },
        "distance": 2.5
      }
    }
  ]
});
