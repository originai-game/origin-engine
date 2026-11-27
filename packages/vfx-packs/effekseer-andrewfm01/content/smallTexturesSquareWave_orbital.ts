import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0 "Ring Outer": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.0 "Ring Outer": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.0.0.1 "Ring Center": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.1 "Ring Center": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.0.0.2 "Ring Inner": ring 10 instances: the first 4 drawn
// effekseer: node 0.0.0.2 "Ring Inner": nested generation: 12 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0,
                1.0,
                0.0392
              ],
              [
                1.0,
                0.0,
                1.0,
                0.0392
              ],
              [
                1.0,
                0.0,
                0.0,
                0.0392
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital/n0-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 66.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.7143,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.2195,
                0.0478,
                0.251
              ],
              [
                1.0,
                0.2195,
                0.0478,
                0.251
              ],
              [
                0.9658,
                0.6187,
                0.2508,
                0.251
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital/n0-0-0-1.effect"
        },
        "scale": 1.4,
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 66.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.125,
            "band": 0.5,
            "colors": [
              [
                1.0,
                0.0,
                0.0,
                1.0
              ],
              [
                1.0,
                0.2195,
                0.0478,
                1.0
              ],
              [
                0.9658,
                0.6187,
                0.2508,
                1.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital/n0-0-0-2.effect"
        },
        "scale": 0.8,
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 66.6667
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
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
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 66.6667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
