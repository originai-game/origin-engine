import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 3 "広がりリング": ring 5 instances: the first 4 drawn
// effekseer: node 4 "広がりリング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n2-0.emitter"
        }
      },
      "at": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n2-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
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
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
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
                0.1275,
                0.1304,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1406,
              4.1288
            ],
            [
              0.2344,
              5.3825
            ],
            [
              0.3281,
              6.3778
            ],
            [
              0.4219,
              7.087
            ],
            [
              0.5156,
              7.5304
            ],
            [
              0.6094,
              7.7586
            ],
            [
              1.0,
              8.0
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
      "at": 0.0,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8907,
            "band": 0.5,
            "colors": [
              [
                0.9574,
                0.1275,
                0.1275,
                0.0
              ],
              [
                1.0,
                0.1275,
                0.1275,
                0.0
              ],
              [
                0.9574,
                0.1275,
                0.1275,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n4.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              4.0
            ],
            [
              0.0938,
              5.2804
            ],
            [
              0.1875,
              6.2709
            ],
            [
              0.2812,
              7.0031
            ],
            [
              0.3906,
              7.5786
            ],
            [
              0.4844,
              7.8771
            ],
            [
              0.5938,
              8.0553
            ],
            [
              1.0,
              8.2
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
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow4/n5-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -90.0
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
      "at": 0.0,
      "life": 0.3,
      "offset": [
        0.0524,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 0.8448,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
