import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "Lines": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.5,
            "colors": [
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_radial/n0.effect"
        },
        "scale": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_radial/n1.emitter"
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
                1.0,
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
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_radial/n2.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.6667,
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
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_radial/n3.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 1.6667
    }
  ],
  "duration": 1.6733,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
