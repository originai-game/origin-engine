import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "火尻尾": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0 "火尻尾": nested generation: 16 instances above its generator folded into one emitter
// effekseer: node 0.1 "火メイン": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire1/n0-0.emitter"
        }
      },
      "at": 0.0,
      "velocity": [
        0.0,
        0.0,
        11.4
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire1/n0-0-0-0.parent"
        }
      },
      "at": 0.0,
      "velocity": [
        0.0,
        0.0,
        11.4
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire1/n0-0-0-0-0.parent"
        }
      },
      "at": 0.0,
      "velocity": [
        0.0,
        0.0,
        11.4
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire1/n0-1.emitter"
        }
      },
      "at": 0.0,
      "velocity": [
        0.0,
        0.0,
        11.4
      ]
    }
  ],
  "duration": 4.25,
  "looping": true
});
