import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.600000023841858,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.25
    }
  ],
  "params": [],
  "elements": [
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
              0.20000000298023224
            ],
            [
              0.800000011920929,
              1.399999976158142
            ],
            [
              1,
              0.4000000059604645
            ]
          ]
        }
      }
    },
    {
      "at": 0.25,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        180,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/warp-streaks")
      }
    },
    {
      "at": 0.25,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/warp-shock")
      }
    },
    {
      "at": 0.25,
      "life": 0.3499999940395355,
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
          1.399999976158142,
          2,
          3.200000047683716,
          0.800000011920929
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "jitter": 0,
        "segments": 1,
        "to": [
          0,
          12,
          0
        ],
        "width": 0.5
      }
    },
    {
      "at": 0.25,
      "life": 0.11999999731779099,
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
        "kind": "glow",
        "color": [
          1.600000023841858,
          2.200000047683716,
          3.4000000953674316,
          0.8999999761581421
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              2.5
            ],
            [
              1,
              0.5
            ]
          ]
        }
      }
    },
    {
      "at": 0.20000000298023224,
      "life": 0.4000000059604645,
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
          0.6000000238418579,
          0.800000011920929,
          1
        ],
        "distance": 10,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
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
      "at": 0.25,
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
        "trauma": 0.30000001192092896
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
          1.2999999523162842,
          1.2999999523162842
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/whoosh")
      }
    }
  ]
});
