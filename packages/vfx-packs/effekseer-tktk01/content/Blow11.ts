import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1.0 "レンズフレア": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.0 "レンズフレアglow": motion out from a circle: approximated along one direction
// effekseer: node 0.3.1 "衝撃リング外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.3.2 "衝撃リング白": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 0.3.3 "白色リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-0-0.emitter"
        }
      },
      "at": 0.0667,
      "offset": [
        1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-0-0-0.emitter"
        }
      },
      "at": 0.0667,
      "offset": [
        1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-0-0-1.emitter"
        }
      },
      "at": 0.0667,
      "offset": [
        1.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-1-0.emitter"
        }
      },
      "at": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-1-0-0.emitter"
        }
      },
      "at": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-2.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-2-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-2-1.emitter"
        }
      },
      "at": 0.0
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
                0.0478,
                0.0478,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.25,
              3.0143
            ],
            [
              0.3594,
              3.6201
            ],
            [
              0.4531,
              4.0265
            ],
            [
              0.5469,
              4.3319
            ],
            [
              0.6562,
              4.5789
            ],
            [
              1.0,
              5.0
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
      "at": 0.0833,
      "rotate": [
        104.974,
        -20.2544,
        -52.3106
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.815,
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
                0.502
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-3-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.25,
              3.0143
            ],
            [
              0.3594,
              3.6201
            ],
            [
              0.4531,
              4.0265
            ],
            [
              0.5469,
              4.3319
            ],
            [
              0.6562,
              4.5789
            ],
            [
              1.0,
              5.0
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
      "rotate": [
        104.974,
        -20.2544,
        -52.3106
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.815,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-3-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.25,
              3.0143
            ],
            [
              0.3594,
              3.6201
            ],
            [
              0.4531,
              4.0265
            ],
            [
              0.5469,
              4.3319
            ],
            [
              0.6562,
              4.5789
            ],
            [
              1.0,
              5.0
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
      "at": 0.0833,
      "rotate": [
        104.974,
        -20.2544,
        -52.3106
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2069,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-3-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.25
            ],
            [
              0.25,
              3.0143
            ],
            [
              0.3594,
              3.6201
            ],
            [
              0.4531,
              4.0265
            ],
            [
              0.5469,
              4.3319
            ],
            [
              0.6562,
              4.5789
            ],
            [
              1.0,
              5.0
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
      "rotate": [
        104.974,
        -20.2544,
        -52.3106
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-4-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-4-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.0,
        1.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-5-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-5-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          1.0,
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
      "at": 0.0833,
      "life": 0.3,
      "offset": [
        0.0,
        0.0911,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0833
    }
  ],
  "duration": 0.6498,
  "events": [
    {
      "name": "impact",
      "time": 0.0833
    }
  ]
});
