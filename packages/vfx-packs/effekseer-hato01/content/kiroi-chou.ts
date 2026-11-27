import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Node": local force fields
// effekseer: node 0.0.0.0.0.0 "Node": model 6 instances: the first 4 drawn
// effekseer: node 0.0.0.0.0.0 "Node": a model that is not an .efkmodel (fbx/hane2.fbx): drawn as a sphere
// effekseer: node 0.0.1.0.0.0 "Node": model 6 instances: the first 4 drawn
// effekseer: node 0.0.1.0.0.0 "Node": a model that is not an .efkmodel (fbx/hane2.fbx): drawn as a sphere
// effekseer: node 0.0.1.0.0.1 "Node": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-0-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.673,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-0-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.673,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.675,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-0-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.673,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 3.35,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-0-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.673,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 5.025,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-1-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.7667,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-1-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.7667,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 1.675,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-1-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.7667,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 3.35,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-1-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.7667,
          1.0,
          0.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 5.025,
      "velocity": [
        0.0,
        0.2418,
        0.0
      ],
      "life": 6.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-1-0-0-1.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        4.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    }
  ],
  "duration": 33.375,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
