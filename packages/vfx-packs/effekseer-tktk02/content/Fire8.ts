import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "Fire": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.0 "ｉｎｔｅｒｖａｌ": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1 "Glow": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-0-0-0-0.parent"
        }
      },
      "at": 0.0,
      "rotate": [
        -90.0,
        0.0,
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-0-1.emitter"
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
        -144.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-1-0-0-0.emitter"
        }
      },
      "at": 0.5,
      "rotate": [
        -178.2671,
        -26.6536,
        171.4417
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-1-0-0-1.emitter"
        }
      },
      "at": 0.5,
      "rotate": [
        -178.2671,
        -26.6536,
        171.4417
      ],
      "until": 1.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-1-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              2.0
            ],
            [
              0.2188,
              4.297
            ],
            [
              0.4531,
              6.3585
            ],
            [
              0.7031,
              8.1893
            ],
            [
              1.0,
              10.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          0.502
        ],
        "blend": "additive"
      },
      "at": 0.5,
      "offset": [
        0.0,
        -0.5,
        0.0
      ],
      "rotate": [
        -90.0,
        0.0,
        -144.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-1-2-0-0-0.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        -0.25,
        0.0
      ],
      "rotate": [
        180.0,
        -36.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-1-2-0-0-1.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        -0.25,
        0.0
      ],
      "rotate": [
        180.0,
        -36.0,
        180.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Fire8/n0-1-2-0-0-2.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        -0.25,
        0.0
      ],
      "rotate": [
        180.0,
        -36.0,
        180.0
      ]
    }
  ],
  "duration": 2.5667,
  "looping": true
});
