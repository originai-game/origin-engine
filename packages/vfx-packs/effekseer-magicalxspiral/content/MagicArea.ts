import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 6.0 "Node": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n0.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          0.0677,
          1.0,
          0.7529
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 2.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.2,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n2.effect"
        },
        "scale": 2.0,
        "color": [
          0.5356,
          0.0582,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        90.0,
        0.0,
        0.0
      ],
      "life": 2.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.1,
            "height": [
              2.8571,
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
                0.5356,
                0.0478,
                1.0,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n3.effect"
        },
        "scale": 0.7,
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
        0.0
      ],
      "life": 2.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.1,
            "height": [
              2.8571,
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
                0.5356,
                0.0478,
                1.0,
                0.502
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n4.effect"
        },
        "scale": 0.7,
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
        120.0
      ],
      "life": 2.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.0,
            "band": 0.5,
            "height": [
              6.6667,
              -0.3333
            ],
            "colors": [
              [
                0.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                0.2195,
                1.0,
                0.502
              ],
              [
                1.0,
                0.0,
                1.0,
                0.502
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n5.effect"
        },
        "scale": 0.6,
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
        0.0
      ],
      "life": 2.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n6-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        -3.4698,
        -13.609,
        -14.4504
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/MagicArea/n7.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    }
  ],
  "duration": 2.9167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
