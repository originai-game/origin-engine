import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "Node": nested generation: 400 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/electric_dust/n0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/electric_dust/n0-0-0.effect"
        },
        "width": [
          0.25,
          0.0625
        ],
        "lifetime": 0.1333,
        "color": [
          1.0,
          0.3562,
          0.047,
          1.0
        ]
      },
      "at": 0.0,
      "life": 0.1333
    }
  ],
  "duration": 3.3998,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
