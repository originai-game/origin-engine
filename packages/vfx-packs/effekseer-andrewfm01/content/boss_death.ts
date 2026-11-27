import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Light Beams": motion out from a circle: approximated along one direction
// effekseer: node 0.5 "Ribbon Guide": attraction to the target
// effekseer: node 0.5.0 "Ribbon": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0 "Ribbon": nested generation: 8 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/boss_death/n0-0.emitter"
        }
      },
      "at": 2.4167,
      "until": 6.25
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
                0.2195,
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/boss_death/n0-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.2
            ],
            [
              0.25,
              1.41
            ],
            [
              0.375,
              2.765
            ],
            [
              0.5,
              4.62
            ],
            [
              0.625,
              6.975
            ],
            [
              0.75,
              9.83
            ],
            [
              0.875,
              13.185
            ],
            [
              1.0,
              17.04
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
      "life": 6.6667
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
                1.0,
                1.0,
                1.0,
                0.8901
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/boss_death/n0-2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.2
            ],
            [
              0.125,
              12.1477
            ],
            [
              0.25,
              37.2281
            ],
            [
              0.5,
              126.7875
            ],
            [
              0.625,
              191.2664
            ],
            [
              0.75,
              268.8781
            ],
            [
              0.875,
              359.6227
            ],
            [
              1.0,
              463.5
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
      "at": 3.5833,
      "life": 3.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/boss_death/n0-3.emitter"
        }
      },
      "at": 0.4167,
      "until": 6.25
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/boss_death/n0-4.emitter"
        }
      },
      "at": 2.4167,
      "until": 6.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/boss_death/n0-5-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.3333,
        "color": [
          0.0,
          0.2195,
          0.5356,
          1.0
        ]
      },
      "at": 1.0833,
      "life": 0.3333
    }
  ],
  "duration": 7.9167,
  "looping": true
});
