import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "リング": ring 5 instances: the first 4 drawn
// effekseer: node 1 "ギザギザリング": ring 5 instances: the first 4 drawn
// effekseer: node 1 "ギザギザリング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0 "煙": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8656,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.0278,
                0.3922
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.2344,
              5.7721
            ],
            [
              0.3125,
              6.8939
            ],
            [
              0.3906,
              7.7152
            ],
            [
              0.4688,
              8.2572
            ],
            [
              0.5469,
              8.5728
            ],
            [
              0.875,
              9.1718
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8656,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.0278,
                0.3922
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.2344,
              5.7721
            ],
            [
              0.3125,
              6.8939
            ],
            [
              0.3906,
              7.7152
            ],
            [
              0.4688,
              8.2572
            ],
            [
              0.5469,
              8.5728
            ],
            [
              0.875,
              9.1718
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8656,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.0278,
                0.3922
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.2344,
              5.7721
            ],
            [
              0.3125,
              6.8939
            ],
            [
              0.3906,
              7.7152
            ],
            [
              0.4688,
              8.2572
            ],
            [
              0.5469,
              8.5728
            ],
            [
              0.875,
              9.1718
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8656,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.0278,
                0.3922
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.2344,
              5.7721
            ],
            [
              0.3125,
              6.8939
            ],
            [
              0.3906,
              7.7152
            ],
            [
              0.4688,
              8.2572
            ],
            [
              0.5469,
              8.5728
            ],
            [
              0.875,
              9.1718
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.005,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 9.0977,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.1275,
                0.7843
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.3633
            ],
            [
              0.1562,
              0.6442
            ],
            [
              0.2344,
              0.8536
            ],
            [
              0.3281,
              1.0259
            ],
            [
              0.4688,
              1.1623
            ],
            [
              0.8281,
              1.2529
            ],
            [
              1.0,
              1.4
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 9.0977,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.1275,
                0.7843
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.3633
            ],
            [
              0.1562,
              0.6442
            ],
            [
              0.2344,
              0.8536
            ],
            [
              0.3281,
              1.0259
            ],
            [
              0.4688,
              1.1623
            ],
            [
              0.8281,
              1.2529
            ],
            [
              1.0,
              1.4
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 9.0977,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.1275,
                0.7843
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.3633
            ],
            [
              0.1562,
              0.6442
            ],
            [
              0.2344,
              0.8536
            ],
            [
              0.3281,
              1.0259
            ],
            [
              0.4688,
              1.1623
            ],
            [
              0.8281,
              1.2529
            ],
            [
              1.0,
              1.4
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 9.0977,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.949,
                0.8592,
                0.1275,
                0.7843
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0781,
              0.3633
            ],
            [
              0.1562,
              0.6442
            ],
            [
              0.2344,
              0.8536
            ],
            [
              0.3281,
              1.0259
            ],
            [
              0.4688,
              1.1623
            ],
            [
              0.8281,
              1.2529
            ],
            [
              1.0,
              1.4
            ]
          ]
        },
        "color": [
          1.25,
          1.25,
          1.25,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.005,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.5167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow3/n2-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9574,
          0.6959
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
  "duration": 0.5832,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
