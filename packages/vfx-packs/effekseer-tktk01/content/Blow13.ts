import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1.0 "割れ": ring 80 instances: the first 4 drawn
// effekseer: node 0.1.0.0 "割れ着色": ring 80 instances: the first 4 drawn
// effekseer: node 0.2.0 "サブ割れ": ring 100 instances: the first 4 drawn
// effekseer: node 0.3.2 "リング着色": ring 5 instances: the first 4 drawn
// effekseer: node 0.3.3 "リング外枠インパクト": ring 5 instances: the first 4 drawn
// effekseer: node 0.4.0.0 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.1 "軌跡カラー": a moving ancestor: its descendants are placed where it starts
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-0-0.emitter"
        }
      },
      "at": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-0-0-1.emitter"
        }
      },
      "at": 0.0,
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
            "inner": 0.0,
            "band": 0.0,
            "start": 262.5,
            "length": 15.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-1-0.effect"
        },
        "scale": 1.25,
        "spin": [
          0.0,
          0.0,
          648.0
        ],
        "color": [
          80.0,
          80.0,
          80.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "velocity": [
        12.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "start": 262.5,
            "length": 15.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.0,
                0.6354,
                0.6354,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-1-0-0.effect"
        },
        "scale": 1.25,
        "color": [
          80.0,
          80.0,
          80.0,
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
            "band": 0.0,
            "start": 266.0,
            "length": 8.0,
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
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-2-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "velocity": [
        9.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-3-0.effect"
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
              2.7876
            ],
            [
              0.1719,
              3.5209
            ],
            [
              0.2656,
              4.054
            ],
            [
              0.375,
              4.4663
            ],
            [
              0.5625,
              4.7916
            ],
            [
              0.875,
              4.8903
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
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9623,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-3-1.effect"
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
              2.8138
            ],
            [
              0.1719,
              3.5716
            ],
            [
              0.2656,
              4.1224
            ],
            [
              0.375,
              4.5485
            ],
            [
              0.5625,
              4.8846
            ],
            [
              0.875,
              4.9866
            ],
            [
              1.0,
              5.1
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
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
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
                0.0478,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-3-2.effect"
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
              2.7876
            ],
            [
              0.1719,
              3.5209
            ],
            [
              0.2656,
              4.054
            ],
            [
              0.375,
              4.4663
            ],
            [
              0.5625,
              4.7916
            ],
            [
              0.875,
              4.8903
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
      "at": 0.0,
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.3355,
            "band": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-3-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              5.0
            ],
            [
              0.0781,
              7.6252
            ],
            [
              0.1719,
              10.0696
            ],
            [
              0.2656,
              11.8465
            ],
            [
              0.375,
              13.2211
            ],
            [
              0.5625,
              14.3053
            ],
            [
              0.875,
              14.6343
            ],
            [
              1.0,
              15.0
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
        -73.8979,
        -25.6589,
        -56.3099
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.4952,
            "band": 0.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-3-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              5.0
            ],
            [
              0.0781,
              6.3126
            ],
            [
              0.1719,
              7.5348
            ],
            [
              0.2656,
              8.4233
            ],
            [
              0.375,
              9.1105
            ],
            [
              0.5625,
              9.6526
            ],
            [
              1.0,
              10.0
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
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-3-5.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-4-0-0.effect"
        },
        "width": [
          7.5,
          1.875
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.866,
        -1.7321,
        -0.5
      ],
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-4-0-1.effect"
        },
        "width": [
          15.0,
          3.75
        ],
        "lifetime": 0.3333,
        "color": [
          0.047,
          0.047,
          0.047,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.866,
        -1.7321,
        -0.5
      ],
      "rotate": [
        -73.8979,
        -25.6589,
        -56.3099
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-4-1-0-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        49.1066,
        -48.5904,
        40.8934
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-4-1-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        49.1066,
        -48.5904,
        40.8934
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-4-1-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        49.1066,
        -48.5904,
        40.8934
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-5-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.9,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow13/n0-5-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.9,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9806,
          0.9806
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
        0.0,
        0.0108,
        0.0
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
  "duration": 0.8317,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
