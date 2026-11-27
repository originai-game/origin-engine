import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "Node": ring 6 instances: the first 4 drawn
// effekseer: node 0 "Node": a ring whose height changes over its life: its middle height kept
// effekseer: node 0 "Node": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5265,
            "band": 0.0,
            "height": [
              1.8956,
              0.0
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
                0.1275,
                0.017,
                1.0
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
          "$ogRef": "base/vfx/effekseer-nextsoft01/PowerUp/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1562,
              1.7164
            ],
            [
              0.3281,
              1.5023
            ],
            [
              1.0,
              1.0
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
      "rotate": [
        -90.0,
        0.0,
        -135.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5265,
            "band": 0.0,
            "height": [
              1.8956,
              0.0
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
                0.1275,
                0.017,
                1.0
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
          "$ogRef": "base/vfx/effekseer-nextsoft01/PowerUp/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1562,
              1.7164
            ],
            [
              0.3281,
              1.5023
            ],
            [
              1.0,
              1.0
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
      "at": 0.25,
      "rotate": [
        -90.0,
        0.0,
        -45.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5265,
            "band": 0.0,
            "height": [
              1.8956,
              0.0
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
                0.1275,
                0.017,
                1.0
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
          "$ogRef": "base/vfx/effekseer-nextsoft01/PowerUp/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1562,
              1.7164
            ],
            [
              0.3281,
              1.5023
            ],
            [
              1.0,
              1.0
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
      "at": 0.5,
      "rotate": [
        -90.0,
        0.0,
        45.0
      ],
      "life": 1.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5265,
            "band": 0.0,
            "height": [
              1.8956,
              0.0
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
                0.1275,
                0.017,
                1.0
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
          "$ogRef": "base/vfx/effekseer-nextsoft01/PowerUp/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.1562,
              1.7164
            ],
            [
              0.3281,
              1.5023
            ],
            [
              1.0,
              1.0
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
      "at": 0.75,
      "rotate": [
        -90.0,
        0.0,
        135.0
      ],
      "life": 1.0
    }
  ],
  "duration": 1.75,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
