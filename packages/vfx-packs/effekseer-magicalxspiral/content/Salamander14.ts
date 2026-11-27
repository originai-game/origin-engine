import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 3.0.0.0.0 "Track": nested generation: 4 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "colors": [
              [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander14/n0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          300.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander14/n1-0.emitter"
        }
      },
      "at": 0.0333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander14/n2-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander14/n3-0-0-0-0.effect"
        },
        "width": [
          2.0,
          2.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          0.0,
          0.7529
        ],
        "orbit": [
          2.0,
          5.8333
        ]
      },
      "at": 0.0,
      "rotate": [
        45.0,
        0.0,
        0.0
      ],
      "life": 0.2333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6111,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.5356,
                0.5356,
                0.0
              ],
              [
                1.0,
                0.5356,
                0.5356,
                0.3765
              ],
              [
                1.0,
                0.5356,
                0.5356,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Salamander14/n4.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.24
            ],
            [
              1.0,
              0.0
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
      "life": 0.1667
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.2195,
          0.0
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
      "at": 0.0333,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0333
    }
  ],
  "duration": 0.8333,
  "events": [
    {
      "name": "impact",
      "time": 0.0333
    }
  ]
});
