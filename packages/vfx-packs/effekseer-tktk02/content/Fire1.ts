import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "Fire": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.0.0.0.1 "Fire": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.1.0.0.0 "Fire": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.1.0.0.1 "Fire": a child of an instanced parent: its motion drawn up the world's y
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-0-0-0-0.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-0-0-0-1.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -0.5,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-1-0-0-0.parent"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        0.0,
        3.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-1-0-0-1.parent"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        0.0,
        3.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-1-1.emitter"
        }
      },
      "at": 0.4667,
      "offset": [
        0.0,
        -0.5,
        3.0
      ],
      "rotate": [
        -90.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 2.9667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
