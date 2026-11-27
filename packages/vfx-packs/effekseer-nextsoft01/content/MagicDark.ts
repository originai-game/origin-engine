import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicDark/n0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicDark/n1-0.effect"
        },
        "scale": 3.0,
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          0.586,
          0.145,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicDark/n2.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.6166,
          0.2083,
          1.0
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "distance": 12.0
      },
      "at": 0.0,
      "life": 0.3,
      "offset": [
        0.0,
        0.1111,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 2.05,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
