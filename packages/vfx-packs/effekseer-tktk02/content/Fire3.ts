import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0 "FireRing": ring 20 instances: the first 4 drawn
// effekseer: node 1.0 "FireRing": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0 "FireRing": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.0 "FireRing": ring 20 instances: the first 4 drawn
// effekseer: node 2.0 "FireRing": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 3.1.0 "ｌｅｎｓ": a ring's arc over its life
// effekseer: node 3.1.0 "ｌｅｎｓ": ring 100 instances: the first 4 drawn
// effekseer: node 4 "Ring": ring 10 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n0-0-0-0-1.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n0-0-0-0-2.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.419,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                0.1882
              ],
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
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
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.419,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                0.1882
              ],
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0042,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.419,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                0.1882
              ],
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0083,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.419,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ],
              [
                1.0,
                0.6354,
                0.4396,
                0.1882
              ],
              [
                1.0,
                0.0619,
                0.0135,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0125,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n1-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n1-2.emitter"
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
            "inner": 0.7333,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ],
              [
                1.0,
                0.5732,
                0.3567,
                0.1882
              ],
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7333,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ],
              [
                1.0,
                0.5732,
                0.3567,
                0.1882
              ],
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4042,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7333,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ],
              [
                1.0,
                0.5732,
                0.3567,
                0.1882
              ],
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4083,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7333,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ],
              [
                1.0,
                0.5732,
                0.3567,
                0.1882
              ],
              [
                1.0,
                0.0532,
                0.0116,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4125,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n2-1.emitter"
        }
      },
      "at": 0.4
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n3-0.emitter"
        }
      },
      "at": 0.4667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "start": 267.5,
            "length": 5.0,
            "colors": [
              [
                1.0,
                0.0781,
                0.017,
                0.0
              ],
              [
                1.0,
                0.2618,
                0.0781,
                1.0
              ],
              [
                1.0,
                0.0781,
                0.017,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n3-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.1094,
              4.2988
            ],
            [
              0.2188,
              5.3936
            ],
            [
              0.3281,
              6.3036
            ],
            [
              0.4531,
              7.1425
            ],
            [
              0.7031,
              8.289
            ],
            [
              1.0,
              9.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          100.0,
          100.0,
          100.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9158,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ],
              [
                1.0,
                0.4198,
                0.1636,
                0.3137
              ],
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n4.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.125,
              3.3815
            ],
            [
              0.2188,
              4.5627
            ],
            [
              0.3125,
              5.4839
            ],
            [
              0.4062,
              6.1389
            ],
            [
              0.5,
              6.5534
            ],
            [
              0.5938,
              6.774
            ],
            [
              1.0,
              7.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.485,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9158,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ],
              [
                1.0,
                0.4198,
                0.1636,
                0.3137
              ],
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n4.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.125,
              3.3815
            ],
            [
              0.2188,
              4.5627
            ],
            [
              0.3125,
              5.4839
            ],
            [
              0.4062,
              6.1389
            ],
            [
              0.5,
              6.5534
            ],
            [
              0.5938,
              6.774
            ],
            [
              1.0,
              7.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4867,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9158,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ],
              [
                1.0,
                0.4198,
                0.1636,
                0.3137
              ],
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n4.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.125,
              3.3815
            ],
            [
              0.2188,
              4.5627
            ],
            [
              0.3125,
              5.4839
            ],
            [
              0.4062,
              6.1389
            ],
            [
              0.5,
              6.5534
            ],
            [
              0.5938,
              6.774
            ],
            [
              1.0,
              7.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4883,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9158,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ],
              [
                1.0,
                0.4198,
                0.1636,
                0.3137
              ],
              [
                1.0,
                0.0415,
                0.009,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire3/n4.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.125,
              3.3815
            ],
            [
              0.2188,
              4.5627
            ],
            [
              0.3125,
              5.4839
            ],
            [
              0.4062,
              6.1389
            ],
            [
              0.5,
              6.5534
            ],
            [
              0.5938,
              6.774
            ],
            [
              1.0,
              7.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.49,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.6667
    }
  ],
  "duration": 1.3,
  "events": [
    {
      "name": "impact",
      "time": 0.4667
    }
  ]
});
