import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.3 "GlowCore": a mesh's colour over its life: its brightest kept, alpha over life kept
// effekseer: node 0.4 "Glow": a mesh's colour over its life: its brightest kept, alpha over life kept
// effekseer: node 1.0.0.0.0.0.0 "LineColor": nested generation: 35 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.0.1.0 "LineGlow": nested generation: 35 instances above its generator folded into one emitter
// effekseer: node 2.0.0.0.0 "Fire": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 2.1 "Glow": a mesh's colour over its life: its brightest kept, alpha over life kept
// effekseer: node 3.0.0.0.0.0 "Fire": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 3.0.0.0.0.1 "Distortion": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.0,
        0.0,
        -0.5
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-0-0-0-1.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.0,
        0.0,
        -0.4
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-0-0-0-2.emitter"
        }
      },
      "at": 0.2667,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-1.emitter"
        }
      },
      "at": 0.335,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-2-0-0-0.emitter"
        }
      },
      "at": 0.2667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-2-0-0-1.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.0,
        0.5,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.1094,
              1.8659
            ],
            [
              0.2188,
              2.5957
            ],
            [
              0.3281,
              3.2024
            ],
            [
              0.4531,
              3.7617
            ],
            [
              0.5781,
              4.1964
            ],
            [
              0.7031,
              4.526
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          1.0,
          0.8501,
          0.0406,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2667,
      "rotate": [
        -90.0,
        0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n0-4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.1094,
              2.9482
            ],
            [
              0.2188,
              4.5903
            ],
            [
              0.3281,
              5.9555
            ],
            [
              0.4531,
              7.2138
            ],
            [
              0.5781,
              8.192
            ],
            [
              0.7031,
              8.9334
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.0,
          0.4423,
          0.0385,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2667,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n1-0-0-0-0-0-0.effect"
        },
        "width": [
          6.0,
          6.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          18.0,
          2.0
        ]
      },
      "at": 0.0,
      "offset": [
        -15.0,
        2.0,
        0.0
      ],
      "velocity": [
        -7.2,
        -4.8,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n1-0-0-0-0-0-1-0.effect"
        },
        "width": [
          30.0,
          30.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          90.0,
          2.0
        ]
      },
      "at": 0.0,
      "offset": [
        -87.0,
        2.0,
        0.0
      ],
      "velocity": [
        -7.2,
        -4.8,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n2-0-0-0-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.0,
        -0.5
      ],
      "rotate": [
        -90.0,
        0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n2-1.effect"
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
              4.0823
            ],
            [
              0.2188,
              4.9946
            ],
            [
              0.4531,
              6.4521
            ],
            [
              0.7031,
              7.4075
            ],
            [
              1.0,
              8.0
            ]
          ]
        },
        "color": [
          1.0,
          0.531,
          0.0254,
          0.8755
        ],
        "blend": "additive"
      },
      "at": 0.4,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n3-0-0-0-0-0.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire2/n3-0-0-0-0-1.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 3.3167,
  "events": [
    {
      "name": "impact",
      "time": 0.2667
    }
  ]
});
