import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.1.0 "蹴りの軌跡": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.1.1 "蹴りの軌跡カラフル": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 0.1.3 "リング外枠": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-0-0.effect"
        },
        "width": [
          1.7778,
          0.4444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.7529
        ],
        "orbit": [
          3.5556,
          -1.6
        ]
      },
      "at": 0.0,
      "offset": [
        -0.4444,
        0.7698,
        0.0
      ],
      "rotate": [
        -90.0,
        60.0,
        -180.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-0-1-0.effect"
        },
        "width": [
          0.0889,
          0.0222
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          3.5556,
          -1.6
        ]
      },
      "at": 0.0,
      "offset": [
        -0.4444,
        0.7698,
        0.0
      ],
      "rotate": [
        -90.0,
        60.0,
        -180.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-0-1-1.effect"
        },
        "width": [
          0.2222,
          0.0556
        ],
        "lifetime": 0.3333,
        "color": [
          0.047,
          0.047,
          0.047,
          0.502
        ],
        "orbit": [
          3.5556,
          -1.6
        ]
      },
      "at": 0.0,
      "offset": [
        -0.4444,
        0.7698,
        0.0
      ],
      "rotate": [
        -90.0,
        60.0,
        -180.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-0-2.effect"
        },
        "width": [
          1.7778,
          0.4444
        ],
        "lifetime": 0.1667,
        "color": [
          0.047,
          0.047,
          0.047,
          1.0
        ],
        "orbit": [
          3.5556,
          -1.6
        ]
      },
      "at": 0.0,
      "offset": [
        -0.4444,
        0.7698,
        0.0
      ],
      "rotate": [
        -90.0,
        60.0,
        -180.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        1.2,
        -2.0785,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-1-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        1.2,
        -2.0785,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-0-0-0-1-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        1.2,
        -2.0785,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -2.5,
        0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-0-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -2.5,
        0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-0-0-1.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -2.5,
        0.0,
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
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.0
            ],
            [
              0.0781,
              6.9
            ],
            [
              0.1719,
              9.6299
            ],
            [
              0.2656,
              11.6442
            ],
            [
              0.375,
              13.235
            ],
            [
              0.4688,
              14.072
            ],
            [
              0.5781,
              14.587
            ],
            [
              1.0,
              15.0
            ]
          ]
        },
        "color": [
          1.0,
          0.0,
          0.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.1167,
      "offset": [
        -3.0,
        0.0,
        0.0
      ],
      "rotate": [
        110.0,
        0.0,
        0.0
      ],
      "life": 0.6667
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-2.effect"
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
              2.5273
            ],
            [
              0.1719,
              3.0236
            ],
            [
              0.2656,
              3.3899
            ],
            [
              0.375,
              3.6791
            ],
            [
              0.5781,
              3.9249
            ],
            [
              1.0,
              4.0
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
      "at": 0.1167,
      "offset": [
        -3.0,
        0.0,
        0.0
      ],
      "rotate": [
        110.0,
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
            "inner": 0.8287,
            "band": 1.0,
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
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-3.effect"
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
              2.5273
            ],
            [
              0.1719,
              3.0236
            ],
            [
              0.2656,
              3.3899
            ],
            [
              0.375,
              3.6791
            ],
            [
              0.5781,
              3.9249
            ],
            [
              1.0,
              4.0
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
      "at": 0.1167,
      "offset": [
        -3.0,
        0.0,
        0.0
      ],
      "rotate": [
        110.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-4-0.emitter"
        }
      },
      "at": 0.1167,
      "offset": [
        -3.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow12/n0-1-4-1.emitter"
        }
      },
      "at": 0.1167,
      "offset": [
        -3.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.4073,
          0.4073
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
      "at": 0.1167,
      "life": 0.3,
      "offset": [
        -3.0,
        0.1072,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.1167
    }
  ],
  "duration": 1.8165,
  "events": [
    {
      "name": "impact",
      "time": 0.1167
    }
  ]
});
