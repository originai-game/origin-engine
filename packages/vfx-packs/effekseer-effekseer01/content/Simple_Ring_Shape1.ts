import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 3 "Octagon": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 4 "Square": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 5 "Triangle": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 6 "Ellipse1": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 7 "Ellipse2": a ring whose thickness changes over its life: its middle thickness kept
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n0.effect"
        },
        "scale": 4.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.1,
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n1.effect"
        },
        "scale": 4.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.9,
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n2.effect"
        },
        "scale": 4.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 2.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.8333
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.125,
              0.9375
            ],
            [
              0.25,
              1.75
            ],
            [
              0.375,
              2.4375
            ],
            [
              0.5,
              3.0
            ],
            [
              0.625,
              3.4375
            ],
            [
              0.75,
              3.75
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 3.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.1667
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.125,
              0.9375
            ],
            [
              0.25,
              1.75
            ],
            [
              0.375,
              2.4375
            ],
            [
              0.5,
              3.0
            ],
            [
              0.625,
              3.4375
            ],
            [
              0.75,
              3.75
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 4.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.1667
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.125,
              0.9375
            ],
            [
              0.25,
              1.75
            ],
            [
              0.375,
              2.4375
            ],
            [
              0.5,
              3.0
            ],
            [
              0.625,
              3.4375
            ],
            [
              0.75,
              3.75
            ],
            [
              1.0,
              4.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 5.6667,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.1667
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.25,
              1.2374
            ],
            [
              0.375,
              1.7236
            ],
            [
              0.5,
              2.1213
            ],
            [
              0.625,
              2.4307
            ],
            [
              0.75,
              2.6517
            ],
            [
              0.875,
              2.7842
            ],
            [
              1.0,
              2.8284
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 7.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.1667
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
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Ring_Shape1/n7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.8284
            ],
            [
              0.1094,
              3.1383
            ],
            [
              0.2344,
              3.2199
            ],
            [
              0.5156,
              2.7636
            ],
            [
              0.875,
              1.3041
            ],
            [
              0.9688,
              0.6232
            ],
            [
              0.9844,
              0.4369
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
        "blend": "alpha"
      },
      "at": 8.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 1.1667
    }
  ],
  "duration": 9.5,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
