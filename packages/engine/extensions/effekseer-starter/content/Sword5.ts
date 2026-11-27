import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.6000000238418579,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.10000000149011612
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        -9,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        50,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          5,
          5,
          5,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Sword5/n0-0.effect"),
        "faceCamera": false,
        "scale": 0.5476999878883362,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 1,
            "colors": [
              [
                0.1274999976158142,
                1,
                1,
                0
              ],
              [
                0.1274999976158142,
                1,
                1,
                0.07840000092983246
              ],
              [
                0.1274999976158142,
                1,
                1,
                0.5881999731063843
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.06669999659061432,
      "life": 0,
      "offset": [
        0.5,
        0,
        0
      ],
      "rotate": [
        -163,
        0,
        -90
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Sword5/n1-0-0.emitter")
      }
    },
    {
      "at": 0.10000000149011612,
      "life": 0.5,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        107,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          4,
          4,
          4,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Sword5/n2.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              2
            ],
            [
              0.10939999669790268,
              3.731800079345703
            ],
            [
              0.21879999339580536,
              5.191400051116943
            ],
            [
              0.3280999958515167,
              6.404799938201904
            ],
            [
              0.4530999958515167,
              7.523399829864502
            ],
            [
              0.5781000256538391,
              8.392900466918945
            ],
            [
              0.7031000256538391,
              9.051899909973145
            ],
            [
              1,
              10
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                1,
                0
              ],
              [
                0.1274999976158142,
                1,
                1,
                1
              ],
              [
                1,
                1,
                1,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.5,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    }
  ]
});
