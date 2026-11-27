import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0 "グロー": a mesh's colour over its life: its brightest kept, alpha over life kept
// effekseer: node 1.2 "地面リンググロー": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 1.3 "地面リング": a ring whose height changes over its life: its middle height kept
// effekseer: node 1.3.0 "Glow": a mesh's colour over its life: its brightest kept, alpha over life kept
// effekseer: node 1.4.0.0 "星": motion out from a circle: approximated along one direction
// effekseer: node 1.4.0.0 "星": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.4.0.0.0 "星グロー": motion out from a circle: approximated along one direction
// effekseer: node 1.4.0.0.0 "星グロー": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.4.0.1 "星反転": motion out from a circle: approximated along one direction
// effekseer: node 1.4.0.1 "星反転": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.4.0.1.0 "星グロー": motion out from a circle: approximated along one direction
// effekseer: node 1.4.0.1.0 "星グロー": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.4.0.2 "粒子": motion out from a circle: approximated along one direction
// effekseer: node 1.4.0.2 "粒子": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.4.0.2.0 "粒子グロー": motion out from a circle: approximated along one direction
// effekseer: node 1.4.0.2.0 "粒子グロー": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        0.0,
        3.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        0.0,
        3.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        0.0,
        3.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.8333
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
                0.8
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-1.effect"
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
              1.2636
            ],
            [
              0.1719,
              1.5118
            ],
            [
              0.2656,
              1.6949
            ],
            [
              0.375,
              1.8395
            ],
            [
              0.5781,
              1.9625
            ],
            [
              1.0,
              2.0
            ]
          ]
        },
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4167,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8845,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                0.586,
                0.0
              ],
              [
                1.0,
                1.0,
                0.4151,
                0.8
              ],
              [
                1.0,
                1.0,
                0.586,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-2.effect"
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
              1.2636
            ],
            [
              0.1719,
              1.5118
            ],
            [
              0.2656,
              1.6949
            ],
            [
              0.375,
              1.8395
            ],
            [
              0.5781,
              1.9625
            ],
            [
              1.0,
              2.0
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
      "at": 0.4167,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.0,
            "height": [
              1.8236,
              0.0
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
                0.9345,
                0.1406
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.2031,
              1.8159
            ],
            [
              0.4219,
              1.6809
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
      "at": 0.4167,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.0938,
              0.6514
            ],
            [
              0.2031,
              1.289
            ],
            [
              0.3125,
              1.8108
            ],
            [
              0.4219,
              2.2336
            ],
            [
              0.5312,
              2.5741
            ],
            [
              0.6562,
              2.884
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          0.586,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4167,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-2.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-0-2-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-1-0.emitter"
        }
      },
      "at": 0.4167,
      "offset": [
        0.0,
        0.0,
        0.5
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-1-0-0.emitter"
        }
      },
      "at": 0.4167,
      "offset": [
        0.0,
        0.0,
        0.5
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-1-1.emitter"
        }
      },
      "at": 0.4167,
      "offset": [
        0.0,
        0.0,
        0.5
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-1-1-0.emitter"
        }
      },
      "at": 0.4167,
      "offset": [
        0.0,
        0.0,
        0.5
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-1-2.emitter"
        }
      },
      "at": 0.4167,
      "offset": [
        0.0,
        0.0,
        0.5
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure7/n1-4-1-2-0.emitter"
        }
      },
      "at": 0.4167,
      "offset": [
        0.0,
        0.0,
        0.5
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 2.6,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
