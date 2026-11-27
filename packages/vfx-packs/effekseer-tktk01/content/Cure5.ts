import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "回復中心もや": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1 "回復中心もや": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.0.0 "周囲の回る線": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0 "周囲の回る線": nested generation: 8 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 220.0,
            "length": 100.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n1-0-0.effect"
        },
        "scale": 75.0,
        "spin": [
          0.0,
          0.0,
          300.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        0.0,
        -2.357,
        2.357
      ],
      "rotate": [
        27.5,
        0.0,
        90.0
      ],
      "velocity": [
        0.0,
        5.3033,
        -5.3033
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 220.0,
            "length": 100.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n1-0-0.effect"
        },
        "scale": 75.0,
        "spin": [
          0.0,
          0.0,
          300.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "offset": [
        0.0,
        -2.357,
        2.357
      ],
      "rotate": [
        32.5,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        5.3033,
        -5.3033
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 220.0,
            "length": 100.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n1-0-0.effect"
        },
        "scale": 75.0,
        "spin": [
          0.0,
          0.0,
          300.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "offset": [
        0.0,
        -2.357,
        2.357
      ],
      "rotate": [
        37.5,
        0.0,
        90.0
      ],
      "velocity": [
        0.0,
        5.3033,
        -5.3033
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "start": 220.0,
            "length": 100.0,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n1-0-0.effect"
        },
        "scale": 75.0,
        "spin": [
          0.0,
          0.0,
          300.0
        ],
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.005,
      "offset": [
        0.0,
        -2.357,
        2.357
      ],
      "rotate": [
        42.5,
        0.0,
        -90.0
      ],
      "velocity": [
        0.0,
        5.3033,
        -5.3033
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -10.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n3-0.parent"
        }
      },
      "at": 0.0
    }
  ],
  "duration": 1.4333,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
