import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 2 "衝撃波": a sprite scaled unevenly: drawn square at its mean
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion2/n0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion2/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.0,
        0.0
      ],
      "velocity": [
        0.0,
        -5.25,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion2/n1.emitter"
        }
      },
      "at": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion2/n2.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              0.0
            ],
            [
              0.2656,
              0.9152
            ],
            [
              0.5,
              2.1494
            ],
            [
              0.75,
              3.9497
            ],
            [
              1.0,
              6.2498
            ]
          ]
        },
        "color": [
          0.586,
          0.7969,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.5833,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ],
      "life": 0.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion2/n3.emitter"
        }
      },
      "at": 0.6667
    }
  ],
  "duration": 1.63,
  "events": [
    {
      "name": "impact",
      "time": 0.5833
    }
  ]
});
