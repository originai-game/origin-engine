import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.1 "Wind_RingPolygon": ring 15 instances: the first 4 drawn
// effekseer: node 0.0.0.1 "Wind_RingPolygon": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.3.2 "Shockwave": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.3.2 "Shockwave": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-0.emitter"
        }
      },
      "at": 0.1
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              0.0,
              0.5536
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
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
      "at": 0.1,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 1.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              0.0,
              0.5536
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
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
      "at": 0.1167,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 1.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              0.0,
              0.5536
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
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
      "at": 0.1333,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 1.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              0.0,
              0.5536
            ],
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2344,
              3.002
            ],
            [
              0.4688,
              5.1558
            ],
            [
              0.7188,
              8.0751
            ],
            [
              1.0,
              12.0
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
      "at": 0.15,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 1.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-2.emitter"
        }
      },
      "at": 0.1,
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
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-0-3.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        6.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-1.emitter"
        }
      },
      "at": 0.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-2.emitter"
        }
      },
      "at": 0.1
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              0.5,
              22.525
            ],
            [
              1.0,
              20.55
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.2133
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        0.0,
        0.0,
        -7.0
      ],
      "rotate": [
        -90.0,
        -89.972,
        0.0
      ],
      "acceleration": [
        0.0,
        0.0,
        3.6
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              0.5,
              22.525
            ],
            [
              1.0,
              20.55
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.2133
        ],
        "blend": "additive"
      },
      "at": 0.2333,
      "offset": [
        0.0,
        0.0,
        -7.0
      ],
      "rotate": [
        -90.0,
        89.972,
        0.0
      ],
      "acceleration": [
        0.0,
        0.0,
        3.6
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              0.5,
              22.525
            ],
            [
              1.0,
              20.55
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.2133
        ],
        "blend": "additive"
      },
      "at": 0.3,
      "offset": [
        0.0,
        0.0,
        -7.0
      ],
      "rotate": [
        -90.0,
        -89.972,
        0.0
      ],
      "acceleration": [
        0.0,
        0.0,
        3.6
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              0.5,
              22.525
            ],
            [
              1.0,
              20.55
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.2133
        ],
        "blend": "additive"
      },
      "at": 0.3667,
      "offset": [
        0.0,
        0.0,
        -7.0
      ],
      "rotate": [
        -90.0,
        89.972,
        0.0
      ],
      "acceleration": [
        0.0,
        0.0,
        3.6
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              1.0,
              24.16
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.138
        ],
        "blend": "additive"
      },
      "at": 0.2,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        -90.0,
        -60.0,
        0.0
      ],
      "life": 0.2667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              1.0,
              24.16
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.138
        ],
        "blend": "additive"
      },
      "at": 0.2333,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        90.0,
        0.0,
        180.0
      ],
      "life": 0.2667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              24.0
            ],
            [
              1.0,
              24.16
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.138
        ],
        "blend": "additive"
      },
      "at": 0.2667,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        -90.0,
        60.0,
        0.0
      ],
      "life": 0.2667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2665,
            "band": 0.4,
            "height": [
              -0.4723,
              -0.0349
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.1255
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
                0.1255
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.1094,
              3.1491
            ],
            [
              0.3906,
              9.2929
            ],
            [
              0.5469,
              12.3356
            ],
            [
              0.7344,
              15.398
            ],
            [
              1.0,
              19.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.9493
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -180.0
      ],
      "life": 0.3667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8,
            "band": 0.5,
            "height": [
              0.0,
              2.0
            ],
            "colors": [
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
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              1.0,
              11.6
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
      "at": 0.1333,
      "offset": [
        0.0,
        0.0,
        -7.0
      ],
      "velocity": [
        0.0,
        0.0,
        -24.0
      ],
      "acceleration": [
        0.0,
        0.0,
        -18.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8,
            "band": 0.5,
            "height": [
              0.0,
              2.0
            ],
            "colors": [
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
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/SonicBoom/n0-0-3-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              1.0,
              11.6
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
      "at": 0.3833,
      "offset": [
        0.0,
        0.0,
        -7.0
      ],
      "velocity": [
        0.0,
        0.0,
        -24.0
      ],
      "acceleration": [
        0.0,
        0.0,
        -18.0
      ],
      "life": 1.0
    }
  ],
  "duration": 1.8333,
  "events": [
    {
      "name": "impact",
      "time": 0.1
    }
  ]
});
