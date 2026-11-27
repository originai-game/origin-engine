import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "縦軸": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.0.1 "横軸": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 1.0.0 "縦軸": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 1.0.1 "横軸": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 2 "リング斜め右上": ring 5 instances: the first 4 drawn
// effekseer: node 2 "リング斜め右上": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3 "リング斜め左上": ring 5 instances: the first 4 drawn
// effekseer: node 3 "リング斜め左上": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.873
            ],
            [
              0.0938,
              4.3617
            ],
            [
              0.2031,
              4.466
            ],
            [
              0.5,
              3.691
            ],
            [
              0.8906,
              1.7135
            ],
            [
              0.9688,
              0.9281
            ],
            [
              0.9844,
              0.6586
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          2.7968,
          0.0833,
          0.0833,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 1.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n0-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.873
            ],
            [
              0.0938,
              4.3617
            ],
            [
              0.2031,
              4.466
            ],
            [
              0.5,
              3.691
            ],
            [
              0.8906,
              1.7135
            ],
            [
              0.9688,
              0.9281
            ],
            [
              0.9844,
              0.6586
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          2.7968,
          0.0833,
          0.0833,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 1.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n1-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.873
            ],
            [
              0.0938,
              4.3617
            ],
            [
              0.2031,
              4.466
            ],
            [
              0.5,
              3.691
            ],
            [
              0.8906,
              1.7135
            ],
            [
              0.9688,
              0.9281
            ],
            [
              0.9844,
              0.6586
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          2.7968,
          0.0833,
          0.0833,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "life": 1.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n1-0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              3.873
            ],
            [
              0.0938,
              4.3617
            ],
            [
              0.2031,
              4.466
            ],
            [
              0.5,
              3.691
            ],
            [
              0.8906,
              1.7135
            ],
            [
              0.9688,
              0.9281
            ],
            [
              0.9844,
              0.6586
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          2.7968,
          0.0833,
          0.0833,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0667,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 1.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7501,
            "band": 0.45,
            "colors": [
              [
                1.0,
                0.006,
                0.006,
                0.0
              ],
              [
                1.0,
                0.0278,
                0.0278,
                1.0
              ],
              [
                1.0,
                0.006,
                0.006,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.0
            ],
            [
              0.0781,
              4.6591
            ],
            [
              0.1719,
              5.2795
            ],
            [
              0.2656,
              5.7373
            ],
            [
              0.375,
              6.0989
            ],
            [
              0.5781,
              6.4061
            ],
            [
              1.0,
              6.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          960.0
        ],
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "rotate": [
        -78.8298,
        -28.0243,
        -67.2041
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7501,
            "band": 0.45,
            "colors": [
              [
                1.0,
                0.006,
                0.006,
                0.0
              ],
              [
                1.0,
                0.0278,
                0.0278,
                1.0
              ],
              [
                1.0,
                0.006,
                0.006,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.0
            ],
            [
              0.0781,
              4.6591
            ],
            [
              0.1719,
              5.2795
            ],
            [
              0.2656,
              5.7373
            ],
            [
              0.375,
              6.0989
            ],
            [
              0.5781,
              6.4061
            ],
            [
              1.0,
              6.5
            ]
          ]
        },
        "spin": [
          0.0,
          0.0,
          -960.0
        ],
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.6667,
      "rotate": [
        -78.8298,
        28.0243,
        -112.7959
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword8/n4-0.emitter"
        }
      },
      "at": 0.6667,
      "offset": [
        2.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    }
  ],
  "duration": 1.4167,
  "events": [
    {
      "name": "impact",
      "time": 0.6667
    }
  ]
});
