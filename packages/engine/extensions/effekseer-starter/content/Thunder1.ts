import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2.1500000953674316,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Thunder1/n0.emitter")
      }
    },
    {
      "at": 0,
      "life": 1.666700005531311,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        90,
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
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Thunder1/n1.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.5
            ],
            [
              1,
              1.75
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
                0,
                0
              ],
              [
                1,
                0.7519000172615051,
                0,
                0.3765000104904175
              ],
              [
                1,
                1,
                0,
                0
              ]
            ],
            "height": [
              0,
              -0.10000000149011612
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
      "at": 0,
      "life": 1.333299994468689,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        90,
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
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Thunder1/n2.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              0.09380000084638596,
              1.05840003490448
            ],
            [
              0.20309999585151672,
              1.6049000024795532
            ],
            [
              0.3125,
              2.0520999431610107
            ],
            [
              0.4219000041484833,
              2.4144999980926514
            ],
            [
              0.5311999917030334,
              2.706399917602539
            ],
            [
              0.6561999917030334,
              2.9719998836517334
            ],
            [
              1,
              3.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                0,
                0,
                0,
                0
              ],
              [
                1,
                0.7519000172615051,
                0,
                0.8784000277519226
              ],
              [
                0,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              -0.009999999776482582
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
          300
        ]
      }
    },
    {
      "at": 0.11670000106096268,
      "life": 1.333299994468689,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        90,
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
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Thunder1/n3.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              1,
              3.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                0,
                0,
                0,
                0
              ],
              [
                1,
                0.7519000172615051,
                0,
                0.8784000277519226
              ],
              [
                0,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              -0.009999999776482582
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
      "at": 0.23330000042915344,
      "life": 1.333299994468689,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        90,
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
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Thunder1/n4.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              0.09380000084638596,
              1.05840003490448
            ],
            [
              0.20309999585151672,
              1.6049000024795532
            ],
            [
              0.3125,
              2.0520999431610107
            ],
            [
              0.4219000041484833,
              2.4144999980926514
            ],
            [
              0.5311999917030334,
              2.706399917602539
            ],
            [
              0.6561999917030334,
              2.9719998836517334
            ],
            [
              1,
              3.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                0,
                0,
                0,
                0
              ],
              [
                1,
                0.7519000172615051,
                0,
                0.8784000277519226
              ],
              [
                0,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              -0.009999999776482582
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
          -300
        ]
      }
    }
  ]
});
