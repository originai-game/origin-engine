import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal1/n0-0-0.effect"
        },
        "width": [
          1.4142,
          0.3536
        ],
        "lifetime": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          1.0,
          1.6667
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.2,
        0.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ],
      "life": 0.8
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal1/n0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        1.0,
        -0.2,
        0.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal1/n0-1-0.effect"
        },
        "width": [
          1.4142,
          0.3536
        ],
        "lifetime": 1.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          1.0,
          1.6667
        ]
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.2,
        0.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ],
      "life": 0.8
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicHeal1/n0-1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -1.0,
        -0.2,
        0.0
      ],
      "velocity": [
        0.0,
        3.6,
        0.0
      ]
    }
  ],
  "duration": 1.9667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
