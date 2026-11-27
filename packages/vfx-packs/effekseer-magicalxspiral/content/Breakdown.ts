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
            "inner": 0.0,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.502
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n0.effect"
        },
        "faceCamera": true,
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.0,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.7,
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                1.0,
                0.502
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n1.effect"
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
              0.0781,
              0.6488
            ],
            [
              0.1562,
              1.1504
            ],
            [
              0.2344,
              1.5243
            ],
            [
              0.3281,
              1.832
            ],
            [
              0.4688,
              2.0755
            ],
            [
              0.8281,
              2.2374
            ],
            [
              1.0,
              2.5
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
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n2.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n3.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n4.emitter"
        }
      },
      "at": 0.0333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n5.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n6.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n7.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.7519,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7519,
                1.0,
                0.6275
              ],
              [
                1.0,
                0.7519,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Breakdown/n8.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.0
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
      "life": 0.1167
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.6129,
          0.9334
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
  "duration": 0.5167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
