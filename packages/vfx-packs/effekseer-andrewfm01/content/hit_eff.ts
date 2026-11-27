import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1 "Action Lines": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.2 "Motion Sparks": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.7
            ],
            [
              1.0,
              2.95
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
      "at": 0.05,
      "offset": [
        0.25,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -135.0
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.7
            ],
            [
              1.0,
              2.95
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
      "at": 0.0583,
      "offset": [
        0.25,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -45.0
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.7
            ],
            [
              1.0,
              2.95
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
      "at": 0.0667,
      "offset": [
        0.25,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        45.0
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.7
            ],
            [
              1.0,
              2.95
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
      "at": 0.075,
      "offset": [
        0.25,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        135.0
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6325
            ],
            [
              0.1875,
              0.7977
            ],
            [
              0.4219,
              0.9652
            ],
            [
              1.0,
              1.2884
            ]
          ]
        },
        "color": [
          1.0,
          0.0,
          0.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0333,
      "offset": [
        0.5,
        0.05,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -2.0
      ],
      "life": 0.15
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6325
            ],
            [
              0.1875,
              0.7977
            ],
            [
              0.4219,
              0.9652
            ],
            [
              1.0,
              1.2884
            ]
          ]
        },
        "color": [
          1.0,
          0.0,
          0.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0335,
      "offset": [
        0.5,
        0.05,
        0.0
      ],
      "life": 0.15
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.6325
            ],
            [
              0.1875,
              0.7977
            ],
            [
              0.4219,
              0.9652
            ],
            [
              1.0,
              1.2884
            ]
          ]
        },
        "color": [
          1.0,
          0.0,
          0.0,
          0.251
        ],
        "blend": "additive"
      },
      "at": 0.0337,
      "offset": [
        0.5,
        0.05,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        2.0
      ],
      "life": 0.15
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-2.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        0.0,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n0-3.emitter"
        }
      },
      "at": 0.1167
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.55,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0,
                0.2195,
                0.1255
              ],
              [
                1.0,
                1.0,
                1.0,
                0.3608
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.6
            ],
            [
              1.0,
              8.8
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
      "life": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/hit_eff/n2.emitter"
        }
      },
      "at": 0.0167
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
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 0.6215,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
