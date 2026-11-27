import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0.0 "破裂": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.1.0.0 "〆破裂リング": ring 5 instances: the first 4 drawn
// effekseer: node 1.1.0.0 "〆破裂リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.1.1.0 "〆破裂リング": ring 5 instances: the first 4 drawn
// effekseer: node 1.1.1.0 "〆破裂リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n0-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        9.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        -8.5,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-1-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -8.5,
        0.0,
        0.0
      ],
      "rotate": [
        15.0,
        90.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8142,
            "band": 0.5,
            "colors": [
              [
                0.0,
                1.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.75
            ],
            [
              1.0,
              2.5
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        -8.5,
        0.0,
        0.0
      ],
      "rotate": [
        15.0,
        90.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-1-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -8.5,
        0.0,
        0.0
      ],
      "rotate": [
        15.0,
        90.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8142,
            "band": 0.5,
            "colors": [
              [
                0.0,
                1.0,
                0.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.2195,
                0.0
              ],
              [
                0.2195,
                1.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-1-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.75
            ],
            [
              1.0,
              2.5
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        -8.5,
        0.0,
        0.0
      ],
      "rotate": [
        15.0,
        90.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-2.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -8.5,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-3.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -8.5,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-4-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -7.5,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-5-0-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -7.0,
        0.0,
        0.0
      ],
      "rotate": [
        16.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Wind2/n1-5-1-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        -7.0,
        0.0,
        0.0
      ],
      "rotate": [
        16.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.4247,
          1.0,
          0.4247
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
      "at": 0.1667,
      "life": 0.3,
      "offset": [
        -8.2724,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.1667
    }
  ],
  "duration": 0.6733,
  "events": [
    {
      "name": "impact",
      "time": 0.1667
    }
  ]
});
