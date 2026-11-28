import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2.200000047683716,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.20000000298023224
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 1.2999999523162842,
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
        "kind": "beam",
        "color": [
          1.600000023841858,
          1.2999999523162842,
          0.6000000238418579,
          0.5
        ],
        "effect": ref("base/vfx/cinematic-realism/beam-soft"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          4.5,
          0
        ],
        "width": 0.8999999761581421
      }
    },
    {
      "at": 0,
      "life": 1.2999999523162842,
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
        "kind": "beam",
        "color": [
          2.200000047683716,
          1.7999999523162842,
          1,
          0.699999988079071
        ],
        "effect": ref("base/vfx/cinematic-realism/beam-soft"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          6,
          0
        ],
        "width": 0.25
      }
    },
    {
      "at": 0,
      "life": 0.800000011920929,
      "offset": [
        0,
        0.019999999552965164,
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
          2,
          1.5,
          0.6000000238418579,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/shield-ripple"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.30000001192092896
            ],
            [
              0.5,
              1.2999999523162842
            ],
            [
              1,
              1.7000000476837158
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
            "inner": 0.8399999737739563,
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
      "at": 0.20000000298023224,
      "life": 0.800000011920929,
      "offset": [
        0,
        0.019999999552965164,
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
          2,
          1.5,
          0.6000000238418579,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/shield-ripple"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.20000000298023224
            ],
            [
              0.5,
              0.8999999761581421
            ],
            [
              1,
              1.2000000476837158
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
        "asset": ref("base/vfx/cinematic-realism/emitters/rise-flecks")
      }
    },
    {
      "at": 0.30000001192092896,
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
        "seed": 3,
        "asset": ref("base/vfx/cinematic-realism/emitters/rise-flecks")
      }
    },
    {
      "at": 0,
      "life": 0.6000000238418579,
      "offset": [
        0,
        1,
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
        "kind": "glow",
        "color": [
          1.7999999523162842,
          1.399999976158142,
          0.6000000238418579,
          0.5
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.5
            ],
            [
              0.30000001192092896,
              2.200000047683716
            ],
            [
              1,
              1
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 1.5,
      "offset": [
        0,
        1.2000000476837158,
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
          0.8199999928474426,
          0.5
        ],
        "distance": 7,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.15000000596046448,
              4
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
        "kind": "sound",
        "pitch": [
          1,
          1
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/chime")
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
          1.2000000476837158,
          1.2999999523162842
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/swell")
      }
    }
  ]
});
