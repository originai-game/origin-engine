import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.600000023841858,
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
      "life": 0.550000011920929,
      "offset": [
        0,
        0.009999999776482582,
        0
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
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/crown"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.11999999731779099
            ],
            [
              0.4000000059604645,
              0.41999998688697815
            ],
            [
              1,
              0.550000011920929
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
                1
              ],
              [
                1,
                1,
                1,
                1
              ],
              [
                1,
                1,
                1,
                1
              ]
            ],
            "height": [
              0.3199999928474426,
              0
            ],
            "inner": 0.8199999928474426,
            "length": 360,
            "segments": 48,
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
      "life": 0,
      "offset": [
        0,
        0.07999999821186066,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/spray")
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
        "asset": ref("base/vfx/cinematic-realism/emitters/column")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.014999999664723873,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/foam")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.10000000149011612,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/mist")
      }
    },
    {
      "at": 0,
      "life": 1.2000000476837158,
      "offset": [
        0,
        0.00800000037997961,
        0
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
        "kind": "mesh",
        "blend": "alpha",
        "color": [
          0.800000011920929,
          0.8799999952316284,
          0.949999988079071,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/water"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.20000000298023224
            ],
            [
              0.30000001192092896,
              0.5
            ],
            [
              1,
              0.800000011920929
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
                1,
                1,
                1,
                0.550000011920929
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
            "inner": 0.8600000143051147,
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
      "at": 0.25,
      "life": 1.2000000476837158,
      "offset": [
        0,
        0.006000000052154064,
        0
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
        "kind": "mesh",
        "blend": "alpha",
        "color": [
          0.800000011920929,
          0.8799999952316284,
          0.949999988079071,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/water"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.10000000149011612
            ],
            [
              0.4000000059604645,
              0.3499999940395355
            ],
            [
              1,
              0.550000011920929
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
                1,
                1,
                1,
                0.44999998807907104
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
            "inner": 0.8799999952316284,
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
        "kind": "sound",
        "pitch": [
          0.8999999761581421,
          1.100000023841858
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/splash")
      }
    }
  ]
});
