import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "Node": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0 "Node": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.1.0.0.0.0.0.0.0 "particleMain": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0.0.0.0.0.1 "particleColor": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0.0.0.0.0.1.0 "particleColor": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.1.0.0.0.0.0.0.2 "Distortion": nested generation: 50 instances above its generator folded into one emitter
// effekseer: node 1.2.0 "Rast": a ring's arc over its life
// effekseer: node 1.2.0 "Rast": ring 200 instances: the first 4 drawn
// effekseer: node 4 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 5 "ｉｎｔｅｒｖａｌ": motion out from a circle: approximated along one direction
// effekseer: node 5.1.0.0.0.2.0.0.0 "particleMain": nested generation: 150 instances above its generator folded into one emitter
// effekseer: node 5.1.0.0.0.2.0.0.1 "particleColor": nested generation: 150 instances above its generator folded into one emitter
// effekseer: node 5.1.0.0.0.2.0.0.1.0 "particleColor": nested generation: 150 instances above its generator folded into one emitter
// effekseer: node 5.1.0.0.0.2.0.0.2 "Distortion": nested generation: 150 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8809,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0478,
                0.0104,
                0.0
              ],
              [
                1.0,
                0.1813,
                0.0478,
                0.2928
              ],
              [
                1.0,
                0.0478,
                0.0104,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n0-0-0.effect"
        },
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.5
            ],
            [
              0.5312,
              3.1741
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.2667,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-0-0-0-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.3,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-0-0-0-1.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.3,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-0-0-0-1-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.3,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-0-0-0-2-0-0-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.3,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-0-0-0-2-0-0-1.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.3,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-0-0-0-2-0-0-1-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.3,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        4.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-0-0-0-1.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        4.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        4.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-0-0-0-2.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        4.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-1.emitter"
        }
      },
      "at": 0.4,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-2.emitter"
        }
      },
      "at": 0.4,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-1-0-0-0-3.emitter"
        }
      },
      "at": 0.4,
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
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "start": 268.75,
            "length": 2.5,
            "colors": [
              [
                1.0,
                0.0478,
                0.0104,
                0.0
              ],
              [
                1.0,
                0.8267,
                0.7201,
                0.502
              ],
              [
                1.0,
                0.0478,
                0.0104,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-2-0.effect"
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
              1.5273
            ],
            [
              0.1719,
              2.0236
            ],
            [
              0.2656,
              2.3899
            ],
            [
              0.375,
              2.6791
            ],
            [
              0.4688,
              2.8313
            ],
            [
              0.5781,
              2.9249
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
          20.0
        ],
        "color": [
          200.0,
          200.0,
          200.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.4,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n1-3.emitter"
        }
      },
      "at": 0.5667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n2-0-0-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n2-0-0-1.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n2-0-0-1-0.emitter"
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
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n3-0-0-0.emitter"
        }
      },
      "at": 0.235,
      "offset": [
        0.0,
        3.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n3-0-0-1.emitter"
        }
      },
      "at": 0.235,
      "offset": [
        0.0,
        3.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n3-0-0-1-0.emitter"
        }
      },
      "at": 0.235,
      "offset": [
        0.0,
        3.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n4.emitter"
        }
      },
      "at": 0.2683
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-0-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-0-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-0-0-0-2-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-0-0-0-2-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-0-0-0-2-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-2-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-2-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-2-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-2-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-3.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-4.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-5.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n6.emitter"
        }
      },
      "at": 0.1667
    }
  ],
  "duration": 2.5333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
