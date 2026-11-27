import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0.0.0.0 "Track": nested generation: 4 instances above its generator folded into one emitter
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n0.effect"
        },
        "faceCamera": true,
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "subtractive"
      },
      "at": 0.0,
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n1-0-0-0-0.effect"
        },
        "width": [
          4.0,
          4.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          0.0,
          0.7529
        ],
        "orbit": [
          4.0,
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
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n2-0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n2-0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n3.emitter"
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
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.0,
                1.0
              ],
              [
                1.0,
                1.0,
                0.2195,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n4.effect"
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
              0.25,
              0.8
            ],
            [
              1.0,
              0.8
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          480.0
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
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n5-0.effect"
        },
        "faceCamera": true,
        "scale": 1.0,
        "spin": [
          0.0,
          0.0,
          600.0
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
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "start": 150.0,
            "length": 240.0,
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n5-0.effect"
        },
        "faceCamera": true,
        "scale": 1.0,
        "spin": [
          0.0,
          0.0,
          600.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
            "colors": [
              [
                1.0,
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
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n6.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              7.0
            ],
            [
              0.0938,
              5.7437
            ],
            [
              0.2031,
              4.514
            ],
            [
              0.3125,
              3.5077
            ],
            [
              0.4219,
              2.6924
            ],
            [
              0.5312,
              2.0357
            ],
            [
              0.6562,
              1.4379
            ],
            [
              1.0,
              0.25
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
      "at": 0.3333,
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.5,
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
                0.0478,
                1.0
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n7.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.25
            ],
            [
              0.0938,
              1.5063
            ],
            [
              0.2031,
              2.736
            ],
            [
              0.3125,
              3.7423
            ],
            [
              0.4219,
              4.5576
            ],
            [
              0.5312,
              5.2143
            ],
            [
              0.6562,
              5.8121
            ],
            [
              1.0,
              7.0
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
      "at": 0.6667,
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6667,
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
                1.0
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n8.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.45
            ],
            [
              0.0781,
              1.3843
            ],
            [
              0.1562,
              2.1066
            ],
            [
              0.2344,
              2.645
            ],
            [
              0.3281,
              3.0881
            ],
            [
              0.4688,
              3.4387
            ],
            [
              0.8281,
              3.6719
            ],
            [
              1.0,
              4.05
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
      "at": 0.5,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.25,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.2195,
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
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n8-0.effect"
        },
        "faceCamera": true,
        "scale": 0.64,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.5,
      "life": 0.5833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n9-0.emitter"
        }
      },
      "at": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n10.emitter"
        }
      },
      "at": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/A_Salamander4/n11.emitter"
        }
      },
      "at": 0.4167
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.8621,
          0.8621
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
      "at": 0.4167,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.4167
    }
  ],
  "duration": 1.1667,
  "events": [
    {
      "name": "impact",
      "time": 0.4167
    }
  ]
});
