import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "大リング": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 3 "大リング": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.9747,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.0367,
                0.0367,
                0.3045
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow2/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.1406,
              5.4748
            ],
            [
              0.2344,
              7.2536
            ],
            [
              0.3281,
              8.6739
            ],
            [
              0.4219,
              9.6894
            ],
            [
              0.5156,
              10.3254
            ],
            [
              0.6094,
              10.6531
            ],
            [
              1.0,
              11.0
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
      "at": 0.0,
      "rotate": [
        63.4349,
        37.7612,
        -50.7685
      ],
      "life": 0.5333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow2/n1-0.effect"
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
              3.3835
            ],
            [
              0.1562,
              2.8658
            ],
            [
              0.2344,
              2.4335
            ],
            [
              0.3281,
              2.0094
            ],
            [
              0.4688,
              1.5194
            ],
            [
              0.8281,
              0.5755
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          0.047,
          0.047,
          0.047,
          0.7843
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.5,
        0.0,
        -0.866
      ],
      "rotate": [
        63.4349,
        37.7612,
        -50.7685
      ],
      "velocity": [
        4.0,
        0.0,
        -6.9282
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow2/n2-0.emitter"
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
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7875,
            "band": 0.75,
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                0.0
              ],
              [
                1.0,
                0.0367,
                0.0367,
                0.3045
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow2/n3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.5
            ],
            [
              0.1406,
              5.4748
            ],
            [
              0.2344,
              7.2536
            ],
            [
              0.3281,
              8.6739
            ],
            [
              0.4219,
              9.6894
            ],
            [
              0.5156,
              10.3254
            ],
            [
              0.6094,
              10.6531
            ],
            [
              1.0,
              11.0
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
      "at": 0.0,
      "rotate": [
        63.4349,
        37.7612,
        -50.7685
      ],
      "life": 0.5333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow2/n4-0-0.emitter"
        }
      },
      "at": 0.0,
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
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.6657,
          0.6608
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
        0.6241,
        0.0,
        -0.0005
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
  "duration": 0.5333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
