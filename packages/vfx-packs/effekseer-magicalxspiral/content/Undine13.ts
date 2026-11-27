import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine13/model-sphere"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine13/n0.effect"
        },
        "scale": 1.5,
        "color": [
          0.5356,
          0.7593,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 3.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine13/model-sphere"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine13/n1.effect"
        },
        "scale": 1.45,
        "color": [
          0.2195,
          0.7519,
          0.7519,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 3.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Undine13/n2-0-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.7,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.3936,
          0.8001,
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
        -0.218,
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
  "duration": 3.9167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
