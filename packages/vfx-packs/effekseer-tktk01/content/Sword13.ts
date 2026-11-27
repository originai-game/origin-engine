import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "ナイフ軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "*衝撃生業用": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0 "ナイフ軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.1 "*衝撃生業用": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0 "ナイフ軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.1 "*衝撃生業用": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.1 "衝撃線": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 4 "衝撃リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n0-0.effect"
        },
        "scale": 2.0,
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
        4.6985,
        0.2899,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        70.0
      ],
      "velocity": [
        -28.1908,
        10.2606,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n0-0-0.effect"
        },
        "width": [
          2.0,
          0.5
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.6667
        ]
      },
      "at": 0.0,
      "offset": [
        4.6985,
        0.2899,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        70.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        5.6382,
        -0.0521,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        70.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n1-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        -4.5,
        3.3301,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -150.0
      ],
      "velocity": [
        15.0,
        -25.9808,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n1-0-0.effect"
        },
        "width": [
          2.0,
          0.5
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.6667
        ]
      },
      "at": 0.1667,
      "offset": [
        -4.5,
        3.3301,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -150.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n1-0-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -5.0,
        4.1962,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -150.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n2-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3333,
      "offset": [
        0.6959,
        -4.9513,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        8.0
      ],
      "velocity": [
        -4.1752,
        29.708,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n2-0-0.effect"
        },
        "width": [
          2.0,
          0.5
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.2619
        ]
      },
      "at": 0.3333,
      "offset": [
        0.6959,
        -4.9513,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        8.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n2-0-1.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        0.835,
        -5.9416,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        8.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n3-0.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n3-1.parent"
        }
      },
      "at": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6251,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n4.effect"
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
              2.7909
            ],
            [
              0.1719,
              3.5354
            ],
            [
              0.2656,
              4.0848
            ],
            [
              0.375,
              4.5186
            ],
            [
              0.4688,
              4.7469
            ],
            [
              0.5781,
              4.8874
            ],
            [
              1.0,
              5.0
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword13/n5.emitter"
        }
      },
      "at": 0.4167
    }
  ],
  "duration": 2.5917,
  "events": [
    {
      "name": "impact",
      "time": 0.4167
    }
  ]
});
