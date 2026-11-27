import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.0.0.0 "ReimsMain": location over an f-curve: straightened
// effekseer: node 0.0.0.0.0.0.0.1 "ReimsColor": location over an f-curve: straightened
// effekseer: node 0.0.0.0.0.0.0.2 "ReimsGlow": location over an f-curve: straightened
// effekseer: node 0.0.0.0.0.0.0.3.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.0.3.0.0.0 "particleMain": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.0.3.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.0.3.0.0.1 "particleColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.0.3.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.0.3.0.0.2 "particleGlow": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.0.3.1.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.0.3.1.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.0.3.1.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.1 "Node": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.0.0.1 "Node": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.1.0.0.0.0 "ReimsMain": location over an f-curve: straightened
// effekseer: node 0.0.0.1.0.0.0.1 "ReimsColor": location over an f-curve: straightened
// effekseer: node 0.0.0.1.0.0.0.2 "ReimsGlow": location over an f-curve: straightened
// effekseer: node 0.0.0.1.0.0.0.3.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.0.0.3.0.0.0 "particleMain": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0.0.3.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.0.0.3.0.0.1 "particleColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0.0.3.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.0.0.3.0.0.2 "particleGlow": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0.0.3.1.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.0.0.3.1.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.0.0.3.1.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.1.0.1 "Node": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.1.0.1 "Node": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.0.2.0.0.0.0 "ReimsMain": location over an f-curve: straightened
// effekseer: node 0.0.0.2.0.0.0.1 "ReimsColor": location over an f-curve: straightened
// effekseer: node 0.0.0.2.0.0.0.2 "ReimsGlow": location over an f-curve: straightened
// effekseer: node 0.0.0.2.0.0.0.3.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.0.0.3.0.0.0 "particleMain": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0.0.0.3.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.0.0.3.0.0.1 "particleColor": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0.0.0.3.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.0.0.3.0.0.2 "particleGlow": nested generation: 7 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0.0.0.3.1.0.0.0 "particleMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.0.0.3.1.0.0.1 "particleColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.0.0.3.1.0.0.2 "particleGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.2.0.1 "Node": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.2.0.1 "Node": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.0.1.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.1.0.0 "Rens": a ring's arc over its life
// effekseer: node 0.0.1.1.0.0 "Rens": ring unlimited instances: the first 4 drawn
// effekseer: more than 64 elements: the last 4 left out
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-1.emitter"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-2.emitter"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-3-0-0-0.emitter"
        }
      },
      "at": 1.5333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-3-0-0-1.emitter"
        }
      },
      "at": 1.5333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-3-0-0-2.emitter"
        }
      },
      "at": 1.5333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-3-1-0-0-0.parent"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-3-1-0-0-1.parent"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-3-1-0-0-2.parent"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-4-0-0-0.parent"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-4-0-0-1.parent"
        }
      },
      "at": 1.2,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-0-0-4-0-0-2.parent"
        }
      },
      "at": 1.2,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-1.effect"
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
      "at": 1.6667,
      "rotate": [
        59.9979,
        0.6371,
        -79.8966
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-1.effect"
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
      "at": 1.6708,
      "rotate": [
        29.0888,
        55.0984,
        136.1134
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-1.effect"
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
      "at": 1.675,
      "rotate": [
        -29.0888,
        -55.0984,
        -133.8866
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-0-0-1.effect"
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
      "at": 1.6792,
      "rotate": [
        -59.9979,
        -0.6371,
        -169.8966
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-0.emitter"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-1.emitter"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-2.emitter"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-3-0-0-0.emitter"
        }
      },
      "at": 1.2667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-3-0-0-1.emitter"
        }
      },
      "at": 1.2667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-3-0-0-2.emitter"
        }
      },
      "at": 1.2667,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-3-1-0-0-0.parent"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-3-1-0-0-1.parent"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-3-1-0-0-2.parent"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-4-0-0-0.parent"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-4-0-0-1.parent"
        }
      },
      "at": 0.9333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-4-0-0-2.parent"
        }
      },
      "at": 0.9333,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
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
                0.2195,
                1.0,
                0.2195,
                0.0
              ],
              [
                0.7464,
                1.0,
                0.7464,
                1.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-1.effect"
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
      "at": 1.4,
      "rotate": [
        59.9979,
        0.6371,
        -79.8966
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
                0.2195,
                1.0,
                0.2195,
                0.0
              ],
              [
                0.7464,
                1.0,
                0.7464,
                1.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-1.effect"
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
      "at": 1.4042,
      "rotate": [
        29.0888,
        55.0984,
        136.1134
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
                0.2195,
                1.0,
                0.2195,
                0.0
              ],
              [
                0.7464,
                1.0,
                0.7464,
                1.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-1.effect"
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
      "at": 1.4083,
      "rotate": [
        -29.0888,
        -55.0984,
        -133.8866
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
                0.2195,
                1.0,
                0.2195,
                0.0
              ],
              [
                0.7464,
                1.0,
                0.7464,
                1.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-1.effect"
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
      "at": 1.4125,
      "rotate": [
        -59.9979,
        -0.6371,
        -169.8966
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-0.emitter"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-1.emitter"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-2.emitter"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-3-0-0-0.emitter"
        }
      },
      "at": 1.1333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-3-0-0-1.emitter"
        }
      },
      "at": 1.1333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-3-0-0-2.emitter"
        }
      },
      "at": 1.1333,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-3-1-0-0-0.parent"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-3-1-0-0-1.parent"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-3-1-0-0-2.parent"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-4-0-0-0.parent"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-4-0-0-1.parent"
        }
      },
      "at": 0.8,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-0-0-4-0-0-2.parent"
        }
      },
      "at": 0.8,
      "rotate": [
        0.0,
        0.0,
        -144.0
      ]
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-1.effect"
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
      "at": 1.2667,
      "rotate": [
        59.9979,
        0.6371,
        -79.8966
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-1.effect"
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
      "at": 1.2708,
      "rotate": [
        29.0888,
        55.0984,
        136.1134
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-1.effect"
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
      "at": 1.275,
      "rotate": [
        -29.0888,
        -55.0984,
        -133.8866
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
                1.0,
                0.2195,
                0.2195,
                0.0
              ],
              [
                1.0,
                0.7464,
                0.7464,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.2195,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-2-0-1.effect"
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
      "at": 1.2792,
      "rotate": [
        -59.9979,
        -0.6371,
        -169.8966
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-0-0.emitter"
        }
      },
      "at": 0.2,
      "until": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-0-1.emitter"
        }
      },
      "at": 0.2,
      "until": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-0-2.emitter"
        }
      },
      "at": 0.2,
      "until": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.1,
            "start": 267.5,
            "length": 5.0,
            "colors": [
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ],
              [
                0.9323,
                1.0,
                0.8685,
                0.4183
              ],
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.1,
            "start": 267.5,
            "length": 5.0,
            "colors": [
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ],
              [
                0.9323,
                1.0,
                0.8685,
                0.4183
              ],
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.375,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.1,
            "start": 267.5,
            "length": 5.0,
            "colors": [
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ],
              [
                0.9323,
                1.0,
                0.8685,
                0.4183
              ],
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3833,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.1,
            "start": 267.5,
            "length": 5.0,
            "colors": [
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ],
              [
                0.9323,
                1.0,
                0.8685,
                0.4183
              ],
              [
                0.7458,
                1.0,
                0.017,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-1-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.2188,
              1.9307
            ],
            [
              0.4531,
              2.3172
            ],
            [
              0.7031,
              2.6605
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          120.0
        ],
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3917,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-0-0-0-0.parent"
        }
      },
      "at": 1.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-0-0-0-1.parent"
        }
      },
      "at": 1.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-0-0-0-2.parent"
        }
      },
      "at": 1.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-1-0-0-0.parent"
        }
      },
      "at": 1.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-1-0-0-1.parent"
        }
      },
      "at": 1.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-1-0-0-2.parent"
        }
      },
      "at": 1.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-2-0-0-0.parent"
        }
      },
      "at": 1.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-2-0-0-1.parent"
        }
      },
      "at": 1.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Light3/n1-2-0-0-2.parent"
        }
      },
      "at": 1.5333
    }
  ],
  "duration": 3.3767,
  "looping": true
});
