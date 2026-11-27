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
      "life": 0,
      "offset": [
        0,
        0.44999998807907104,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/motes")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.3499999940395355,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/glints")
      }
    },
    {
      "at": 0,
      "life": 0.5,
      "offset": [
        0,
        0.44999998807907104,
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
          1.399999976158142,
          0.800000011920929,
          0.20000000298023224,
          0.44999998807907104
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.10000000149011612
            ],
            [
              0.20000000298023224,
              0.550000011920929
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
      "life": 0.44999998807907104,
      "offset": [
        0,
        0.6499999761581421,
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
          0.6499999761581421,
          0.25
        ],
        "distance": 1.600000023841858,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.20000000298023224,
              0.20000000298023224
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    }
  ]
});
