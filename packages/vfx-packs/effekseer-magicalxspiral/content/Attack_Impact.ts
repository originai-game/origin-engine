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
                0.251
              ],
              [
                1.0,
                1.0,
                1.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.4
            ],
            [
              1.0,
              1.8
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.05,
      "life": 0.25
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n1.effect"
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
      "life": 0.0833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n2.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n3.emitter"
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
            "inner": 0.0,
            "band": 0.5,
            "start": 225.0,
            "length": 90.0,
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
                0.251
              ],
              [
                1.0,
                0.5356,
                1.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n4.effect"
        },
        "faceCamera": true,
        "scale": 1.0,
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
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n5.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n6.emitter"
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
            "inner": 0.2778,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n7.effect"
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
              2.7
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
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n8.emitter"
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
            "inner": 0.0,
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
                0.502
              ],
              [
                1.0,
                0.5356,
                0.5356,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Attack_Impact/n9.effect"
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
              1.5
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
          0.6909,
          0.6909
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
  "duration": 0.5667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
