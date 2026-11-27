import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "斬撃基本": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0 "斬撃基本２": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2 "衝撃リング１": ring 5 instances: the first 4 drawn
// effekseer: node 2 "衝撃リング１": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3 "衝撃リング２": ring 5 instances: the first 4 drawn
// effekseer: node 3 "衝撃リング２": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 4.0 "斬撃〆": a moving ancestor: its descendants are placed where it starts
// effekseer: node 4.0 "斬撃〆": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 5 "〆リング": ring 10 instances: the first 4 drawn
// effekseer: node 5 "〆リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 6 "〆リング２": ring 10 instances: the first 4 drawn
// effekseer: node 6 "〆リング２": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 8 "〆リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 9 "〆リング２": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        9.3969,
        3.4202,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        110.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n1-0.emitter"
        }
      },
      "at": 0.25,
      "offset": [
        -9.3969,
        3.4202,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -110.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5083,
            "band": 0.9,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.278,
                0.7047,
                0.278,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              2.7786
            ],
            [
              0.1562,
              3.3805
            ],
            [
              0.2344,
              3.8292
            ],
            [
              0.3281,
              4.1984
            ],
            [
              0.4688,
              4.4905
            ],
            [
              0.8281,
              4.6849
            ],
            [
              1.0,
              5.0
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
      "at": 0.1333,
      "rotate": [
        133.2192,
        -62.0091,
        -43.2192
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5083,
            "band": 0.9,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.278,
                0.7047,
                0.278,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              2.7786
            ],
            [
              0.1562,
              3.3805
            ],
            [
              0.2344,
              3.8292
            ],
            [
              0.3281,
              4.1984
            ],
            [
              0.4688,
              4.4905
            ],
            [
              0.8281,
              4.6849
            ],
            [
              1.0,
              5.0
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
      "at": 0.3833,
      "rotate": [
        -60.968,
        -61.0954,
        122.3756
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n4-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        0.0,
        15.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5083,
            "band": 0.9,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.047,
                0.4647,
                0.047,
                0.2941
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              2.7786
            ],
            [
              0.1562,
              3.3805
            ],
            [
              0.2344,
              3.8292
            ],
            [
              0.3281,
              4.1984
            ],
            [
              0.4688,
              4.4905
            ],
            [
              0.8281,
              4.6849
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.7833,
      "rotate": [
        -69.6394,
        -22.521,
        -45.9047
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5083,
            "band": 0.9,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.047,
                0.4647,
                0.047,
                0.2941
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n6.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.0781,
              2.7786
            ],
            [
              0.1562,
              3.3805
            ],
            [
              0.2344,
              3.8292
            ],
            [
              0.3281,
              4.1984
            ],
            [
              0.4688,
              4.4905
            ],
            [
              0.8281,
              4.6849
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.7833,
      "rotate": [
        110.3606,
        -22.521,
        134.0953
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n7-0.emitter"
        }
      },
      "at": 0.7833,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5083,
            "band": 0.9,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.1275,
                1.0,
                0.1275,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n8.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.0781,
              4.1678
            ],
            [
              0.1562,
              5.0707
            ],
            [
              0.2344,
              5.7437
            ],
            [
              0.3281,
              6.2976
            ],
            [
              0.4688,
              6.7358
            ],
            [
              0.8281,
              7.0273
            ],
            [
              1.0,
              7.5
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
      "at": 0.7833,
      "rotate": [
        -69.6394,
        -22.521,
        -45.9047
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5083,
            "band": 0.9,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.1275,
                1.0,
                0.1275,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword4/n9.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.0
            ],
            [
              0.0781,
              4.1678
            ],
            [
              0.1562,
              5.0707
            ],
            [
              0.2344,
              5.7437
            ],
            [
              0.3281,
              6.2976
            ],
            [
              0.4688,
              6.7358
            ],
            [
              0.8281,
              7.0273
            ],
            [
              1.0,
              7.5
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
      "at": 0.7833,
      "rotate": [
        110.3606,
        -22.521,
        134.0953
      ],
      "life": 0.4167
    }
  ],
  "duration": 1.3332,
  "events": [
    {
      "name": "impact",
      "time": 0.7833
    }
  ]
});
