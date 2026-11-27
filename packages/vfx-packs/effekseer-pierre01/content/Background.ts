import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "Expander": ring unlimited instances: the first 4 drawn
// effekseer: node 0.0.0 "Expander": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.0 "Expander": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.1 "Top": ring unlimited instances: the first 4 drawn
// effekseer: node 0.0.1 "Top": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.2 "Expander2": ring unlimited instances: the first 4 drawn
// effekseer: node 0.0.2 "Expander2": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.2 "Expander2": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.3 "Line": ring 8 instances: the first 4 drawn
// effekseer: node 0.0.3 "Line": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.3 "Line": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6923,
            "band": 0.5,
            "height": [
              -2.0,
              -1.3846
            ],
            "colors": [
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ],
              [
                0.2845,
                0.2195,
                1.0,
                0.251
              ],
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              44.0
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
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 1.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6923,
            "band": 0.5,
            "height": [
              -2.0,
              -1.3846
            ],
            "colors": [
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ],
              [
                0.2845,
                0.2195,
                1.0,
                0.251
              ],
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              44.0
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
      "at": 0.1833,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 1.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6923,
            "band": 0.5,
            "height": [
              -2.0,
              -1.3846
            ],
            "colors": [
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ],
              [
                0.2845,
                0.2195,
                1.0,
                0.251
              ],
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              44.0
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
      "at": 0.35,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 1.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6923,
            "band": 0.5,
            "height": [
              -2.0,
              -1.3846
            ],
            "colors": [
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ],
              [
                0.2845,
                0.2195,
                1.0,
                0.251
              ],
              [
                0.2195,
                0.1166,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              44.0
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
      "at": 0.5167,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 1.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              -2.0,
              0.0
            ],
            "colors": [
              [
                0.2195,
                0.3587,
                0.5356,
                0.0
              ],
              [
                0.3587,
                0.5356,
                0.7519,
                1.0
              ],
              [
                0.2195,
                0.3587,
                0.5356,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-1.effect"
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
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              -2.0,
              0.0
            ],
            "colors": [
              [
                0.2195,
                0.3587,
                0.5356,
                0.0
              ],
              [
                0.3587,
                0.5356,
                0.7519,
                1.0
              ],
              [
                0.2195,
                0.3587,
                0.5356,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-1.effect"
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
      "at": 0.0833,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              -2.0,
              0.0
            ],
            "colors": [
              [
                0.2195,
                0.3587,
                0.5356,
                0.0
              ],
              [
                0.3587,
                0.5356,
                0.7519,
                1.0
              ],
              [
                0.2195,
                0.3587,
                0.5356,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-1.effect"
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
      "at": 0.1667,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "height": [
              -2.0,
              0.0
            ],
            "colors": [
              [
                0.2195,
                0.3587,
                0.5356,
                0.0
              ],
              [
                0.3587,
                0.5356,
                0.7519,
                1.0
              ],
              [
                0.2195,
                0.3587,
                0.5356,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-1.effect"
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
      "at": 0.25,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 0.5
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
              -2.0,
              -1.0
            ],
            "colors": [
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                1.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              16.0
            ],
            [
              1.0,
              48.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.5312
        ],
        "blend": "additive"
      },
      "at": 0.0167,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
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
            "height": [
              -2.0,
              -1.0
            ],
            "colors": [
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                1.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              16.0
            ],
            [
              1.0,
              48.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.5312
        ],
        "blend": "additive"
      },
      "at": 0.1833,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
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
            "height": [
              -2.0,
              -1.0
            ],
            "colors": [
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                1.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              16.0
            ],
            [
              1.0,
              48.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.5312
        ],
        "blend": "additive"
      },
      "at": 0.35,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
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
            "height": [
              -2.0,
              -1.0
            ],
            "colors": [
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                1.0
              ],
              [
                0.0254,
                0.1166,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              16.0
            ],
            [
              1.0,
              48.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.5312
        ],
        "blend": "additive"
      },
      "at": 0.5167,
      "offset": [
        0.0,
        0.0,
        10.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6894,
            "band": 0.3,
            "height": [
              -1.5,
              -0.8078
            ],
            "colors": [
              [
                0.3587,
                0.6661,
                1.0,
                1.0
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.7529
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-3.effect"
        },
        "scale": 36.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 8.0,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 7.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6894,
            "band": 0.3,
            "height": [
              -1.5,
              -0.8078
            ],
            "colors": [
              [
                0.3587,
                0.6661,
                1.0,
                1.0
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.7529
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-3.effect"
        },
        "scale": 36.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 8.5,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 7.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6894,
            "band": 0.3,
            "height": [
              -1.5,
              -0.8078
            ],
            "colors": [
              [
                0.3587,
                0.6661,
                1.0,
                1.0
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.7529
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-3.effect"
        },
        "scale": 36.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 9.0,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 7.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6894,
            "band": 0.3,
            "height": [
              -1.5,
              -0.8078
            ],
            "colors": [
              [
                0.3587,
                0.6661,
                1.0,
                1.0
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.7529
              ],
              [
                0.3587,
                0.6661,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-0-3.effect"
        },
        "scale": 36.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 9.5,
      "offset": [
        0.0,
        0.0,
        -15.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 7.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-1-0.emitter"
        }
      },
      "at": 5.0,
      "offset": [
        0.0,
        0.0,
        5.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/Background/n0-1-1.emitter"
        }
      },
      "at": 10.0,
      "offset": [
        0.0,
        0.0,
        5.0
      ]
    }
  ],
  "duration": 19.3333,
  "events": [
    {
      "name": "impact",
      "time": 10.0
    }
  ]
});
