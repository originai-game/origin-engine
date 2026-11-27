import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.1 "LineColoe": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.1.0 "LineColoe": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.0.0.1 "LineColoe": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.0.0.1.0 "LineColoe": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2.0.0.0.1.0 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.2.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.3 "Ring": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-0-0-0-0.parent"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-0-0-0-1.parent"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-0-0-0-1-0.parent"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-1-0-0-0.parent"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-1-0-0-1.parent"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-1-0-0-1-0.parent"
        }
      },
      "at": 0.05
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-2-0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-2-0-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-2-0-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-2-1.emitter"
        }
      },
      "at": 0.0017
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8287,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.4134,
                0.147,
                0.0
              ],
              [
                1.0,
                0.722,
                0.4892,
                0.502
              ],
              [
                1.0,
                0.4134,
                0.147,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.125,
              1.6733
            ],
            [
              0.2188,
              2.1029
            ],
            [
              0.3125,
              2.4405
            ],
            [
              0.4219,
              2.713
            ],
            [
              0.5156,
              2.8528
            ],
            [
              0.6094,
              2.9245
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
        "color": [
          3.0,
          3.0,
          3.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1917,
      "life": 0.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n0-0-4.emitter"
        }
      },
      "at": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n1-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.8682,
        4.924
      ],
      "rotate": [
        10.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        10.4189,
        -59.0885
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n1-0-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.8682,
        4.924
      ],
      "rotate": [
        10.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        10.4189,
        -59.0885
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n1-0-0-0-0-2-0.effect"
        },
        "width": [
          0.6667,
          0.6667
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.8682,
        4.924
      ],
      "rotate": [
        10.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        10.4189,
        -59.0885
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun6/n1-0-0-0-0-2-1-0.effect"
        },
        "width": [
          3.3333,
          3.3333
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.8682,
        4.924
      ],
      "rotate": [
        10.0,
        0.0,
        0.0
      ],
      "velocity": [
        0.0,
        10.4189,
        -59.0885
      ],
      "life": 0.1667
    }
  ],
  "duration": 2.2167,
  "events": [
    {
      "name": "impact",
      "time": 0.05
    }
  ]
});
