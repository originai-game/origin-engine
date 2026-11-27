import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "刃上": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.0.0.0 "岩軌跡": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.1.0 "ライン": nested generation: 600 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1 "グロー画像": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1 "グロー画像": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.0.2 "刃": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.2.0 "着弾時衝撃リング": ring 3 instances: the first 4 drawn
// effekseer: node 0.0.0.2.0 "着弾時衝撃リング": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0 "着弾時衝撃リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.2.1 "着弾時衝撃リング柱": ring 3 instances: the first 4 drawn
// effekseer: node 0.0.0.2.1 "着弾時衝撃リング柱": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.1 "着弾時衝撃リング柱": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.0.0.2.1 "着弾時衝撃リング柱": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.2.2.0 "着弾時煙": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.2.0 "着弾時煙": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.0.2.3.0 "消滅時線": nested generation: 10 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "height": [
              1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
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
            "inner": 0.0,
            "band": 0.0,
            "height": [
              1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0333,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "height": [
              1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0667,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "height": [
              1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.1,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-0-0-0.effect"
        },
        "width": [
          1.0,
          1.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          0.5882
        ]
      },
      "at": 0.0,
      "offset": [
        4.6488,
        0.0,
        9.9694
      ],
      "rotate": [
        0.0,
        25.0,
        0.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        5.0714,
        0.0,
        10.8757
      ],
      "rotate": [
        90.0,
        0.0,
        -25.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-1.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        3.8388,
        0.0,
        8.2323
      ],
      "rotate": [
        90.0,
        0.0,
        -25.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ],
      "until": 0.5
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
              1.0,
              -1.0
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
                0.2153
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
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
            "inner": 0.0,
            "band": 0.5,
            "height": [
              1.0,
              -1.0
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
                0.2153
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0333,
      "life": 0.6667
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
              1.0,
              -1.0
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
                0.2153
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0667,
      "life": 0.6667
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
              1.0,
              -1.0
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
                0.2153
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.1,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5514,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.3112,
                0.0781,
                0.0781,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2-0.effect"
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
              1.5273
            ],
            [
              0.1719,
              2.0236
            ],
            [
              0.2656,
              2.3899
            ],
            [
              0.375,
              2.6791
            ],
            [
              0.4688,
              2.8313
            ],
            [
              0.5781,
              2.9249
            ],
            [
              1.0,
              3.0
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
      "at": 0.4667,
      "offset": [
        0.2817,
        0.0,
        0.6042
      ],
      "rotate": [
        0.0,
        25.0,
        0.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6057,
            "band": 0.0,
            "height": [
              0.2113,
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
                0.3112,
                0.0781,
                0.0781,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2-1.effect"
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
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
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
      "at": 0.4667,
      "offset": [
        0.2817,
        0.0,
        0.6042
      ],
      "rotate": [
        0.0,
        25.0,
        0.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2-2-0.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        0.2817,
        0.0,
        0.6042
      ],
      "rotate": [
        0.0,
        25.0,
        0.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Soil1/n0-0-0-2-3-0.emitter"
        }
      },
      "at": 0.5833,
      "offset": [
        -0.2817,
        0.0,
        -0.6042
      ],
      "rotate": [
        0.0,
        25.0,
        -90.0
      ],
      "velocity": [
        -8.4524,
        0.0,
        -18.1262
      ]
    }
  ],
  "duration": 1.4,
  "looping": true
});
