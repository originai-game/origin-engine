import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "地面リングカラー": ring 10 instances: the first 4 drawn
// effekseer: node 0.1 "地面柱カラー": ring 10 instances: the first 4 drawn
// effekseer: node 0.1 "地面柱カラー": a ring whose height changes over its life: its middle height kept
// effekseer: node 0.2 "地面リング内側": ring 10 instances: the first 4 drawn
// effekseer: node 1.1 "地面リング白内側": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 1.2 "リング白柱": a ring whose height changes over its life: its middle height kept
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light3/n0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 1.0,
            "height": [
              0.0,
              1.0588
            ],
            "colors": [
              [
                1.0,
                0.9914,
                1.0,
                0.0
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light3/n0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light3/n0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9695,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light3/n1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
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
      "at": 0.0,
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.2605,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light3/n1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
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
      "at": 0.0,
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9695,
            "band": 1.0,
            "height": [
              0.0,
              1.0473
            ],
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
          "$ogRef": "base/vfx/effekseer-tktk01/Light3/n1-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0938,
              1.7445
            ],
            [
              0.2031,
              2.4732
            ],
            [
              0.3125,
              3.0695
            ],
            [
              0.4219,
              3.5527
            ],
            [
              0.6562,
              4.296
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
      "at": 0.0,
      "life": 1.0
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
  "duration": 1.0,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
