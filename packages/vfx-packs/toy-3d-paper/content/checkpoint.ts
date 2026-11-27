import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Checkpoint: on the ground at the flag.
export default file.vfx({
  "duration": 2,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "elements": [
    {
      "life": 0.8,
      "offset": [
        0,
        0.02,
        0
      ],
      "rotate": [
        -90,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.84,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              1,
              1.4
            ]
          ]
        },
        "color": [
          0.75,
          0.522,
          0.115,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "at": 0.15,
      "life": 0.8,
      "offset": [
        0,
        0.02,
        0
      ],
      "rotate": [
        -90,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.84,
            "segments": 20
          }
        },
        "effect": ref("base/vfx/toy-3d-paper/fade"),
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.2
            ],
            [
              1,
              1.1
            ]
          ]
        },
        "color": [
          0.868,
          0.804,
          0.712,
          1
        ],
        "blend": "alpha"
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/check-confetti-a")
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/check-confetti-b")
      }
    },
    {
      "offset": [
        0,
        0.1,
        0
      ],
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/toy-3d-paper/emitters/check-confetti-c")
      }
    }
  ]
});
