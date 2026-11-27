import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Fire": ring 6 instances: the first 4 drawn
// effekseer: node 0.0.0.0 "Fire": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0 "Distortion": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1 "Glow": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2.0 "Fog": nested generation: 10 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0 "Fog": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.2.0 "Fog": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.0.2.1 "Distortion": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.2.1 "Distortion": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.0 "FireNull": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0.0 "Fire": ring 6 instances: the first 4 drawn
// effekseer: node 1.0.0.0.0 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 1.0.2.0 "Fog": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.0.2.0 "Fog": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 1.0.2.1 "Distortion": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.4531,
              1.0224
            ],
            [
              1.0,
              1.25
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
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
        -135.0
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.4531,
              1.0224
            ],
            [
              1.0,
              1.25
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
      "at": 0.0833,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
        -45.0
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.4531,
              1.0224
            ],
            [
              1.0,
              1.25
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
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
        45.0
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.75
            ],
            [
              0.4531,
              1.0224
            ],
            [
              1.0,
              1.25
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
      "at": 0.25,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -90.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-0-0-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-2-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        -180.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-2-1.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "rotate": [
        90.0,
        0.0,
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.4531,
              2.0448
            ],
            [
              1.0,
              2.5
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
        -90.0,
        0.0,
        -135.0
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.4531,
              2.0448
            ],
            [
              1.0,
              2.5
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
      "at": 0.5833,
      "rotate": [
        -90.0,
        0.0,
        -45.0
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.4531,
              2.0448
            ],
            [
              1.0,
              2.5
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
        -90.0,
        0.0,
        45.0
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
            "band": 0.5,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.5356,
                1.0,
                1.0,
                0.251
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.4531,
              2.0448
            ],
            [
              1.0,
              2.5
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
      "at": 0.75,
      "rotate": [
        -90.0,
        0.0,
        135.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-0-0-0.parent"
        }
      },
      "at": 0.5,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-1.effect"
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
              5.4455
            ],
            [
              0.4531,
              8.5377
            ],
            [
              0.7031,
              11.2839
            ],
            [
              1.0,
              14.0
            ]
          ]
        },
        "color": [
          1.0,
          0.147,
          0.032,
          0.7503
        ],
        "blend": "additive"
      },
      "at": 0.5,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-2-0.parent"
        }
      },
      "at": 0.5,
      "rotate": [
        -180.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n1-0-2-1.emitter"
        }
      },
      "at": 0.5,
      "rotate": [
        -180.0,
        0.0,
        180.0
      ]
    }
  ],
  "duration": 2.9983,
  "events": [
    {
      "name": "impact",
      "time": 0.5
    }
  ]
});
