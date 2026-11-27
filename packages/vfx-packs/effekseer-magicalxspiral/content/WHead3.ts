import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 4 "Smoke": ring 13 instances: the first 4 drawn
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n0.effect"
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
      "at": 0.0333,
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
                0.3587,
                0.0
              ],
              [
                1.0,
                0.5356,
                0.3587,
                0.3765
              ],
              [
                1.0,
                0.5356,
                0.3587,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n1.effect"
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n2.emitter"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n3.emitter"
        }
      },
      "at": 0.0667
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
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.1166,
                0.251
              ],
              [
                1.0,
                1.0,
                0.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n4.effect"
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
      "at": 0.0167,
      "life": 0.1667
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
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.1166,
                0.251
              ],
              [
                1.0,
                1.0,
                0.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n4.effect"
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
      "at": 0.0333,
      "life": 0.1667
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
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.1166,
                0.251
              ],
              [
                1.0,
                1.0,
                0.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n4.effect"
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
      "at": 0.05,
      "life": 0.1667
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
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.1166,
                0.251
              ],
              [
                1.0,
                1.0,
                0.0,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n4.effect"
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
      "at": 0.0667,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n5.emitter"
        }
      },
      "at": 0.0167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n6.emitter"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n7.emitter"
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
                0.3587,
                0.0
              ],
              [
                1.0,
                0.5356,
                0.3587,
                0.3765
              ],
              [
                1.0,
                0.5356,
                0.3587,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/WHead3/n8.effect"
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
      "at": 0.0833,
      "life": 0.25
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.5933,
          0.1661
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
      "at": 0.05,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.05
    }
  ],
  "duration": 0.5667,
  "events": [
    {
      "name": "impact",
      "time": 0.05
    }
  ]
});
