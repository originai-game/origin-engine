import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.1.0 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.2.0.0 "ｃｒａｓｈ": a ring's arc over its life
// effekseer: node 0.2.0.0 "ｃｒａｓｈ": ring 100 instances: the first 4 drawn
// effekseer: node 1.0.0.0 "6Ring": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0.0.0 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 1.0.2.0.0.0 "shield": location over an f-curve: straightened
// effekseer: node 1.0.2.0.0.0 "shield": motion out from a circle: approximated along one direction
// effekseer: node 1.2.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.2.0.0.0.0 "LineMain": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 1.2.0.0.0.1.0 "LineMain": nested generation: 100 instances above its generator folded into one emitter
// effekseer: node 1.2.0.0.0.1.1.0 "LineMain": nested generation: 100 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.5017,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-0-0-1.emitter"
        }
      },
      "at": 0.5017,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.5017,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-1-0-0.emitter"
        }
      },
      "at": 0.435,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-1-0-1.emitter"
        }
      },
      "at": 0.435,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-1-0-1-0.emitter"
        }
      },
      "at": 0.435,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-0-2.emitter"
        }
      },
      "at": 0.435
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-1.emitter"
        }
      },
      "at": 0.5
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
                0.2618,
                0.0781,
                0.0
              ],
              [
                1.0,
                0.6189,
                0.3402,
                0.9481
              ],
              [
                1.0,
                0.2618,
                0.0781,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n0-2-0-0.effect"
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
              2.0984
            ],
            [
              0.4531,
              2.5356
            ],
            [
              0.7031,
              2.8222
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
          100.0,
          100.0,
          100.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9668,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.5944,
                0.2972,
                1.0
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.0781,
              1.6455
            ],
            [
              0.1719,
              2.0177
            ],
            [
              0.2656,
              2.2924
            ],
            [
              0.375,
              2.5093
            ],
            [
              0.4688,
              2.6235
            ],
            [
              0.5781,
              2.6937
            ],
            [
              1.0,
              2.75
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
      "at": 0.5,
      "rotate": [
        0.0,
        0.0,
        30.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9668,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.5944,
                0.2972,
                1.0
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.0781,
              1.6455
            ],
            [
              0.1719,
              2.0177
            ],
            [
              0.2656,
              2.2924
            ],
            [
              0.375,
              2.5093
            ],
            [
              0.4688,
              2.6235
            ],
            [
              0.5781,
              2.6937
            ],
            [
              1.0,
              2.75
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
      "at": 0.5167,
      "rotate": [
        0.0,
        0.0,
        30.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9668,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.5944,
                0.2972,
                1.0
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.0781,
              1.6455
            ],
            [
              0.1719,
              2.0177
            ],
            [
              0.2656,
              2.2924
            ],
            [
              0.375,
              2.5093
            ],
            [
              0.4688,
              2.6235
            ],
            [
              0.5781,
              2.6937
            ],
            [
              1.0,
              2.75
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
      "at": 0.5333,
      "rotate": [
        0.0,
        0.0,
        30.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9668,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.5944,
                0.2972,
                1.0
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.0781,
              1.6455
            ],
            [
              0.1719,
              2.0177
            ],
            [
              0.2656,
              2.2924
            ],
            [
              0.375,
              2.5093
            ],
            [
              0.4688,
              2.6235
            ],
            [
              0.5781,
              2.6937
            ],
            [
              1.0,
              2.75
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
      "at": 0.55,
      "rotate": [
        0.0,
        0.0,
        30.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-0-0-0.parent"
        }
      },
      "at": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-1-0-0.effect"
        },
        "scale": 0.0,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-2-0-0-0.emitter"
        }
      },
      "at": 0.535
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9668,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ],
              [
                1.0,
                0.5773,
                0.2737,
                1.0
              ],
              [
                1.0,
                0.3587,
                0.1166,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.0781,
              1.8955
            ],
            [
              0.1719,
              2.2677
            ],
            [
              0.2656,
              2.5424
            ],
            [
              0.375,
              2.7593
            ],
            [
              0.4688,
              2.8735
            ],
            [
              0.5781,
              2.9437
            ],
            [
              1.0,
              3.0
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
      "rotate": [
        0.0,
        0.0,
        30.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-1-0-0-0-0.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-2-0-0-0-0.effect"
        },
        "width": [
          0.5,
          0.5
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.0
        ]
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        -2.4,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-2-0-0-0-1-0.effect"
        },
        "width": [
          1.0,
          1.0
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.0
        ]
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        2.6,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun5/n1-2-0-0-0-1-1-0.effect"
        },
        "width": [
          2.5,
          2.5
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          0.0,
          3.0
        ]
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        2.6,
        0.0
      ],
      "life": 0.4167
    }
  ],
  "duration": 1.7333,
  "events": [
    {
      "name": "impact",
      "time": 0.4333
    }
  ]
});
