import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1.0 "Distortion2": motion out from a circle: approximated along one direction
// effekseer: node 0.3.0 "Distortion1": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.0.0 "LineMain": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.1.0.0 "LineGlow": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.1.1 "LineColor": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 1.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 2.0.0.0 "SwordMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0.0 "SwordMain": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 2.0.0.1.0 "SwordColor": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 2.0.0.1.1.0 "SwordGlow": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 2.0.0.2.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0.2.0 "particleMain": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 2.0.0.2.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0.2.1 "particleColor": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 2.0.0.2.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 2.0.0.2.2 "particleGlow": nested generation: 6 instances above its generator folded into one emitter
// effekseer: node 3.0.0.2 "LineColor": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 3.1.0.2 "LineColor": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 4.1.0 "Distortion2": motion out from a circle: approximated along one direction
// effekseer: node 4.3.0 "Distortion1": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-0-0-0-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-0-0-0-1-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-0-0-0-1-1-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-1-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-2-0-0-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-2-0-0-1-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-2-0-0-1-1-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-3-0.parent"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n0-4.emitter"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n1-0-0-0-0-0.effect"
        },
        "width": [
          0.5,
          0.5
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        3.5,
        0.0,
        0.0
      ],
      "rotate": [
        30.0,
        0.0,
        0.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n1-0-0-0-0-1-0-0.effect"
        },
        "width": [
          1.5556,
          1.5556
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        3.5,
        0.0,
        0.0
      ],
      "rotate": [
        30.0,
        0.0,
        0.0
      ],
      "velocity": [
        -7.0,
        0.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n1-0-0-0-0-1-1.effect"
        },
        "width": [
          0.6667,
          0.6667
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        3.5,
        0.0,
        0.0
      ],
      "rotate": [
        30.0,
        0.0,
        0.0
      ],
      "velocity": [
        -7.0,
        0.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n1-1.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        30.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n1-2.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        30.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n1-3.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        30.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n2-0-0-0.effect"
        },
        "width": [
          0.5833,
          0.5833
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          1.75,
          0.4
        ]
      },
      "at": 0.2667,
      "offset": [
        0.0,
        -1.75,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n2-0-0-1-0.effect"
        },
        "width": [
          0.7778,
          0.7778
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          2.3333,
          0.4
        ]
      },
      "at": 0.2667,
      "offset": [
        0.0,
        -1.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        19.6,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n2-0-0-1-1-0.effect"
        },
        "width": [
          1.8148,
          1.8148
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          5.4444,
          0.4
        ]
      },
      "at": 0.2667,
      "offset": [
        0.0,
        1.9444,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        19.6,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n2-0-0-2-0.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.0,
        -3.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n2-0-0-2-1.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.0,
        -3.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n2-0-0-2-2.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.0,
        -3.5,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n3-0-0-0.effect"
        },
        "scale": 1.1547,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "offset": [
        4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ],
      "velocity": [
        -16.6277,
        -9.6,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n3-0-0-1.effect"
        },
        "scale": 2.3094,
        "color": [
          0.2195,
          0.0478,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "offset": [
        4.5611,
        2.6333,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ],
      "velocity": [
        -16.6277,
        -9.6,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n3-0-0-2.effect"
        },
        "scale": 4.2164,
        "color": [
          0.2195,
          0.0478,
          1.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "offset": [
        4.792,
        2.7667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ],
      "velocity": [
        -16.6277,
        -9.6,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n3-1-0-0.effect"
        },
        "scale": 1.1547,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "offset": [
        -4.3301,
        2.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -120.0
      ],
      "velocity": [
        16.6277,
        -9.6,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n3-1-0-1.effect"
        },
        "scale": 2.3094,
        "color": [
          0.2195,
          0.0478,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "offset": [
        -4.5611,
        2.6333,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -120.0
      ],
      "velocity": [
        16.6277,
        -9.6,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n3-1-0-2.effect"
        },
        "scale": 4.2164,
        "color": [
          0.2195,
          0.0478,
          1.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.8333,
      "offset": [
        -4.792,
        2.7667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -120.0
      ],
      "velocity": [
        16.6277,
        -9.6,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-0-0-0-0.emitter"
        }
      },
      "at": 0.9,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-0-0-0-1-0.emitter"
        }
      },
      "at": 0.9,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-0-0-0-1-1-0.emitter"
        }
      },
      "at": 0.9,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-1-0.emitter"
        }
      },
      "at": 0.9
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-2-0-0-0.emitter"
        }
      },
      "at": 0.9,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-2-0-0-1-0.emitter"
        }
      },
      "at": 0.9,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-2-0-0-1-1-0.emitter"
        }
      },
      "at": 0.9,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-3-0.emitter"
        }
      },
      "at": 0.9
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Dark2/n4-4.emitter"
        }
      },
      "at": 0.9
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.9759,
          0.9743,
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
      "at": 0.3667,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.3667
    }
  ],
  "duration": 3.0333,
  "events": [
    {
      "name": "impact",
      "time": 0.3667
    }
  ]
});
