import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "height": [
              0.5,
              -0.5
            ],
            "colors": [
              [
                0.5234,
                0.5356,
                1.0,
                0.0
              ],
              [
                0.5234,
                0.5356,
                1.0,
                0.3765
              ],
              [
                0.5234,
                0.5356,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Topwater1/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
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
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Topwater1/n1-0-0.emitter"
        }
      },
      "at": 0.0,
      "velocity": [
        0.0,
        6.0,
        0.0
      ],
      "acceleration": [
        0.0,
        -36.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "height": [
              0.5,
              -0.5
            ],
            "colors": [
              [
                0.5234,
                0.5356,
                1.0,
                0.0
              ],
              [
                0.5234,
                0.5356,
                1.0,
                0.3765
              ],
              [
                0.5234,
                0.5356,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Topwater1/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.9836,
          0.9937,
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
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 0.65,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
