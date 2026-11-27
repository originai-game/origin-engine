import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 2 "Energy": motion out from a circle: approximated along one direction
// effekseer: node 3 "Attracted Energy": attraction to the target
// effekseer: node 3.0 "Attacted Energy Trail": a moving ancestor: its descendants are placed where it starts
// effekseer: node 3.0 "Attacted Energy Trail": nested generation: 15 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
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
                0.2195,
                0.0,
                1.0,
                0.0
              ],
              [
                0.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
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
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 2.0
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
                0.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              1.0,
              12.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.05,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n2.emitter"
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
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n3-0.effect"
        },
        "width": [
          1.1133,
          0.2783
        ],
        "lifetime": 0.6667,
        "color": [
          0.0,
          0.2195,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              1.0,
              46.0
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
      "at": 2.25,
      "rotate": [
        -180.0,
        70.0,
        -180.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n5.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              1.0,
              46.0
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
      "at": 2.3333,
      "rotate": [
        70.0,
        0.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7273,
            "band": 0.5,
            "height": [
              0.0,
              54.5455
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n6.effect"
        },
        "scale": 0.55,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 2.3333,
      "life": 1666.6666
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.875,
            "band": 0.5,
            "height": [
              0.0,
              37.5
            ],
            "colors": [
              [
                0.2932,
                0.6524,
                0.9914,
                1.0
              ],
              [
                0.0,
                1.0,
                1.0,
                1.0
              ],
              [
                0.0,
                0.2195,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n7.effect"
        },
        "scale": 0.8,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 2.3333,
      "life": 1666.6666
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.3333,
            "band": 0.5,
            "height": [
              0.0,
              33.3333
            ],
            "colors": [
              [
                0.0,
                0.2195,
                1.0,
                1.0
              ],
              [
                0.0004,
                0.344,
                0.9406,
                1.0
              ],
              [
                0.0,
                0.1011,
                0.4704,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n8.effect"
        },
        "scale": 0.9,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 2.3333,
      "life": 1666.6666
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2143,
            "band": 0.5,
            "height": [
              -1.7143,
              0.0
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.0,
                0.2195,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n9.effect"
        },
        "scale": 7.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 2.3333,
      "offset": [
        0.0,
        0.0,
        2.0
      ],
      "life": 1666.6666
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2857,
            "band": 0.5,
            "height": [
              -2.0,
              0.0
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n10.effect"
        },
        "scale": 3.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 2.3333,
      "offset": [
        0.0,
        0.0,
        1.0
      ],
      "life": 1666.6666
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/blue_laser/n11.emitter"
        }
      },
      "at": 2.3333
    }
  ],
  "duration": 10.0,
  "looping": true
});
