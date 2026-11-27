import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "振り下ろしglow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1 "glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2 "振り下ろし": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0 "衝撃リングギザギザ": ring 5 instances: the first 4 drawn
// effekseer: node 0.2.0 "衝撃リングギザギザ": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.2.1 "衝撃リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.2.1 "衝撃リング": a UV region of a texture whose size is not known (not a PNG)
// effekseer: node 0.3.0.0 "火の粉": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.1 "火の粉余韻": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-0-0-0.effect"
        },
        "width": [
          0.2222,
          0.0556
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          0.0104,
          0.2195,
          0.502
        ],
        "orbit": [
          0.6667,
          1.2556
        ]
      },
      "at": 0.0,
      "offset": [
        2.3473,
        2.3182,
        5.0
      ],
      "rotate": [
        -90.0,
        -67.0,
        -180.0
      ],
      "velocity": [
        0.0,
        -12.0,
        -20.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        2.6078,
        2.9319,
        5.0
      ],
      "rotate": [
        0.0,
        0.0,
        -113.0
      ],
      "velocity": [
        0.0,
        -12.0,
        -20.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-0-0-2.effect"
        },
        "width": [
          0.2222,
          0.0556
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.7529
        ],
        "orbit": [
          0.6667,
          1.2556
        ]
      },
      "at": 0.0,
      "offset": [
        2.3473,
        2.3182,
        5.0
      ],
      "rotate": [
        -90.0,
        -67.0,
        -180.0
      ],
      "velocity": [
        0.0,
        -12.0,
        -20.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-1-0.emitter"
        }
      },
      "at": 0.2333,
      "offset": [
        -8.0,
        -1.5,
        0.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-1-0-0.emitter"
        }
      },
      "at": 0.2333,
      "offset": [
        -8.0,
        -1.5,
        0.0
      ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-1-0-1.emitter"
        }
      },
      "at": 0.2333,
      "offset": [
        -8.0,
        -1.5,
        0.0
      ],
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
            "inner": 0.1745,
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
                0.2195,
                0.5356,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-2-0.effect"
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
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
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
      "at": 0.2167,
      "offset": [
        -8.0,
        -1.5,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6698,
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
                0.2195,
                0.5356,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-2-1.effect"
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
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
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
      "at": 0.2167,
      "offset": [
        -8.0,
        -1.5,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-2-2.effect"
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
              5.2182
            ],
            [
              0.1719,
              9.189
            ],
            [
              0.2656,
              12.1189
            ],
            [
              0.375,
              14.4327
            ],
            [
              0.4688,
              15.6502
            ],
            [
              0.5781,
              16.3993
            ],
            [
              1.0,
              17.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0104,
          0.2195,
          0.3922
        ],
        "blend": "additive"
      },
      "at": 0.2167,
      "offset": [
        -8.0,
        -1.5,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-3-0-0.emitter"
        }
      },
      "at": 0.2333,
      "offset": [
        -8.0,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword17/n0-3-0-1.emitter"
        }
      },
      "at": 0.2333,
      "offset": [
        -8.0,
        -1.0,
        0.0
      ]
    }
  ],
  "duration": 1.0683,
  "events": [
    {
      "name": "impact",
      "time": 0.2333
    }
  ]
});
