import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Heavy Line Render": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0 "Heavy Line Render": motion out from a circle: approximated along one direction
// effekseer: node 1.0 "Light Line Render": nested generation: 15 instances above its generator folded into one emitter
// effekseer: node 1.0 "Light Line Render": motion out from a circle: approximated along one direction
// effekseer: node 2.0 "Mid Line Render": nested generation: 4 instances above its generator folded into one emitter
// effekseer: node 2.0 "Mid Line Render": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_halo2/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        5.375,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_halo2/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/actionLines_halo2/n2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.625,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    }
  ],
  "duration": 1.6832,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
