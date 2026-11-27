import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 13.005999565124512,
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
      "at": 0.08330000191926956,
      "life": 0.44999998807907104,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        100.31410217285156,
        17.229400634765625,
        -141.5666961669922
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
        "blend": "alpha",
        "color": [
          1,
          1,
          1,
          0.800000011920929
        ],
        "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.125
            ],
            [
              1,
              11.25
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
                0.1881999969482422
              ],
              [
                1,
                1,
                1,
                0.7529000043869019
              ],
              [
                1,
                1,
                1,
                0.1881999969482422
              ]
            ],
            "height": [
              -0.25,
              0.25
            ],
            "inner": 1,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          360
        ]
      }
    },
    {
      "at": 0.08879999816417694,
      "life": 0.44999998807907104,
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
        "blend": "alpha",
        "color": [
          1,
          1,
          1,
          0.800000011920929
        ],
        "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.125
            ],
            [
              1,
              11.25
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
                0.1881999969482422
              ],
              [
                1,
                1,
                1,
                0.7529000043869019
              ],
              [
                1,
                1,
                1,
                0.1881999969482422
              ]
            ],
            "height": [
              -0.25,
              0.25
            ],
            "inner": 1,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          360
        ]
      }
    },
    {
      "at": 0.09430000185966492,
      "life": 0.44999998807907104,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        79.68589782714844,
        17.229400634765625,
        141.5666961669922
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
        "blend": "alpha",
        "color": [
          1,
          1,
          1,
          0.800000011920929
        ],
        "effect": ref("base/vfx/effekseer-starter/FeatherBomb/n0-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.125
            ],
            [
              1,
              11.25
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
                0.1881999969482422
              ],
              [
                1,
                1,
                1,
                0.7529000043869019
              ],
              [
                1,
                1,
                1,
                0.1881999969482422
              ]
            ],
            "height": [
              -0.25,
              0.25
            ],
            "inner": 1,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          360
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-0-0.parent")
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-1-0.parent")
      }
    },
    {
      "at": 0.3499999940395355,
      "life": 0,
      "offset": [
        0,
        0,
        1.125
      ],
      "rotate": [
        14.036199569702148,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-0-0.parent")
      }
    },
    {
      "at": 0.3499999940395355,
      "life": 0,
      "offset": [
        0,
        0,
        1.125
      ],
      "rotate": [
        11.534600257873535,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-1.emitter")
      }
    },
    {
      "at": 0.48330000042915344,
      "life": 0,
      "offset": [
        0,
        0,
        1.125
      ],
      "rotate": [
        11.534600257873535,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-2.emitter")
      }
    },
    {
      "at": 0.6166999936103821,
      "life": 0,
      "offset": [
        0,
        0,
        1.125
      ],
      "rotate": [
        11.534600257873535,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-3.emitter")
      }
    },
    {
      "at": 0.75,
      "life": 0,
      "offset": [
        0,
        0,
        1.125
      ],
      "rotate": [
        11.534600257873535,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-2-4.emitter")
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-0.parent")
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-1.parent")
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-2.parent")
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-3.parent")
      }
    },
    {
      "at": 0.604200005531311,
      "life": 0,
      "offset": [
        0,
        2.0434000492095947,
        2.9182000160217285
      ],
      "rotate": [
        55,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-4.emitter")
      }
    },
    {
      "at": 0.704200029373169,
      "life": 0,
      "offset": [
        0,
        2.0434000492095947,
        2.9182000160217285
      ],
      "rotate": [
        55,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-3-0-5.emitter")
      }
    },
    {
      "at": 0.516700029373169,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-4-0-0.parent")
      }
    },
    {
      "at": 0.516700029373169,
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-4-0-1.parent")
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
        "asset": ref("base/vfx/effekseer-starter/FeatherBomb/n0-2.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.30000001192092896,
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
        "kind": "light",
        "color": [
          1,
          1,
          1
        ],
        "distance": 12,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              8
            ],
            [
              1,
              0
            ]
          ]
        }
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
        "kind": "shake",
        "trauma": 0.4000000059604645
      }
    }
  ]
});
