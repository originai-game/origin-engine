import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1.1 "リング縁": a UV region of a texture whose size is not known (not a PNG)
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 1.6667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        15.7569,
        -2.7784,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-1.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 1.6667,
        "color": [
          0.047,
          0.047,
          0.047,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        15.7569,
        -2.7784,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        17.2752,
        -3.0461,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -100.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-2-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        17.2752,
        -3.0461,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -100.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-2-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        17.2752,
        -3.0461,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -100.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-2-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        24.1073,
        -4.2508,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -100.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-0-0-2-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        24.1073,
        -4.2508,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -100.0
      ],
      "velocity": [
        -35.4531,
        6.2513,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-1-0.effect"
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
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "rotate": [
        25.5056,
        67.7313,
        62.7268
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9827,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-1-1.effect"
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
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "rotate": [
        25.5056,
        67.7313,
        62.7268
      ],
      "life": 0.5
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
                0.5356,
                0.5356,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-1-2.effect"
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
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
            ],
            [
              1.0,
              6.0
            ]
          ]
        },
        "color": [
          2.0,
          2.0,
          2.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1333,
      "rotate": [
        25.5056,
        67.7313,
        62.7268
      ],
      "life": 0.5
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
                0.047,
                0.0478,
                0.0478,
                0.0
              ],
              [
                0.047,
                0.0478,
                0.0478,
                0.0
              ],
              [
                0.047,
                0.0478,
                0.0478,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-1-3.effect"
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
              3.0546
            ],
            [
              0.1719,
              4.0472
            ],
            [
              0.2656,
              4.7797
            ],
            [
              0.375,
              5.3582
            ],
            [
              0.4688,
              5.6625
            ],
            [
              0.5781,
              5.8498
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
      "at": 0.1333,
      "rotate": [
        25.5056,
        67.7313,
        62.7268
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-2-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        -1.231,
        0.2171,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-2-0-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        -1.231,
        0.2171,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-3-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        -1.231,
        0.2171,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-3-0-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        -1.231,
        0.2171,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword21/n0-3-0-1.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        -1.231,
        0.2171,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        80.0
      ]
    }
  ],
  "duration": 3.3333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
