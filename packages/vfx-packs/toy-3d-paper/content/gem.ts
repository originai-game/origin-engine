import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Gem Collect: where the gem was.
export default file.vfx({
  "duration": 1.1,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "life": 0.5,
      "velocity": [
        0,
        1.4,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": ref("base/vfx/toy-3d-paper/models/gemFlat")
        },
        "effect": ref("base/vfx/toy-3d-paper/solid"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.3
            ],
            [
              0.4,
              0.36
            ],
            [
              1,
              0
            ]
          ]
        },
        "spin": [
          0,
          900,
          0
        ],
        "color": [
          0.061,
          0.436,
          0.374,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.4,
      "offset": [
        0,
        0.6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/gem-bits")
      }
    },
    {
      "at": 0.4,
      "life": 0.25,
      "offset": [
        0,
        0.6,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.1
            ],
            [
              1,
              0.5
            ]
          ]
        },
        "color": [
          0.468,
          0.739,
          0.88,
          1
        ],
        "blend": "alpha"
      }
    }
  ]
});
