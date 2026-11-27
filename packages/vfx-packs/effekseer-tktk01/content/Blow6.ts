import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "衝撃つぶ": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow6/n0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.4045,
          0.1701
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
        "distance": 4.0
      },
      "at": 0.0,
      "life": 0.3
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.2
      },
      "at": 0.0
    }
  ],
  "duration": 2.3667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
