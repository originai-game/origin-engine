import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 6,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [
    {
      "default": 1,
      "name": "mark"
    }
  ],
  "elements": [
    {
      "at": 0,
      "life": 0.44999998807907104,
      "offset": [
        0,
        0.800000011920929,
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
          0.5,
          1.100000023841858,
          2.799999952316284,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/energy-shell"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.30000001192092896
            ],
            [
              0.30000001192092896,
              1.600000023841858
            ],
            [
              1,
              2.4000000953674316
            ]
          ]
        },
        "shape": {
          "kind": "sphere"
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
      "life": 0.5,
      "offset": [
        0,
        0.05000000074505806,
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
          0.5,
          1,
          2.4000000953674316,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/shield-ripple"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.4000000059604645
            ],
            [
              0.4000000059604645,
              2.5999999046325684
            ],
            [
              1,
              3.5999999046325684
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
            "inner": 0.800000011920929,
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
        0.800000011920929,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/energy-flecks")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.800000011920929,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/warp-shock")
      }
    },
    {
      "at": 0,
      "life": 0.20000000298023224,
      "offset": [
        0,
        0.800000011920929,
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
          1.2000000476837158,
          1.7999999523162842,
          3.200000047683716,
          0.800000011920929
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              1
            ],
            [
              0.30000001192092896,
              3
            ],
            [
              1,
              1.5
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        0,
        0.8999999761581421,
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
          0.44999998807907104,
          0.699999988079071,
          1
        ],
        "distance": 9,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              12
            ],
            [
              0.20000000298023224,
              6
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
        "trauma": 0.3499999940395355
      }
    },
    {
      "at": 0.05000000074505806,
      "life": 5.949999809265137,
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
        "kind": "decal",
        "angleFade": 80,
        "color": [
          0.699999988079071,
          0.75,
          0.8500000238418579,
          0.800000011920929
        ],
        "emissive": [
          0,
          0,
          0
        ],
        "fade": 3,
        "normal": ref("base/vfx/cinematic-realism/textures/scorch-normal"),
        "normalScale": 1,
        "roughness": 0.8999999761581421,
        "size": [
          3,
          0.800000011920929,
          3
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/scorch"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      },
      "when": "mark"
    },
    {
      "at": 0.05000000074505806,
      "life": 3,
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
        "kind": "decal",
        "angleFade": 80,
        "color": [
          0.05000000074505806,
          0.05000000074505806,
          0.05999999865889549,
          1
        ],
        "emissive": [
          0.6000000238418579,
          1.399999976158142,
          3.200000047683716
        ],
        "fade": 2.5999999046325684,
        "normalScale": 1,
        "size": [
          1.600000023841858,
          0.800000011920929,
          1.600000023841858
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/embers"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      },
      "when": "mark"
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
          0.6000000238418579,
          0.699999988079071
        ],
        "volume": 0.8999999761581421,
        "asset": ref("base/vfx/cinematic-realism/audio/zap")
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
          0.800000011920929,
          0.8999999761581421
        ],
        "volume": 0.5,
        "asset": ref("base/vfx/cinematic-realism/audio/blast-small")
      }
    }
  ]
});
