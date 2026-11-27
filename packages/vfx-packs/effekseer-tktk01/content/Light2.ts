import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "ギザギザ柱右": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.0 "ギザギザ柱右": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.2 "ギザギザ柱左": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.2 "ギザギザ柱左": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.0 "ギザギザ柱右": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.1.0 "ギザギザ柱右": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.2 "ギザギザ柱左": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.1.2 "ギザギザ柱左": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.2 "リング外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.3 "柱外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.5 "上昇リング": ring 10 instances: the first 4 drawn
// effekseer: node 0.5 "上昇リング": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.5 "上昇リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.6 "柱中心": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.7 "柱中心glow": a sprite scaled unevenly: drawn square at its mean
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 1.0,
            "height": [
              0.0,
              3.7321
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-0-0.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-0-1.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          -60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 1.0,
            "height": [
              0.0,
              3.7321
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-0-2.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          -60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-0-3.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 1.0,
            "height": [
              0.0,
              3.7321
            ],
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-1-0.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-1-1.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          -60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 1.0,
            "height": [
              0.0,
              3.7321
            ],
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-1-2.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          -60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-1-3.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          60.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8549,
            "band": 1.0,
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-2.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          -60.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0167,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 1.0,
            "height": [
              0.0,
              0.5
            ],
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
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-3.effect"
        },
        "scale": 2.0,
        "spin": [
          0.0,
          0.0,
          -60.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0167,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              3.3727
            ],
            [
              0.1719,
              5.6063
            ],
            [
              0.2656,
              7.2544
            ],
            [
              0.375,
              8.5559
            ],
            [
              0.4688,
              9.2407
            ],
            [
              0.5781,
              9.6621
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0478,
          0.0104,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.05,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 0.5625,
            "height": [
              0.0,
              4.6651
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1166,
                0.0478,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-5.effect"
        },
        "scale": 2.0,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0167,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 0.5625,
            "height": [
              0.0,
              4.6651
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1166,
                0.0478,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-5.effect"
        },
        "scale": 2.0,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0183,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 0.5625,
            "height": [
              0.0,
              4.6651
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1166,
                0.0478,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-5.effect"
        },
        "scale": 2.0,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.02,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9665,
            "band": 0.5625,
            "height": [
              0.0,
              4.6651
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.1166,
                0.0478,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-5.effect"
        },
        "scale": 2.0,
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0217,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0469,
              2.4677
            ],
            [
              0.0938,
              2.7515
            ],
            [
              0.1562,
              2.9743
            ],
            [
              0.2344,
              3.1121
            ],
            [
              0.3906,
              3.1815
            ],
            [
              1.0,
              3.1623
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
      "offset": [
        0.0,
        0.0,
        1.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        8.8889
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n0-7.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.4495
            ],
            [
              0.0469,
              3.0223
            ],
            [
              0.0938,
              3.3699
            ],
            [
              0.1562,
              3.6428
            ],
            [
              0.2344,
              3.8115
            ],
            [
              0.3906,
              3.8965
            ],
            [
              1.0,
              3.873
            ]
          ]
        },
        "color": [
          1.0,
          0.0,
          0.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        8.8889
      ],
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              1.0,
              1.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.7529
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "life": 0.45
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light2/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              5.0
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0478,
          0.0104,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.1,
      "life": 0.45
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.0478,
          0.0104
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
  "duration": 0.55,
  "events": [
    {
      "name": "impact",
      "time": 0.05
    }
  ]
});
