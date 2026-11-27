import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6667,
            "band": 0.5,
            "height": [
              -0.3333,
              0.3333
            ],
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                0.0478,
                0.1166,
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
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Snowman1/n0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              1.5
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
      "life": 0.3333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.25,
            "band": 0.5,
            "height": [
              -0.25,
              0.25
            ],
            "colors": [
              [
                0.0,
                0.0,
                1.0,
                0.0
              ],
              [
                0.2195,
                1.0,
                1.0,
                0.502
              ],
              [
                0.0,
                0.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Snowman1/n1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              1.0,
              1.5
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
      "at": 0.0833,
      "life": 0.3333
    }
  ],
  "duration": 0.4167,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
