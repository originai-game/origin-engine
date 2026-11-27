import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1.0.0.0 "火メイン": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 2.0 "火メイン": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.9,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n0-0.effect"
        },
        "scale": 3.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.5,
            "band": 0.9,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n0-1.effect"
        },
        "scale": 3.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -180.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.125,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n0-2.effect"
        },
        "scale": 3.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 1.1111,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n0-3.effect"
        },
        "scale": 3.375,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 0.8333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n1-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n2-0.emitter"
        }
      },
      "at": 0.5
    }
  ],
  "duration": 2.425,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
