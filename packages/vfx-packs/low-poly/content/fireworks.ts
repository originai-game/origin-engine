import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Fireworks: on the ground: it bursts 6 m up.
export default file.vfx({
  "duration": 3,
  "events": [
    {
      "name": "impact",
      "time": 0.7
    }
  ],
  "elements": [
    {
      "until": 0.7,
      "velocity": [
        0,
        8.6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/fw-rise")
      }
    },
    {
      "at": 0.7,
      "offset": [
        -0.8,
        6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/fw-burst-0")
      }
    },
    {
      "at": 0.82,
      "offset": [
        0,
        6.5,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/fw-burst-1")
      }
    },
    {
      "at": 0.94,
      "offset": [
        0.8,
        6,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/fw-burst-2")
      }
    },
    {
      "at": 0.7,
      "life": 0.8,
      "offset": [
        0,
        6,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.644,
          0.042
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              6
            ],
            [
              1,
              0
            ]
          ]
        },
        "distance": 10
      }
    }
  ]
});
