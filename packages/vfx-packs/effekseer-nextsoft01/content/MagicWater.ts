import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 9 "水発生": a sound
// effekseer: node 9.0 "水音": a sound
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n0.emitter"
        }
      },
      "at": 0.4667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n1.emitter"
        }
      },
      "at": 0.0333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n2.emitter"
        }
      },
      "at": 0.4667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n3.emitter"
        }
      },
      "at": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n4.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n5.emitter"
        }
      },
      "at": 0.3667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n6.emitter"
        }
      },
      "at": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n7.emitter"
        }
      },
      "at": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/MagicWater/n8.emitter"
        }
      },
      "at": 0.4167
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
      "at": 0.3667,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.3667
    }
  ],
  "duration": 2.09,
  "events": [
    {
      "name": "impact",
      "time": 0.3667
    }
  ]
});
