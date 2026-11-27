import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2.5999999046325684,
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
        1,
        0,
        3
      ],
      "rotate": [
        -90,
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n0-0-0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        1,
        0,
        3
      ],
      "rotate": [
        -90,
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n0-0-0-1.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        1,
        0,
        3
      ],
      "rotate": [
        -90,
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n0-0-0-2.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.833299994468689,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          2,
          2,
          2,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Cure7/n1-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              0.09380000084638596,
              1.7445000410079956
            ],
            [
              0.20309999585151672,
              2.4732000827789307
            ],
            [
              0.3125,
              3.069499969482422
            ],
            [
              0.4219000041484833,
              3.5527000427246094
            ],
            [
              0.6561999917030334,
              4.296000003814697
            ],
            [
              1,
              5
            ]
          ]
        },
        "shape": {
          "kind": "quad"
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0.833299994468689,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          3,
          3,
          3,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Cure7/n1-1.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              0.07810000330209732,
              1.2635999917984009
            ],
            [
              0.17190000414848328,
              1.5118000507354736
            ],
            [
              0.2655999958515167,
              1.6949000358581543
            ],
            [
              0.375,
              1.8394999504089355
            ],
            [
              0.5781000256538391,
              1.962499976158142
            ],
            [
              1,
              2
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 1,
            "colors": [
              [
                1,
                1,
                1,
                0
              ],
              [
                1,
                1,
                1,
                0.800000011920929
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
      "at": 0.41670000553131104,
      "life": 0.833299994468689,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Cure7/n1-2.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              0.07810000330209732,
              1.2635999917984009
            ],
            [
              0.17190000414848328,
              1.5118000507354736
            ],
            [
              0.2655999958515167,
              1.6949000358581543
            ],
            [
              0.375,
              1.8394999504089355
            ],
            [
              0.5781000256538391,
              1.962499976158142
            ],
            [
              1,
              2
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 1,
            "colors": [
              [
                1,
                1,
                0.5860000252723694,
                0
              ],
              [
                1,
                1,
                0.41510000824928284,
                0.800000011920929
              ],
              [
                1,
                1,
                0.5860000252723694,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.8845000267028809,
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
      "at": 0.41670000553131104,
      "life": 0.666700005531311,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Cure7/n1-3.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              2
            ],
            [
              0.20309999585151672,
              1.8158999681472778
            ],
            [
              0.4219000041484833,
              1.680899977684021
            ],
            [
              1,
              1.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0,
            "colors": [
              [
                1,
                1,
                1,
                0
              ],
              [
                1,
                1,
                0.934499979019165,
                0.14059999585151672
              ],
              [
                1,
                1,
                1,
                0
              ]
            ],
            "height": [
              1.8236000537872314,
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
    },
    {
      "at": 0.41670000553131104,
      "life": 0.666700005531311,
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          0.5860000252723694,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Cure7/n1-3-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.09380000084638596,
              0.6514000296592712
            ],
            [
              0.20309999585151672,
              1.2890000343322754
            ],
            [
              0.3125,
              1.8107999563217163
            ],
            [
              0.4219000041484833,
              2.233599901199341
            ],
            [
              0.5311999917030334,
              2.5741000175476074
            ],
            [
              0.6561999917030334,
              2.884000062942505
            ],
            [
              1,
              3.5
            ]
          ]
        },
        "shape": {
          "kind": "quad"
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-0.parent")
      }
    },
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-0-0.parent")
      }
    },
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-1.parent")
      }
    },
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-1-0.parent")
      }
    },
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-2.parent")
      }
    },
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
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-0-2-0.parent")
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0,
      "offset": [
        0,
        0,
        0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-1-0.emitter")
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0,
      "offset": [
        0,
        0,
        0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-1-0-0.emitter")
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0,
      "offset": [
        0,
        0,
        0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-1-1.emitter")
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0,
      "offset": [
        0,
        0,
        0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-1-1-0.emitter")
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0,
      "offset": [
        0,
        0,
        0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-1-2.emitter")
      }
    },
    {
      "at": 0.41670000553131104,
      "life": 0,
      "offset": [
        0,
        0,
        0.5
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Cure7/n1-4-1-2-0.emitter")
      }
    }
  ]
});
