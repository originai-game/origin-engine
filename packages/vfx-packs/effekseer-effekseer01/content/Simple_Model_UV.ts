import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0 "Arrow": a model that is not an .efkmodel (Model/Arrow.fbx): drawn as a sphere
// effekseer: node 1 "Arrow": a model that is not an .efkmodel (Model/Arrow.fbx): drawn as a sphere
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Model_UV/n0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        4.0,
        0.0
      ],
      "life": 3.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "sphere"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-effekseer01/Simple_Model_UV/n1.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.5,
      "offset": [
        0.0,
        4.0,
        0.0
      ],
      "life": 3.0
    }
  ],
  "duration": 3.5,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
