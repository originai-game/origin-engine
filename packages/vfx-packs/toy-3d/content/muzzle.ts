import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Toy Blaster: at the muzzle, +y along the barrel.
export default file.vfx({
  "duration": 0.7,
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
        "asset": ref("base/vfx/toy-3d/emitters/muzzle-flame")
      }
    },
    {
      "at": 0.02,
      "offset": [
        0,
        0.04,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/muzzle-puffs")
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d/emitters/muzzle-stars")
      }
    },
    {
      "life": 0.08,
      "offset": [
        0,
        0.15,
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
              1.5
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 2
      }
    }
  ]
});
