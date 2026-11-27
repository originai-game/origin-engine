import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "ReimsMain": location over an f-curve: straightened
// effekseer: node 0.0.0.0.0 "ReimsMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.1 "ReimsGlow": location over an f-curve: straightened
// effekseer: node 0.0.0.0.1 "ReimsGlow": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.2.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.2.0.0.0 "particleMain": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.2.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.2.0.0.1 "particleColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.2.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.2.0.0.2 "particleGlow": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.2.1.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "Angle": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.2.1.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.2.1.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "Distortion1": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2 "Distortion2": motion out from a circle: approximated along one direction
// effekseer: node 0.0.3 "Node": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.3 "Node": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.0.0.0.0.0 "ReimsMain": location over an f-curve: straightened
// effekseer: node 1.0.0.0.0.0 "ReimsMain": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.1 "ReimsColor": location over an f-curve: straightened
// effekseer: node 1.0.0.0.0.1 "ReimsColor": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.2 "ReimsGlow": location over an f-curve: straightened
// effekseer: node 1.0.0.0.0.2 "ReimsGlow": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.3.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.0.3.0.0.0 "particleMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.3.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.0.3.0.0.1 "particleColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.3.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.0.3.0.0.2 "particleGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.0.3.1.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0 "Angle": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0.0.3.1.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.0.0.3.1.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0.0.1 "Distortion1": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.2 "Distortion2": motion out from a circle: approximated along one direction
// effekseer: node 1.1.0.0.0.0 "ReimsMain": location over an f-curve: straightened
// effekseer: node 1.1.0.0.0.0 "ReimsMain": motion out from a circle: approximated along one direction
// effekseer: node 1.1.0.0.0.1 "ReimsColor": location over an f-curve: straightened
// effekseer: node 1.1.0.0.0.1 "ReimsColor": motion out from a circle: approximated along one direction
// effekseer: node 1.1.0.0.0.2 "ReimsGlow": location over an f-curve: straightened
// effekseer: node 1.1.0.0.0.2 "ReimsGlow": motion out from a circle: approximated along one direction
// effekseer: node 1.1.0.0.0.3.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.0.0.3.0.0.0 "particleMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0.0.3.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.0.0.3.0.0.1 "particleColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0.0.3.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.0.0.3.0.0.2 "particleGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0.0.3.1.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.0 "Angle": motion out from a circle: approximated along one direction
// effekseer: node 1.1.0.0.0.3.1.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.0.0.3.1.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.1.0.1 "Distortion1": motion out from a circle: approximated along one direction
// effekseer: node 1.1.0.2 "Distortion2": motion out from a circle: approximated along one direction
// effekseer: node 3.0.0 "Lins": a ring's arc over its life
// effekseer: node 3.0.0 "Lins": ring 100 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-1.emitter"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-2-0-0-0.emitter"
        }
      },
      "at": 0.9333,
      "offset": [
        -0.8817,
        1.2135,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-2-0-0-1.emitter"
        }
      },
      "at": 0.9333,
      "offset": [
        -0.8817,
        1.2135,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-2-0-0-2.emitter"
        }
      },
      "at": 0.9333,
      "offset": [
        -0.8817,
        1.2135,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-2-1-0-0-0.parent"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-2-1-0-0-1.parent"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-2-1-0-0-2.parent"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-3-0-0-0.parent"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-3-0-0-1.parent"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-3-0-0-2.parent"
        }
      },
      "at": 0.6,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-1.emitter"
        }
      },
      "at": 0.6
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-2.emitter"
        }
      },
      "at": 1.0017
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.875,
            "start": 165.0,
            "length": 210.0,
            "colors": [
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ],
              [
                1.0,
                1.0,
                0.8055,
                1.0
              ],
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.1094,
              1.9329
            ],
            [
              0.2188,
              2.2979
            ],
            [
              0.3281,
              2.6012
            ],
            [
              0.4531,
              2.8808
            ],
            [
              0.5781,
              3.0982
            ],
            [
              0.7031,
              3.263
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.0667,
      "rotate": [
        0.0,
        0.0,
        81.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.875,
            "start": 165.0,
            "length": 210.0,
            "colors": [
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ],
              [
                1.0,
                1.0,
                0.8055,
                1.0
              ],
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.1094,
              1.9329
            ],
            [
              0.2188,
              2.2979
            ],
            [
              0.3281,
              2.6012
            ],
            [
              0.4531,
              2.8808
            ],
            [
              0.5781,
              3.0982
            ],
            [
              0.7031,
              3.263
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.0708,
      "rotate": [
        0.0,
        0.0,
        171.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.875,
            "start": 165.0,
            "length": 210.0,
            "colors": [
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ],
              [
                1.0,
                1.0,
                0.8055,
                1.0
              ],
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.1094,
              1.9329
            ],
            [
              0.2188,
              2.2979
            ],
            [
              0.3281,
              2.6012
            ],
            [
              0.4531,
              2.8808
            ],
            [
              0.5781,
              3.0982
            ],
            [
              0.7031,
              3.263
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.075,
      "rotate": [
        0.0,
        0.0,
        -99.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9336,
            "band": 0.875,
            "start": 165.0,
            "length": 210.0,
            "colors": [
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ],
              [
                1.0,
                1.0,
                0.8055,
                1.0
              ],
              [
                0.7602,
                1.0,
                0.0254,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.1094,
              1.9329
            ],
            [
              0.2188,
              2.2979
            ],
            [
              0.3281,
              2.6012
            ],
            [
              0.4531,
              2.8808
            ],
            [
              0.5781,
              3.0982
            ],
            [
              0.7031,
              3.263
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          1200.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.0792,
      "rotate": [
        0.0,
        0.0,
        -9.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-0.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-1.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-2.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-3-0-0-0.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        -0.375,
        -1.299,
        0.6495
      ],
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-3-0-0-1.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        -0.375,
        -1.299,
        0.6495
      ],
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-3-0-0-2.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        -0.375,
        -1.299,
        0.6495
      ],
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-3-1-0-0-0.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-3-1-0-0-1.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-3-1-0-0-2.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-4-0-0-0.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-4-0-0-1.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-4-0-0-2.parent"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-1.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-2.emitter"
        }
      },
      "at": 0.5683,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-3.effect"
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
              2.4071
            ],
            [
              0.2188,
              3.593
            ],
            [
              0.3281,
              4.5789
            ],
            [
              0.4531,
              5.4878
            ],
            [
              0.5781,
              6.1942
            ],
            [
              0.7031,
              6.7297
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
          0.0478,
          0.1673
        ],
        "blend": "additive"
      },
      "at": 0.6,
      "rotate": [
        -33.6901,
        -25.6589,
        -16.1021
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-3-0-0-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.3863,
        -1.2858,
        0.6691
      ],
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-3-0-0-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.3863,
        -1.2858,
        0.6691
      ],
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-3-0-0-2.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        0.3863,
        -1.2858,
        0.6691
      ],
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-3-1-0-0-0.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-3-1-0-0-1.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-3-1-0-0-2.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-4-0-0-0.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-4-0-0-1.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-4-0-0-2.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-1.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-2.emitter"
        }
      },
      "at": 0.235,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-3.effect"
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
              2.4071
            ],
            [
              0.2188,
              3.593
            ],
            [
              0.3281,
              4.5789
            ],
            [
              0.4531,
              5.4878
            ],
            [
              0.5781,
              6.1942
            ],
            [
              0.7031,
              6.7297
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
          0.0478,
          0.1673
        ],
        "blend": "additive"
      },
      "at": 0.2667,
      "rotate": [
        -34.7535,
        25.3777,
        16.5602
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n2.effect"
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
              3.3727
            ],
            [
              0.1719,
              5.6063
            ],
            [
              0.2656,
              7.2544
            ],
            [
              0.375,
              8.5559
            ],
            [
              0.4688,
              9.2407
            ],
            [
              0.5781,
              9.6621
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
          0.0478,
          0.251
        ],
        "blend": "additive"
      },
      "at": 1.1,
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
            "start": 267.5,
            "length": 5.0,
            "colors": [
              [
                1.0,
                1.0,
                0.0,
                0.0
              ],
              [
                1.0,
                1.0,
                0.2195,
                0.502
              ],
              [
                1.0,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light4/n3-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.2188,
              2.5984
            ],
            [
              0.4531,
              3.0356
            ],
            [
              0.7031,
              3.3222
            ],
            [
              1.0,
              3.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          90.0
        ],
        "color": [
          100.0,
          100.0,
          100.0,
          0.6667
        ],
        "blend": "additive"
      },
      "at": 1.0333,
      "life": 0.3333
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.8625,
          1.0,
          0.541
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
      "at": 0.6,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.6
    }
  ],
  "duration": 2.7767,
  "events": [
    {
      "name": "impact",
      "time": 0.6
    }
  ]
});
