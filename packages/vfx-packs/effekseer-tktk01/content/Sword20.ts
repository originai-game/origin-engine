import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "斬撃部分": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0 "色付け": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.1.0 "粒子": nested generation: 56 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.0 "粒子色付け": nested generation: 56 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1.0.1 "glow": nested generation: 56 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0 "粒子": nested generation: 56 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2.0.0 "粒子色付け": nested generation: 56 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -2.5,
        0.0
      ],
      "velocity": [
        0.0,
        14.4,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0-1-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -2.5,
        0.0
      ],
      "velocity": [
        0.0,
        14.4,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0-1-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -2.5,
        0.0
      ],
      "velocity": [
        0.0,
        14.4,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0-2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -1.5,
        0.0
      ],
      "velocity": [
        0.0,
        14.4,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword20/n0-0-0-2-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -1.5,
        0.0
      ],
      "velocity": [
        0.0,
        14.4,
        0.0
      ]
    }
  ],
  "duration": 2.8667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
