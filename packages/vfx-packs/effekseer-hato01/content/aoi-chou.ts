import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "Node": local force fields
// effekseer: node 0.0.0.0.0 "Node": model unlimited instances: the first 4 drawn
// effekseer: node 0.0.0.0.0 "Node": a model that is not an .efkmodel (fbx/hane2.fbx): drawn as a sphere
// effekseer: node 0.1.0.0.0 "Node": model unlimited instances: the first 4 drawn
// effekseer: node 0.1.0.0.0 "Node": a model that is not an .efkmodel (fbx/hane2.fbx): drawn as a sphere
// effekseer: node 0.1.0.0.1 "Node": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.1.0.0.1.0 "Node": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.1 "Node": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.0.1.0 "Node": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-0-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-0-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 4.1667,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-0-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 8.3333,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-0-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 12.5,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 4.1667,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 8.3333,
      "life": 20.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-0.effect"
        },
        "scale": 4.3333,
        "color": [
          0.0026,
          0.212,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 12.5,
      "life": 20.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-1.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-1-0.parent"
        }
      },
      "at": 1.6667,
      "offset": [
        0.0,
        2.0,
        0.0
      ],
      "rotate": [
        -152.4,
        -0.0001,
        -36.2
      ],
      "until": 20.0
    }
  ],
  "duration": 20.0,
  "looping": true
});
