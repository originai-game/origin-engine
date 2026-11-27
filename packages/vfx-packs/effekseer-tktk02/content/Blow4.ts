import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.2.0.0 "RIng": ring 10 instances: the first 4 drawn
// effekseer: node 0.2.0.0 "RIng": a ring whose thickness changes over its life: its middle thickness kept
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.8516,
            "band": 0.825,
            "start": 135.0,
            "length": 270.0,
            "colors": [
              [
                1.0,
                0.1263,
                0.0275,
                0.0
              ],
              [
                1.0,
                0.7742,
                0.5767,
                0.7786
              ],
              [
                1.0,
                0.1263,
                0.0275,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-2-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5
            ],
            [
              0.0781,
              1.8955
            ],
            [
              0.1719,
              2.2677
            ],
            [
              0.2656,
              2.5424
            ],
            [
              0.375,
              2.7593
            ],
            [
              0.5781,
              2.9437
            ],
            [
              1.0,
              3.0
            ]
          ]
        },
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
          "$ogRef": "base/vfx/effekseer-tktk02/Blow4/n0-3.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9533,
          0.2842
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
      "life": 0.3,
      "offset": [
        0.0935,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 1.0001,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
