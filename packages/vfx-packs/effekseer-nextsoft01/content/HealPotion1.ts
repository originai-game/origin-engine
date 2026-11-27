import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion1/n0.emitter"
        }
      },
      "at": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion1/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-nextsoft01/HealPotion1/n1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.0,
        0.0
      ]
    }
  ],
  "duration": 1.2667,
  "events": [
    {
      "name": "impact",
      "time": 0.5
    }
  ]
});
