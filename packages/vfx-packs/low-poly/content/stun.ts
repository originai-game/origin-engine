import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Stunned: above the head.
export default file.vfx({
  "duration": 1.2,
  "looping": true,
  "elements": [
    {
      "life": 1.2,
      "rotate": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "trail",
        "lifetime": 0.3,
        "width": [
          0.05,
          0
        ],
        "orbit": [
          0.24,
          0.85
        ],
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          1,
          0.888,
          0.254,
          1
        ]
      }
    },
    {
      "life": 1.2,
      "rotate": [
        0,
        120,
        0
      ],
      "node": {
        "kind": "trail",
        "lifetime": 0.3,
        "width": [
          0.05,
          0
        ],
        "orbit": [
          0.24,
          0.85
        ],
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          1,
          0.888,
          0.254,
          1
        ]
      }
    },
    {
      "life": 1.2,
      "rotate": [
        0,
        240,
        0
      ],
      "node": {
        "kind": "trail",
        "lifetime": 0.3,
        "width": [
          0.05,
          0
        ],
        "orbit": [
          0.24,
          0.85
        ],
        "effect": ref("base/vfx/low-poly/solid"),
        "color": [
          1,
          0.888,
          0.254,
          1
        ]
      }
    },
    {
      "node": {
        "kind": "emitter",
        "asset": ref("base/vfx/low-poly/emitters/stun-stars")
      }
    }
  ]
});
