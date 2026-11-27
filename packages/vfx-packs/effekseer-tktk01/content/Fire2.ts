import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "溶岩本体": ring 10 instances: the first 4 drawn
// effekseer: node 1.0.0.0 "湯気単体": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 1.0.0.0 "湯気単体": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 2.0.0.0 "湯気単体": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 2.0.0.0 "湯気単体": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 0.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                1.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire2/n0-0.effect"
        },
        "scale": 1.0,
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.75
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire2/n1-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Fire2/n2-0-0-0.parent"
        }
      },
      "at": 0.0
    }
  ],
  "duration": 3.0133,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
