import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "斬撃": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.0.0 "glow": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.1.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.4.0 "粒子": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.4.0.0 "glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.4.0.1 "glow大": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.4.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.4.0.2 "軌跡": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 0.0.1.4.0.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1.4.0.3 "軌跡glow": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 0.1.0 "斬撃": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.1.0.0 "glow": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.1.1.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.4.0 "粒子": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.4.0.0 "glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.4.0.1 "glow大": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.4.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.4.0.2 "軌跡": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 0.1.1.4.0.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.1.4.0.3 "軌跡glow": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 0.2.0 "斬撃": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.2.0.0 "glow": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.2.1.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.4.0 "粒子": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.4.0.0 "glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.4.0.1 "glow大": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.4.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.4.0.2 "軌跡": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 0.2.1.4.0.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.1.4.0.3 "軌跡glow": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 1.0 "斬撃": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 1.0.0 "glow": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 1.1.1 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.2 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.0 "粒子": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.0 "粒子": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 1.1.3.0.0 "glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.0.0 "glow": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 1.1.3.0.1 "glow大": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.0.1 "glow大": nested generation: 250 instances above its generator folded into one emitter
// effekseer: node 1.1.3.0.2 "軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.0.2 "軌跡": nested generation: 500 instances above its generator folded into one emitter
// effekseer: node 1.1.3.0.3 "軌跡glow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.0.3 "軌跡glow": nested generation: 500 instances above its generator folded into one emitter
// effekseer: node 1.1.3.1 "ライン衝撃": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.3.1.0 "giow": a moving ancestor: its descendants are placed where it starts
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.5,
              1.5811
            ],
            [
              0.625,
              1.2076
            ],
            [
              0.7188,
              0.8229
            ],
            [
              0.7656,
              0.5352
            ],
            [
              0.7812,
              0.3953
            ],
            [
              0.8125,
              0.0
            ],
            [
              1.0,
              0.0
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
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -2.5981,
        -1.5,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-0-0.effect"
        },
        "scale": 2.5927,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -2.5981,
        -1.5,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -2.5981,
        -1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -2.5981,
        -1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-1-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -2.5981,
        -1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-2.effect"
        },
        "width": [
          1.0833,
          0.2708
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-3.effect"
        },
        "width": [
          2.1667,
          0.5417
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-4-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-4-0-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-4-0-1.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-4-0-2.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.1333,
      "offset": [
        4.8301,
        1.634,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-0-1-4-0-3.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          0.0,
          0.2195,
          0.3922
        ]
      },
      "at": 0.1333,
      "offset": [
        4.8301,
        1.634,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.4062,
              1.5646
            ],
            [
              0.5312,
              1.1859
            ],
            [
              0.625,
              0.7906
            ],
            [
              0.6719,
              0.4841
            ],
            [
              0.6875,
              0.3227
            ],
            [
              0.7031,
              0.0
            ],
            [
              1.0,
              0.0
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
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        2.5981,
        -1.5,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-0-0.effect"
        },
        "scale": 2.5927,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        2.5981,
        -1.5,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-1-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        2.5981,
        -1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-1-0-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        2.5981,
        -1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-1-0-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        2.5981,
        -1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-2.effect"
        },
        "width": [
          1.0833,
          0.2708
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-3.effect"
        },
        "width": [
          2.1667,
          0.5417
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ]
      },
      "at": 0.1667,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-4-0.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-4-0-0.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-4-0-1.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-4-0-2.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.3,
      "offset": [
        -3.8301,
        3.366,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-1-1-4-0-3.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          0.0,
          0.2195,
          0.3922
        ]
      },
      "at": 0.3,
      "offset": [
        -3.8301,
        3.366,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.3906,
              1.5811
            ],
            [
              0.4062,
              1.5562
            ],
            [
              0.4688,
              1.2809
            ],
            [
              0.5469,
              0.8149
            ],
            [
              0.5781,
              0.5229
            ],
            [
              0.6094,
              0.0
            ],
            [
              1.0,
              0.0
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
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.0,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-0-0.effect"
        },
        "scale": 2.5927,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.0,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-1-0.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-1-0-0.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-1-0-1.emitter"
        }
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-2.effect"
        },
        "width": [
          1.0833,
          0.2708
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-3.effect"
        },
        "width": [
          2.1667,
          0.5417
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ]
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-4-0.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        5.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-4-0-0.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        5.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-4-0-1.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        5.0,
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
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-4-0-2.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.4667,
      "offset": [
        5.0,
        -1.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n0-2-1-4-0-3.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          0.0,
          0.2195,
          0.3922
        ]
      },
      "at": 0.4667,
      "offset": [
        5.0,
        -1.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.3717
            ],
            [
              0.1094,
              2.3367
            ],
            [
              0.2812,
              1.5617
            ],
            [
              0.3438,
              1.1577
            ],
            [
              0.3906,
              0.7186
            ],
            [
              0.4062,
              0.4916
            ],
            [
              0.4219,
              0.0
            ],
            [
              1.0,
              0.0
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
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "velocity": [
        0.0,
        -3.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-0-0.effect"
        },
        "scale": 3.8891,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "velocity": [
        0.0,
        -3.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-1.effect"
        },
        "width": [
          1.5417,
          0.3854
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-2.effect"
        },
        "width": [
          3.0833,
          0.7708
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ]
      },
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-0-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-0-1.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
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
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-0-2.effect"
        },
        "width": [
          0.125,
          0.0312
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.3922
        ]
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
        0.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-0-3.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          0.0,
          0.2195,
          0.3922
        ]
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
        0.0
      ],
      "life": 0.0833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-1.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-3-1-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        1.0,
        5.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-4-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "velocity": [
        0.0,
        -3.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-4-0-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "velocity": [
        0.0,
        -3.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword19/n1-1-4-0-1.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        0.0,
        5.0,
        0.0
      ],
      "velocity": [
        0.0,
        -3.0,
        0.0
      ]
    }
  ],
  "duration": 2.49,
  "events": [
    {
      "name": "impact",
      "time": 0.6667
    }
  ]
});
