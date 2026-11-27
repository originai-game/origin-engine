import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 1 "Node": model 10 instances: the first 4 drawn
// effekseer: node 2 "Node": model 10 instances: the first 4 drawn
// effekseer: node 3.0.0.0 "Track": nested generation: 16 instances above its generator folded into one emitter
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-rock1"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n0.effect"
        },
        "scale": 1.5,
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.25,
      "life": 2.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n1.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n1.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.0167,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n1.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.0333,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block2"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n1.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.05,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n2.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n2.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.0167,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n2.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.0333,
      "life": 0.25
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/model-block3"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n2.effect"
        },
        "scale": 0.75,
        "color": [
          1.0,
          1.0,
          1.0,
          0.8784
        ],
        "blend": "alpha"
      },
      "at": 0.05,
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n3-0-0-0.effect"
        },
        "width": [
          4.0,
          4.0
        ],
        "lifetime": 0.0833,
        "color": [
          1.0,
          1.0,
          1.0,
          0.0
        ],
        "orbit": [
          4.0,
          15.0
        ]
      },
      "at": 0.0,
      "life": 0.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-magicalxspiral/Gohlem7/n4-0.emitter"
        }
      },
      "at": 0.0833
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
      "at": 0.25,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.25
    }
  ],
  "duration": 2.9167,
  "events": [
    {
      "name": "impact",
      "time": 0.25
    }
  ]
});
