import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicThunder/n0.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicThunder/n1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        0.0,
        10.0,
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
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicThunder/n2.emitter"
        }
      },
      "at": 0.2,
      "offset": [
        0.0,
        0.0,
        -0.5
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicThunder/n3.emitter"
        }
      },
      "at": 0.4333,
      "offset": [
        -0.3,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicThunder/n4.emitter"
        }
      },
      "at": 0.2667,
      "offset": [
        0.5,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicThunder/n5.emitter"
        }
      },
      "at": 0.0167
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.0478,
          0.5326,
          1.0
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
      "at": 0.1667,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.1667
    }
  ],
  "duration": 1.05,
  "events": [
    {
      "name": "impact",
      "time": 0.1667
    }
  ]
});
