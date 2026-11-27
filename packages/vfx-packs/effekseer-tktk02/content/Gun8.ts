import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.1 "LineColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.2 "LineGlow": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 1.0.0 "RingMain": ring 9 instances: the first 4 drawn
// effekseer: node 1.1.0.0 "Ring5": ring 5 instances: the first 4 drawn
// effekseer: node 2.0.0.0.0 "Ring": ring 9 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n0-0-0-0-0.emitter"
        }
      },
      "at": 1.1683,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n0-0-0-0-1.emitter"
        }
      },
      "at": 1.1683,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n0-0-0-0-2.emitter"
        }
      },
      "at": 1.1683,
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
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n0-0-1.emitter"
        }
      },
      "at": 1.1683
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n0-0-2.emitter"
        }
      },
      "at": 1.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.975,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.1667,
      "rotate": [
        0.0,
        0.0,
        -135.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.975,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.1833,
      "rotate": [
        0.0,
        0.0,
        -45.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.975,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.2,
      "rotate": [
        0.0,
        0.0,
        45.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.975,
            "band": 0.875,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.2167,
      "rotate": [
        0.0,
        0.0,
        135.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9762,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.1667,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9762,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.2,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9762,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.2333,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9762,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ],
              [
                1.0,
                0.616,
                0.3269,
                1.0
              ],
              [
                1.0,
                0.2707,
                0.0714,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n1-1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 1.2667,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.75,
            "band": 1.0,
            "colors": [
              [
                1.0,
                0.2444,
                0.0589,
                0.0
              ],
              [
                1.0,
                0.5751,
                0.2703,
                1.0
              ],
              [
                1.0,
                0.2444,
                0.0589,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Gun8/n2-0-0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.6875,
              2.0
            ],
            [
              0.7031,
              2.1875
            ],
            [
              0.7969,
              7.8125
            ],
            [
              0.8125,
              8.25
            ],
            [
              0.8906,
              9.8125
            ],
            [
              0.9062,
              10.0
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          9.0,
          9.0,
          9.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "life": 1.6667
    }
  ],
  "duration": 2.1,
  "events": [
    {
      "name": "impact",
      "time": 1.1667
    }
  ]
});
