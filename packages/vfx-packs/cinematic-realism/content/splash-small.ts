import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.2000000476837158,
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
      "life": 0.3499999940395355,
      "offset": [
        0,
        0.004999999888241291,
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
              0.03999999910593033
            ],
            [
              0.4000000059604645,
              0.12999999523162842
            ],
            [
              1,
              0.17000000178813934
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
              0.30000001192092896,
              0
            ],
            "inner": 0.800000011920929,
            "length": 360,
            "segments": 32,
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
        0.019999999552965164,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/plip-jet")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.029999999329447746,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/spray-small")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.05000000074505806,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/plip-mist")
      }
    },
    {
      "at": 0,
      "life": 1,
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
              0.05999999865889549
            ],
            [
              0.30000001192092896,
              0.20000000298023224
            ],
            [
              1,
              0.36000001430511475
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
                0.5
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
      "at": 0.20000000298023224,
      "life": 1,
      "offset": [
        0,
        0.004999999888241291,
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
              0.03999999910593033
            ],
            [
              0.4000000059604645,
              0.14000000059604645
            ],
            [
              1,
              0.23999999463558197
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
                0.4000000059604645
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
          1.399999976158142,
          1.7000000476837158
        ],
        "volume": 0.6000000238418579,
        "asset": ref("base/vfx/cinematic-realism/audio/splash")
      }
    }
  ]
});
