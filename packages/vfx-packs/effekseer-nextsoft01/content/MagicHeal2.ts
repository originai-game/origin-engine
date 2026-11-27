import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal2/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal2/n0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
        -1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal2/n1.emitter"
        }
      },
      "at": 1.0,
      "offset": [
        0.0,
        4.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.48,
            "band": 0.5,
            "height": [
              0.4,
              -0.1
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
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal2/n2.effect"
        },
        "scale": 5.0,
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
      "life": 4.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal2/n3.effect"
        },
        "scale": 6.0,
        "spin": [
          0.0,
          0.0,
          -240.0
        ],
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
      "life": 4.0
    }
  ],
  "duration": 4.0,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
