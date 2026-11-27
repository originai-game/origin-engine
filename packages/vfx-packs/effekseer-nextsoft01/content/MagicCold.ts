import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicCold/n0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
        8.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        159.4439
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicCold/n1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.0,
        10.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        163.3007
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicCold/n4.emitter"
        }
      },
      "at": 0.8333
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.5326,
          1.0,
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
      "at": 0.8333,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.8333
    }
  ],
  "duration": 2.15,
  "events": [
    {
      "name": "impact",
      "time": 0.8333
    }
  ]
});
