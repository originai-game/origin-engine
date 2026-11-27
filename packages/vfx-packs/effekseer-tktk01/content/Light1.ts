import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "光の柱右側": ring 3 instances: the first 4 drawn
// effekseer: node 0.0.0.0.0 "光の柱右側": nested generation: 25 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0 "光の柱左側": ring 3 instances: the first 4 drawn
// effekseer: node 0.0.0.1.0 "光の柱左側": nested generation: 25 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2 "光の柱中心ベース": ring 3 instances: the first 4 drawn
// effekseer: node 0.0.0.2 "光の柱中心ベース": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2 "光の柱中心ベース": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.0.2.0 "光の柱中心ベース": ring 3 instances: the first 4 drawn
// effekseer: node 0.0.0.2.0 "光の柱中心ベース": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0 "光の柱中心ベース": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.0.3 "光の足元": ring 5 instances: the first 4 drawn
// effekseer: node 0.0.0.3 "光の足元": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.3 "光の足元": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.1.0.0.0 "光の柱右側": ring 3 instances: the first 4 drawn
// effekseer: node 0.1.0.0.0 "光の柱右側": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.1.0.1.0 "光の柱左側": ring 3 instances: the first 4 drawn
// effekseer: node 0.1.0.1.0 "光の柱左側": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.1.0.2 "光の柱中心ベース": ring 3 instances: the first 4 drawn
// effekseer: node 0.1.0.2 "光の柱中心ベース": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 0.1.0.2 "光の柱中心ベース": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.1.0.2.0 "光の柱中心ベース": ring 3 instances: the first 4 drawn
// effekseer: node 0.1.0.2.0 "光の柱中心ベース": nested generation: 18 instances above its generator folded into one emitter
// effekseer: node 0.1.0.2.0 "光の柱中心ベース": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.1.0.3 "光の足元": ring 5 instances: the first 4 drawn
// effekseer: node 0.1.0.3 "光の足元": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 0.1.0.3 "光の足元": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              10.0,
              -10.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-0-0.effect"
        },
        "scale": 0.825,
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        8.5833,
        0.0,
        9.0
      ],
      "velocity": [
        3.5,
        0.0,
        0.0
      ],
      "life": 0.3333
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
              10.0,
              -10.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-1-0.effect"
        },
        "scale": 0.825,
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "offset": [
        7.4167,
        0.0,
        9.0
      ],
      "velocity": [
        -3.5,
        0.0,
        0.0
      ],
      "life": 0.3333
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.4667
            ],
            [
              1.0,
              0.9333
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
      "offset": [
        8.0,
        0.0,
        9.0
      ],
      "life": 1.0
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.4667
            ],
            [
              1.0,
              0.9333
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
      "at": 0.25,
      "offset": [
        8.0,
        0.0,
        9.0
      ],
      "life": 1.0
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.4667
            ],
            [
              1.0,
              0.9333
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
      "offset": [
        8.0,
        0.0,
        9.0
      ],
      "life": 1.0
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2353
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.5882
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2353
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0889
            ],
            [
              1.0,
              2.1778
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
      "offset": [
        8.0,
        0.0,
        9.0
      ],
      "life": 0.5
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2353
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.5882
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2353
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0889
            ],
            [
              1.0,
              2.1778
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
      "at": 0.25,
      "offset": [
        8.0,
        0.0,
        9.0
      ],
      "life": 0.5
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2353
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.5882
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2353
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0889
            ],
            [
              1.0,
              2.1778
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
      "offset": [
        8.0,
        0.0,
        9.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 16.0,
            "band": 1.0,
            "colors": [
              [
                0.6456,
                0.9828,
                0.9914,
                0.0
              ],
              [
                0.5668,
                0.9574,
                0.9914,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-0-0-3.effect"
        },
        "scale": 0.0,
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        8.0,
        0.0,
        -14.3333
      ],
      "life": 1.1667
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
              10.0,
              -10.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-0-0.effect"
        },
        "scale": 0.9625,
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        15.8472,
        0.0,
        9.0
      ],
      "velocity": [
        4.0833,
        0.0,
        0.0
      ],
      "life": 0.3333
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
              10.0,
              -10.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-1-0.effect"
        },
        "scale": 0.9625,
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "offset": [
        14.4861,
        0.0,
        9.0
      ],
      "velocity": [
        -4.0833,
        0.0,
        0.0
      ],
      "life": 0.3333
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5444
            ],
            [
              1.0,
              1.0889
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
      "offset": [
        15.1667,
        0.0,
        9.0
      ],
      "life": 1.0
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5444
            ],
            [
              1.0,
              1.0889
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
      "at": 0.25,
      "offset": [
        15.1667,
        0.0,
        9.0
      ],
      "life": 1.0
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2745
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.5444
            ],
            [
              1.0,
              1.0889
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
      "offset": [
        15.1667,
        0.0,
        9.0
      ],
      "life": 1.0
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
              6.6667,
              -6.6667
            ],
            "colors": [
              [
                0.3489,
                0.7519,
                0.9743,
                0.2353
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.5882
              ],
              [
                0.3112,
                0.7519,
                0.9743,
                0.2353
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.2704
            ],
            [
              1.0,
              2.5407
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
      "at": 0.0,
      "offset": [
        15.1667,
        0.0,
        9.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 16.0,
            "band": 1.0,
            "colors": [
              [
                0.6456,
                0.9828,
                0.9914,
                0.0
              ],
              [
                0.5668,
                0.9574,
                0.9914,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-1-0-3.effect"
        },
        "scale": 0.0,
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        15.1667,
        0.0,
        -18.2222
      ],
      "life": 1.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Light1/n0-2.effect"
        },
        "scale": 35.0,
        "spin": [
          0.0,
          0.0,
          -240.0
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
      "offset": [
        0.0,
        0.0,
        -22.0
      ],
      "life": 2.6667
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          1.0,
          1.0
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
      "life": 0.3,
      "offset": [
        0.3329,
        0.0,
        -21.2352
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 2.6667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
