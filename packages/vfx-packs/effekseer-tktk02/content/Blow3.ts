import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "LineMain": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 0.0.1 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "LineColor": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 0.0.2 "LineGlow": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.2 "LineGlow": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 0.0.3.0.0.0 "particleMain": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 0.0.3 "particleNull": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.3.0.0.1 "particleGlow": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 3 "Ring": ring 10 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n0-0-0.effect"
        },
        "width": [
          3.0,
          3.0
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -6.0,
        5.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n0-0-1.effect"
        },
        "width": [
          6.0,
          6.0
        ],
        "lifetime": 0.1667,
        "color": [
          0.2195,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -6.0,
        5.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n0-0-2.effect"
        },
        "width": [
          15.0,
          15.0
        ],
        "lifetime": 0.1667,
        "color": [
          0.0478,
          0.2195,
          1.0,
          1.0
        ],
        "orbit": [
          9.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -6.0,
        5.0
      ],
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n0-0-3-0-0-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.0,
        5.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n0-0-3-0-0-1.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.0,
        5.0
      ],
      "velocity": [
        0.0,
        0.0,
        -10.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n1-0-0-0.emitter"
        }
      },
      "at": 0.4333,
      "offset": [
        0.0,
        0.75,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n1-0-0-1.emitter"
        }
      },
      "at": 0.4333,
      "offset": [
        0.0,
        0.75,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n1-1.emitter"
        }
      },
      "at": 0.4333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n2-0-0-0.emitter"
        }
      },
      "at": 0.3667,
      "offset": [
        0.0,
        0.75,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n2-0-0-1.emitter"
        }
      },
      "at": 0.3667,
      "offset": [
        0.0,
        0.75,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n2-0-0-2.emitter"
        }
      },
      "at": 0.3667,
      "offset": [
        0.0,
        0.75,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n2-1.emitter"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.375,
            "colors": [
              [
                0.269,
                0.458,
                1.0,
                0.0
              ],
              [
                0.458,
                1.0,
                1.0,
                0.1757
              ],
              [
                0.269,
                0.458,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.1094,
              1.2988
            ],
            [
              0.2188,
              2.3936
            ],
            [
              0.3281,
              3.3036
            ],
            [
              0.4531,
              4.1425
            ],
            [
              0.5781,
              4.7947
            ],
            [
              0.7031,
              5.289
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
      "at": 0.3667,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.375,
            "colors": [
              [
                0.269,
                0.458,
                1.0,
                0.0
              ],
              [
                0.458,
                1.0,
                1.0,
                0.1757
              ],
              [
                0.269,
                0.458,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.1094,
              1.2988
            ],
            [
              0.2188,
              2.3936
            ],
            [
              0.3281,
              3.3036
            ],
            [
              0.4531,
              4.1425
            ],
            [
              0.5781,
              4.7947
            ],
            [
              0.7031,
              5.289
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
      "at": 0.3833,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.375,
            "colors": [
              [
                0.269,
                0.458,
                1.0,
                0.0
              ],
              [
                0.458,
                1.0,
                1.0,
                0.1757
              ],
              [
                0.269,
                0.458,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.1094,
              1.2988
            ],
            [
              0.2188,
              2.3936
            ],
            [
              0.3281,
              3.3036
            ],
            [
              0.4531,
              4.1425
            ],
            [
              0.5781,
              4.7947
            ],
            [
              0.7031,
              5.289
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
      "at": 0.4,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.375,
            "colors": [
              [
                0.269,
                0.458,
                1.0,
                0.0
              ],
              [
                0.458,
                1.0,
                1.0,
                0.1757
              ],
              [
                0.269,
                0.458,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow3/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.1094,
              1.2988
            ],
            [
              0.2188,
              2.3936
            ],
            [
              0.3281,
              3.3036
            ],
            [
              0.4531,
              4.1425
            ],
            [
              0.5781,
              4.7947
            ],
            [
              0.7031,
              5.289
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
      "at": 0.4167,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.6234,
          0.6913,
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
      "at": 0.4333,
      "life": 0.3,
      "offset": [
        0.0,
        0.3203,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.4333
    }
  ],
  "duration": 2.4,
  "events": [
    {
      "name": "impact",
      "time": 0.4333
    }
  ]
});
